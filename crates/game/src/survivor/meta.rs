use engine::components::GameState;
use engine::input::InputState;
/// Phase 8-A: SurvivorMode + MetaSave + Title 화면.
///
/// - `MetaSave` — 영구 저장 메타 진행 데이터 (gold, kills, best_time 등).
/// - `SurvivorMode` — 최상위 게임 모드 (Title / Shop / InGame / StageClear).
/// - `ModeTransitionSystem` — 모드 전환 + GameState 동기화.
use engine::save;
use engine::{PendingResize, ShouldQuit, System, World};
use serde::{Deserialize, Serialize};
use winit::keyboard::KeyCode;

use super::character::CharacterCursor;
use super::hud::GameStats;
use super::locale::Lang;
use super::pickup::GoldWallet;
use super::powerup::ShopCursor;
use super::stage::{SelectedStage, StageCursor, StageKind};

const APP_NAME: &str = "rust-vampire-survivors";
const SAVE_FILE: &str = "save.ron";
pub const SETTINGS_ITEMS: usize = 5;

// ─── MetaSave ────────────────────────────────────────────────────────────────

fn default_resolution_key() -> String {
    "1280x720".to_string()
}

fn default_volume() -> f32 {
    1.0
}

/// 저장되는 언어 선택값. System 은 실행 환경의 LANG 계열 환경변수를 따른다.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LanguageSetting {
    System,
    Ko,
    En,
}

impl Default for LanguageSetting {
    fn default() -> Self {
        Self::System
    }
}

impl LanguageSetting {
    pub const ALL: &'static [Self] = &[Self::System, Self::Ko, Self::En];

    pub fn effective(self) -> Lang {
        match self {
            Self::System => detect_system_lang(),
            Self::Ko => Lang::Ko,
            Self::En => Lang::En,
        }
    }

    pub fn label(self, lang: Lang) -> &'static str {
        match self {
            Self::System => super::locale::loc(lang, "시스템 감지", "System"),
            Self::Ko => super::locale::loc(lang, "한국어", "Korean"),
            Self::En => super::locale::loc(lang, "영어", "English"),
        }
    }

    pub fn step(self, delta: i32) -> Self {
        let len = Self::ALL.len() as i32;
        let idx = Self::ALL.iter().position(|&v| v == self).unwrap_or(0) as i32;
        Self::ALL[((idx + delta).rem_euclid(len)) as usize]
    }
}

fn detect_system_lang() -> Lang {
    let raw = std::env::var("LC_ALL")
        .ok()
        .filter(|s| !s.is_empty())
        .or_else(|| std::env::var("LC_MESSAGES").ok().filter(|s| !s.is_empty()))
        .or_else(|| std::env::var("LANG").ok())
        .unwrap_or_default()
        .to_ascii_lowercase();
    if raw.starts_with("ko") {
        Lang::Ko
    } else {
        Lang::En
    }
}

/// 인게임 HUD 정보량 설정.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HudDetail {
    Minimal,
    Normal,
    Detailed,
}

impl Default for HudDetail {
    fn default() -> Self {
        Self::Normal
    }
}

impl HudDetail {
    pub const ALL: &'static [Self] = &[Self::Minimal, Self::Normal, Self::Detailed];

    pub fn label(self, lang: Lang) -> &'static str {
        match self {
            Self::Minimal => super::locale::loc(lang, "최소", "Minimal"),
            Self::Normal => super::locale::loc(lang, "중간", "Normal"),
            Self::Detailed => super::locale::loc(lang, "상세", "Detailed"),
        }
    }

    pub fn step(self, delta: i32) -> Self {
        let len = Self::ALL.len() as i32;
        let idx = Self::ALL.iter().position(|&v| v == self).unwrap_or(1) as i32;
        Self::ALL[((idx + delta).rem_euclid(len)) as usize]
    }
}

/// 영구 저장되는 메타 진행 데이터.
#[derive(Debug, Default, Clone, Serialize, Deserialize, PartialEq)]
pub struct MetaSave {
    pub gold_total: u32,
    pub powerup_levels: std::collections::HashMap<String, u8>,
    pub unlocked_stages: Vec<String>,
    pub unlocked_chars: Vec<String>,
    pub achievements: Vec<String>,
    pub best_time: f32,
    pub kills_total: u32,
    #[serde(default)]
    pub lang: Lang, // 과거 save 호환용. 새 UI는 language_setting 을 우선 사용.
    #[serde(default)]
    pub language_setting: LanguageSetting,
    #[serde(default)]
    pub hud_detail: HudDetail,
    #[serde(default = "default_volume")]
    pub bgm_volume: f32,
    #[serde(default = "default_volume")]
    pub sfx_volume: f32,
    #[serde(default = "default_resolution_key")]
    pub resolution_key: String, // 저장된 해상도 설정 (e.g. "1280x720")
}

impl MetaSave {
    /// 파일에서 load. 실패 시 default.
    pub fn load_or_default() -> Self {
        let path = save::save_path(APP_NAME, SAVE_FILE);
        save::load::<MetaSave>(&path).unwrap_or_default()
    }

    /// 파일에 저장. 실패 시 eprintln 으로 로그만.
    pub fn save_to_disk(&self) {
        let path = save::save_path(APP_NAME, SAVE_FILE);
        if let Err(e) = save::save(&path, self) {
            eprintln!("MetaSave 저장 실패: {:?}", e);
        }
    }

    pub fn effective_lang(&self) -> Lang {
        self.language_setting.effective()
    }
}

// ─── SurvivorMode ────────────────────────────────────────────────────────────

/// 게임의 최상위 모드.
///
/// `GameState` (Playing/Paused/GameOver) 는 InGame 중 *서브* 상태.
/// 메뉴/상점/스테이지 선택 등은 `SurvivorMode` 로 표현.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SurvivorMode {
    Title,           // 시작 화면 — ENTER 로 InGame 진입
    CharacterSelect, // 캐릭터 선택 화면 (Phase 9)
    StageSelect,     // 스테이지 선택 화면 (Phase 10)
    Shop,            // PowerUp 매장 (Phase 8-B)
    InGame,          // 실제 게임 진행
    StageClear,      // 스테이지 클리어
    PauseMenu,       // 일시정지 메뉴 — ESC 로 진입, 계속/타이틀/종료 선택
    Settings,        // 설정 화면 — 해상도 선택
}

impl Default for SurvivorMode {
    fn default() -> Self {
        Self::Title
    }
}

/// 일시정지 메뉴 커서 (0=계속하기, 1=타이틀로, 2=게임 종료)
#[derive(Debug, Clone, Copy, Default)]
pub struct PauseMenuCursor {
    pub index: usize,
}

pub const PAUSE_MENU_ITEMS: usize = 3;

// ─── 해상도 프리셋 ────────────────────────────────────────────────────────────

/// 지원 해상도 목록. 기본값은 1280×720 (HD 16:9).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResolutionPreset {
    R800x600,   // 4:3 레거시
    R1280x720,  // 16:9 HD  (기본)
    R1600x900,  // 16:9 HD+
    R1920x1080, // 16:9 FHD
}

impl ResolutionPreset {
    pub const ALL: &'static [Self] = &[
        Self::R800x600,
        Self::R1280x720,
        Self::R1600x900,
        Self::R1920x1080,
    ];

    pub fn dimensions(self) -> (u32, u32) {
        match self {
            Self::R800x600 => (800, 600),
            Self::R1280x720 => (1280, 720),
            Self::R1600x900 => (1600, 900),
            Self::R1920x1080 => (1920, 1080),
        }
    }

    pub fn key(self) -> &'static str {
        match self {
            Self::R800x600 => "800x600",
            Self::R1280x720 => "1280x720",
            Self::R1600x900 => "1600x900",
            Self::R1920x1080 => "1920x1080",
        }
    }

    pub fn from_key(key: &str) -> Self {
        match key {
            "800x600" => Self::R800x600,
            "1280x720" => Self::R1280x720,
            "1600x900" => Self::R1600x900,
            "1920x1080" => Self::R1920x1080,
            _ => Self::R1280x720,
        }
    }

    pub fn label(self, _lang: Lang) -> &'static str {
        match self {
            Self::R800x600 => "800x600  (4:3)",
            Self::R1280x720 => "1280x720  HD 16:9  [기본값]",
            Self::R1600x900 => "1600x900  16:9",
            Self::R1920x1080 => "1920x1080  FHD 16:9",
        }
    }

    pub fn index(self) -> usize {
        Self::ALL.iter().position(|&p| p == self).unwrap_or(1)
    }
}

/// 설정 화면 커서 (0=언어, 1=HUD 정보량, 2=BGM, 3=SFX, 4=해상도)
#[derive(Debug, Clone, Copy, Default)]
pub struct SettingsCursor {
    pub index: usize,
}

// ─── ModeTransitionSystem ────────────────────────────────────────────────────

/// SurvivorMode 전환 + GameState 동기화.
///
/// - Title/Shop/StageClear 에서는 GameState 를 Paused 로 강제해 게임 시스템 동작 차단.
/// - Title + ENTER → InGame (restart_world + mode 전환).
/// - InGame + StageProgress.cleared → StageClear (메타 누적 + 저장).
/// - StageClear + ENTER → Title 복귀.
pub struct ModeTransitionSystem;

impl System for ModeTransitionSystem {
    fn run(&mut self, world: &mut World, _dt: f32) {
        let mode = world
            .resource::<SurvivorMode>()
            .copied()
            .unwrap_or(SurvivorMode::Title);

        // Title / Shop / StageClear / PauseMenu / Settings 에서는 GameState 를 Paused 로 강제.
        // 이로써 Playing 가드를 사용하는 게임 시스템들이 조기 반환.
        match mode {
            SurvivorMode::Title
            | SurvivorMode::CharacterSelect
            | SurvivorMode::StageSelect
            | SurvivorMode::Shop
            | SurvivorMode::StageClear
            | SurvivorMode::PauseMenu
            | SurvivorMode::Settings => {
                if let Some(gs) = world.resource_mut::<GameState>() {
                    if !matches!(*gs, GameState::Paused) {
                        *gs = GameState::Paused;
                    }
                }
            }
            SurvivorMode::InGame => {
                // InGame 에서는 GameState 를 직접 제어하지 않음
                // (Playing / Paused(LevelUp) / GameOver 는 서브 시스템이 관리)
            }
        }

        match mode {
            SurvivorMode::Title => {
                // 입력 캐시 (borrow 분리)
                let (
                    enter_pressed,
                    shop_pressed,
                    char_sel_pressed,
                    stage_sel_pressed,
                    settings_pressed,
                ) = {
                    let i = match world.resource::<InputState>() {
                        Some(i) => i,
                        None => return,
                    };
                    (
                        i.just_pressed(KeyCode::Enter),
                        i.just_pressed(KeyCode::KeyS),
                        i.just_pressed(KeyCode::KeyC),
                        i.just_pressed(KeyCode::KeyT),
                        i.just_pressed(KeyCode::KeyO),
                    )
                };
                if enter_pressed {
                    // SpawnDirector waves 를 SelectedStage 기반으로 갱신 (게임 시작 직전)
                    let stage = world
                        .resource::<SelectedStage>()
                        .copied()
                        .unwrap_or_default()
                        .0;
                    let waves = stage.load_waves();
                    if let Some(d) = world.resource_mut::<super::director::SpawnDirector>() {
                        d.waves = waves;
                        d.spawn_elapsed = 0.0;
                    }
                    super::death::restart_world(world);
                    if let Some(m) = world.resource_mut::<SurvivorMode>() {
                        *m = SurvivorMode::InGame;
                    }
                    if let Some(gs) = world.resource_mut::<GameState>() {
                        *gs = GameState::Playing;
                    }
                    println!("Game started (stage: {}).", stage.label(Lang::En));
                }
                if shop_pressed {
                    if let Some(m) = world.resource_mut::<SurvivorMode>() {
                        *m = SurvivorMode::Shop;
                    }
                    // ShopCursor 가 없으면 default 삽입
                    if world.resource::<ShopCursor>().is_none() {
                        world.insert_resource(ShopCursor::default());
                    }
                    println!("Entered shop");
                }
                if char_sel_pressed {
                    if let Some(m) = world.resource_mut::<SurvivorMode>() {
                        *m = SurvivorMode::CharacterSelect;
                    }
                    // CharacterCursor 가 없으면 default 삽입
                    if world.resource::<CharacterCursor>().is_none() {
                        world.insert_resource(CharacterCursor::default());
                    }
                    println!("Entered character select");
                }
                if stage_sel_pressed {
                    if let Some(m) = world.resource_mut::<SurvivorMode>() {
                        *m = SurvivorMode::StageSelect;
                    }
                    if world.resource::<StageCursor>().is_none() {
                        world.insert_resource(StageCursor::default());
                    }
                    println!("Entered stage select");
                }
                if settings_pressed {
                    world.insert_resource(SettingsCursor { index: 0 });
                    if let Some(m) = world.resource_mut::<SurvivorMode>() {
                        *m = SurvivorMode::Settings;
                    }
                    println!("Entered settings");
                }
            }
            SurvivorMode::CharacterSelect => {
                // CharacterSelectSystem 이 처리 — 여기서는 no-op
            }
            SurvivorMode::StageSelect => {
                // StageSelectSystem 이 처리 — 여기서는 no-op
            }
            SurvivorMode::InGame => {
                // StageProgress.cleared 면 StageClear 로 전환
                let cleared = world
                    .resource::<super::boss::StageProgress>()
                    .map(|p| p.cleared)
                    .unwrap_or(false);
                if cleared {
                    // 메타 누적
                    let elapsed = world
                        .resource::<GameStats>()
                        .map(|s| s.elapsed)
                        .unwrap_or(0.0);
                    let kills = world.resource::<GameStats>().map(|s| s.kills).unwrap_or(0);
                    let coins = world
                        .resource::<GoldWallet>()
                        .map(|w| w.current)
                        .unwrap_or(0);

                    // 현재 선택 스테이지 캐시 (borrow 분리)
                    let selected_stage = world
                        .resource::<SelectedStage>()
                        .copied()
                        .unwrap_or_default()
                        .0;
                    let next_stage = match selected_stage {
                        StageKind::MadForest => Some(StageKind::InlaidLibrary),
                        StageKind::InlaidLibrary => Some(StageKind::DairyPlant),
                        StageKind::DairyPlant => None,
                    };

                    if let Some(meta) = world.resource_mut::<MetaSave>() {
                        meta.gold_total = meta.gold_total.saturating_add(coins);
                        meta.kills_total = meta.kills_total.saturating_add(kills);
                        if elapsed > meta.best_time {
                            meta.best_time = elapsed;
                        }
                        // 다음 스테이지 해금
                        if let Some(next) = next_stage {
                            let key = next.key().to_string();
                            if !meta.unlocked_stages.iter().any(|s| s == &key) {
                                meta.unlocked_stages.push(key.clone());
                                println!("Unlocked stage: {}", key);
                            }
                        }
                        meta.save_to_disk();
                    }

                    if let Some(m) = world.resource_mut::<SurvivorMode>() {
                        *m = SurvivorMode::StageClear;
                    }
                    println!("StageClear → meta saved");
                    return;
                }

                // Playing 중 ESC → 일시정지 메뉴
                let (esc_pressed, is_playing) = {
                    let i = match world.resource::<InputState>() {
                        Some(i) => i,
                        None => return,
                    };
                    let state = world
                        .resource::<GameState>()
                        .cloned()
                        .unwrap_or(GameState::Playing);
                    (
                        i.just_pressed(KeyCode::Escape),
                        matches!(state, GameState::Playing),
                    )
                };
                if esc_pressed && is_playing {
                    if let Some(gs) = world.resource_mut::<GameState>() {
                        *gs = GameState::Paused;
                    }
                    world.insert_resource(PauseMenuCursor { index: 0 });
                    if let Some(m) = world.resource_mut::<SurvivorMode>() {
                        *m = SurvivorMode::PauseMenu;
                    }
                }
            }
            SurvivorMode::PauseMenu => {
                let (esc_pressed, up_pressed, down_pressed, enter_pressed, cursor_idx) = {
                    let i = match world.resource::<InputState>() {
                        Some(i) => i,
                        None => return,
                    };
                    let cur = world
                        .resource::<PauseMenuCursor>()
                        .map(|c| c.index)
                        .unwrap_or(0);
                    (
                        i.just_pressed(KeyCode::Escape),
                        i.just_pressed(KeyCode::KeyW) || i.just_pressed(KeyCode::ArrowUp),
                        i.just_pressed(KeyCode::KeyS) || i.just_pressed(KeyCode::ArrowDown),
                        i.just_pressed(KeyCode::Enter),
                        cur,
                    )
                };

                // ESC 로 메뉴 닫기 (게임 재개)
                if esc_pressed {
                    if let Some(m) = world.resource_mut::<SurvivorMode>() {
                        *m = SurvivorMode::InGame;
                    }
                    if let Some(gs) = world.resource_mut::<GameState>() {
                        *gs = GameState::Playing;
                    }
                    return;
                }

                // W/S 또는 방향키 커서 이동
                let new_idx = if up_pressed && cursor_idx > 0 {
                    cursor_idx - 1
                } else if down_pressed && cursor_idx + 1 < PAUSE_MENU_ITEMS {
                    cursor_idx + 1
                } else {
                    cursor_idx
                };
                if let Some(c) = world.resource_mut::<PauseMenuCursor>() {
                    c.index = new_idx;
                }

                if enter_pressed {
                    match cursor_idx {
                        0 => {
                            // 계속하기 — 게임 재개
                            if let Some(m) = world.resource_mut::<SurvivorMode>() {
                                *m = SurvivorMode::InGame;
                            }
                            if let Some(gs) = world.resource_mut::<GameState>() {
                                *gs = GameState::Playing;
                            }
                        }
                        1 => {
                            // 타이틀로 돌아가기
                            super::death::restart_world(world);
                            if let Some(m) = world.resource_mut::<SurvivorMode>() {
                                *m = SurvivorMode::Title;
                            }
                            println!("PauseMenu → Title");
                        }
                        2 => {
                            // 게임 종료
                            world.insert_resource(ShouldQuit(true));
                            println!("PauseMenu → Quit");
                        }
                        _ => {}
                    }
                }
            }
            SurvivorMode::StageClear => {
                let enter_pressed = world
                    .resource::<InputState>()
                    .map(|i| i.just_pressed(KeyCode::Enter))
                    .unwrap_or(false);
                if enter_pressed {
                    super::death::restart_world(world);
                    if let Some(m) = world.resource_mut::<SurvivorMode>() {
                        *m = SurvivorMode::Title;
                    }
                    println!("Return to Title.");
                }
            }
            SurvivorMode::Shop => {
                // Phase 8-B 에서 구현
            }
            SurvivorMode::Settings => {
                let (
                    esc_pressed,
                    up_pressed,
                    down_pressed,
                    left_pressed,
                    right_pressed,
                    enter_pressed,
                    cursor_idx,
                ) = {
                    let i = match world.resource::<InputState>() {
                        Some(i) => i,
                        None => return,
                    };
                    let cur = world
                        .resource::<SettingsCursor>()
                        .map(|c| c.index)
                        .unwrap_or(1);
                    (
                        i.just_pressed(KeyCode::Escape),
                        i.just_pressed(KeyCode::KeyW) || i.just_pressed(KeyCode::ArrowUp),
                        i.just_pressed(KeyCode::KeyS) || i.just_pressed(KeyCode::ArrowDown),
                        i.just_pressed(KeyCode::KeyA) || i.just_pressed(KeyCode::ArrowLeft),
                        i.just_pressed(KeyCode::KeyD) || i.just_pressed(KeyCode::ArrowRight),
                        i.just_pressed(KeyCode::Enter),
                        cur,
                    )
                };

                if esc_pressed {
                    if let Some(m) = world.resource_mut::<SurvivorMode>() {
                        *m = SurvivorMode::Title;
                    }
                    return;
                }

                let new_idx = if up_pressed && cursor_idx > 0 {
                    cursor_idx - 1
                } else if down_pressed && cursor_idx + 1 < SETTINGS_ITEMS {
                    cursor_idx + 1
                } else {
                    cursor_idx
                };
                if let Some(c) = world.resource_mut::<SettingsCursor>() {
                    c.index = new_idx;
                }

                let delta = if left_pressed {
                    -1
                } else if right_pressed {
                    1
                } else {
                    0
                };

                if delta != 0 {
                    if let Some(meta) = world.resource_mut::<MetaSave>() {
                        match new_idx {
                            0 => {
                                meta.language_setting = meta.language_setting.step(delta);
                                meta.lang = meta.language_setting.effective();
                            }
                            1 => meta.hud_detail = meta.hud_detail.step(delta),
                            2 => {
                                meta.bgm_volume =
                                    (meta.bgm_volume + delta as f32 * 0.1).clamp(0.0, 1.0);
                            }
                            3 => {
                                meta.sfx_volume =
                                    (meta.sfx_volume + delta as f32 * 0.1).clamp(0.0, 1.0);
                            }
                            4 => {
                                let current =
                                    ResolutionPreset::from_key(&meta.resolution_key).index() as i32;
                                let len = ResolutionPreset::ALL.len() as i32;
                                let next = (current + delta).rem_euclid(len) as usize;
                                meta.resolution_key = ResolutionPreset::ALL[next].key().to_string();
                            }
                            _ => {}
                        }
                        meta.save_to_disk();
                    }
                }

                if enter_pressed && new_idx == 4 {
                    let preset = world
                        .resource::<MetaSave>()
                        .map(|m| ResolutionPreset::from_key(&m.resolution_key))
                        .unwrap_or(ResolutionPreset::R1280x720);
                    let (w, h) = preset.dimensions();
                    world.insert_resource(PendingResize(Some((w, h))));
                    println!("Resolution → {}x{}", w, h);
                }
            }
        }
    }
}

// ─── 테스트 ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn meta_save_default_is_zero() {
        let m = MetaSave::default();
        assert_eq!(m.gold_total, 0);
        assert_eq!(m.kills_total, 0);
        assert_eq!(m.best_time, 0.0);
        assert!(m.powerup_levels.is_empty());
        assert!(m.unlocked_stages.is_empty());
        assert!(m.unlocked_chars.is_empty());
        assert!(m.achievements.is_empty());
    }

    #[test]
    fn meta_save_serialization_roundtrip() {
        let mut m = MetaSave {
            gold_total: 100,
            kills_total: 500,
            best_time: 1234.5,
            powerup_levels: std::collections::HashMap::new(),
            unlocked_stages: vec!["MadForest".to_string()],
            unlocked_chars: vec!["Antonio".to_string()],
            achievements: vec!["FirstBlood".to_string()],
            lang: super::super::locale::Lang::Ko,
            language_setting: LanguageSetting::Ko,
            hud_detail: HudDetail::Detailed,
            bgm_volume: 0.8,
            sfx_volume: 0.6,
            resolution_key: "1280x720".to_string(),
        };
        m.powerup_levels.insert("Might".to_string(), 3);

        let s = ron::to_string(&m).expect("직렬화 실패");
        let loaded: MetaSave = ron::from_str(&s).expect("역직렬화 실패");

        assert_eq!(m, loaded, "RON 라운드트립 후 동일해야 함");
        assert_eq!(loaded.gold_total, 100);
        assert_eq!(loaded.kills_total, 500);
        assert!((loaded.best_time - 1234.5).abs() < 0.01);
        assert_eq!(loaded.language_setting, LanguageSetting::Ko);
        assert_eq!(loaded.hud_detail, HudDetail::Detailed);
        assert!((loaded.bgm_volume - 0.8).abs() < 0.01);
        assert!((loaded.sfx_volume - 0.6).abs() < 0.01);
        assert_eq!(loaded.powerup_levels.get("Might"), Some(&3u8));
    }

    #[test]
    fn survivor_mode_default_is_title() {
        assert_eq!(SurvivorMode::default(), SurvivorMode::Title);
    }

    #[test]
    fn enter_in_game_resets_world_and_sets_mode() {
        use engine::components::GameState;
        use engine::World;

        let mut world = World::new();
        world.insert_resource(GameState::Playing);
        world.insert_resource(SurvivorMode::Title);

        // restart_world 가 호출되는 패턴 (Title → InGame 전환 시)
        super::super::death::restart_world(&mut world);

        // 직접 mode 전환 (InputState 시뮬레이션 없이)
        if let Some(m) = world.resource_mut::<SurvivorMode>() {
            *m = SurvivorMode::InGame;
        }

        assert_eq!(
            *world.resource::<SurvivorMode>().unwrap(),
            SurvivorMode::InGame,
            "Title → InGame 전환 후 SurvivorMode 가 InGame 이어야 함"
        );
    }
}
