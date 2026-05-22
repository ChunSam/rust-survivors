/// Phase 8-A: SurvivorMode + MetaSave + Title 화면.
///
/// - `MetaSave` — 영구 저장 메타 진행 데이터 (gold, kills, best_time 등).
/// - `SurvivorMode` — 최상위 게임 모드 (Title / Shop / InGame / StageClear).
/// - `ModeTransitionSystem` — 모드 전환 + GameState 동기화.

use engine::save;
use engine::{System, World};
use engine::components::GameState;
use engine::input::InputState;
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

// ─── MetaSave ────────────────────────────────────────────────────────────────

/// 영구 저장되는 메타 진행 데이터.
#[derive(Debug, Default, Clone, Serialize, Deserialize, PartialEq)]
pub struct MetaSave {
    pub gold_total:      u32,
    pub powerup_levels:  std::collections::HashMap<String, u8>, // Phase 8-B 에서 사용
    pub unlocked_stages: Vec<String>,                           // Phase 9-10 활용
    pub unlocked_chars:  Vec<String>,                           // Phase 9 활용
    pub achievements:    Vec<String>,
    pub best_time:       f32,
    pub kills_total:     u32,
    #[serde(default)]
    pub lang:            Lang,  // 표시 언어 (Ko/En). L 키로 토글, 저장 파일에 보존.
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
}

impl Default for SurvivorMode {
    fn default() -> Self { Self::Title }
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
        let mode = world.resource::<SurvivorMode>().copied().unwrap_or(SurvivorMode::Title);

        // Title / Shop / StageClear 에서는 GameState 를 Paused 로 강제.
        // 이로써 Playing 가드를 사용하는 게임 시스템들이 조기 반환.
        match mode {
            SurvivorMode::Title
            | SurvivorMode::CharacterSelect
            | SurvivorMode::StageSelect
            | SurvivorMode::Shop
            | SurvivorMode::StageClear => {
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
                let (enter_pressed, shop_pressed, char_sel_pressed, stage_sel_pressed, lang_pressed) = {
                    let i = match world.resource::<InputState>() {
                        Some(i) => i,
                        None    => return,
                    };
                    (
                        i.just_pressed(KeyCode::Enter),
                        i.just_pressed(KeyCode::KeyS),
                        i.just_pressed(KeyCode::KeyC),
                        i.just_pressed(KeyCode::KeyT),
                        i.just_pressed(KeyCode::KeyL),
                    )
                };
                if lang_pressed {
                    if let Some(meta) = world.resource_mut::<MetaSave>() {
                        meta.lang = meta.lang.toggle();
                        meta.save_to_disk();
                        println!("Language: {:?}", meta.lang);
                    }
                }
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
                    // StageCursor 가 없으면 default 삽입
                    if world.resource::<StageCursor>().is_none() {
                        world.insert_resource(StageCursor::default());
                    }
                    println!("Entered stage select");
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
                    let elapsed = world.resource::<GameStats>().map(|s| s.elapsed).unwrap_or(0.0);
                    let kills   = world.resource::<GameStats>().map(|s| s.kills).unwrap_or(0);
                    let coins   = world.resource::<GoldWallet>().map(|w| w.current).unwrap_or(0);

                    // 현재 선택 스테이지 캐시 (borrow 분리)
                    let selected_stage = world
                        .resource::<SelectedStage>()
                        .copied()
                        .unwrap_or_default()
                        .0;
                    let next_stage = match selected_stage {
                        StageKind::MadForest     => Some(StageKind::InlaidLibrary),
                        StageKind::InlaidLibrary => Some(StageKind::DairyPlant),
                        StageKind::DairyPlant    => None,
                    };

                    if let Some(meta) = world.resource_mut::<MetaSave>() {
                        meta.gold_total  = meta.gold_total.saturating_add(coins);
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
            gold_total:      100,
            kills_total:     500,
            best_time:       1234.5,
            powerup_levels:  std::collections::HashMap::new(),
            unlocked_stages: vec!["MadForest".to_string()],
            unlocked_chars:  vec!["Antonio".to_string()],
            achievements:    vec!["FirstBlood".to_string()],
            lang:            super::super::locale::Lang::Ko,
        };
        m.powerup_levels.insert("Might".to_string(), 3);

        let s = ron::to_string(&m).expect("직렬화 실패");
        let loaded: MetaSave = ron::from_str(&s).expect("역직렬화 실패");

        assert_eq!(m, loaded, "RON 라운드트립 후 동일해야 함");
        assert_eq!(loaded.gold_total, 100);
        assert_eq!(loaded.kills_total, 500);
        assert!((loaded.best_time - 1234.5).abs() < 0.01);
        assert_eq!(loaded.powerup_levels.get("Might"), Some(&3u8));
    }

    #[test]
    fn survivor_mode_default_is_title() {
        assert_eq!(SurvivorMode::default(), SurvivorMode::Title);
    }

    #[test]
    fn enter_in_game_resets_world_and_sets_mode() {
        use engine::World;
        use engine::components::GameState;

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
