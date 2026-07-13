//! 효과음(SFX) 큐 + 재생 시스템 — Phase 11-D.
//!
//! 오디오 파일 없이 `AudioManager::play_tone` (사인파 합성) 으로 SFX 를 생성한다.
//!
//! ## 흐름
//! 1. 여러 시스템이 `SfxQueue` 에 `SfxEvent` 를 push (AudioManager borrow 불필요)
//! 2. `SfxSystem` 이 매 프레임 끝에 큐를 drain 하고 AudioManager 로 재생
//!
//! ## 같은 프레임 중복 방지
//! 피격 등 동일 이벤트가 프레임당 수십 번 발생할 수 있으므로
//! 같은 카테고리 이벤트는 프레임당 최대 횟수를 제한한다.

use engine::{AudioManager, System, World};

use super::meta::MetaSave;

const AUDIO_DIR: &str = "assets/audio";
const EXTENSIONS: &[&str] = &["ogg", "wav"];
const SFX_BUS: &str = "sfx";

fn find_audio_file(key: &str) -> Option<String> {
    for ext in EXTENSIONS {
        let path = format!("{AUDIO_DIR}/{key}.{ext}");
        if std::path::Path::new(&path).exists() {
            return Some(path);
        }
    }
    None
}

fn prepare_sfx_channel(audio: &mut AudioManager, channel: &str, gain: f32) {
    audio.assign_bus(channel, SFX_BUS);
    audio.set_volume(channel, gain);
}

fn play_file(audio: &mut AudioManager, channel: &str, key: &str, gain: f32) -> bool {
    let Some(path) = find_audio_file(key) else {
        return false;
    };
    audio.play(channel, &path, false);
    prepare_sfx_channel(audio, channel, gain);
    true
}

fn play_tone(audio: &mut AudioManager, channel: &str, freq: f32, duration_secs: f32, gain: f32) {
    audio.assign_bus(channel, SFX_BUS);
    audio.play_tone(channel, freq, duration_secs, gain);
}

fn base_gains(event: SfxEvent) -> &'static [f32] {
    match event {
        SfxEvent::EnemyHit => &[0.22],
        SfxEvent::EnemyDie => &[0.35],
        SfxEvent::PlayerHit => &[0.50],
        SfxEvent::LevelUp => &[0.55, 0.55],
        SfxEvent::XpGem => &[0.12],
        SfxEvent::Pickup => &[0.40],
        SfxEvent::Bomb => &[0.70, 0.45],
        SfxEvent::ChestOpen => &[0.45, 0.35],
        SfxEvent::BossAppear => &[0.70, 0.45],
    }
}

fn base_gain(event: SfxEvent, index: usize) -> f32 {
    base_gains(event)[index]
}

/// SFX 이벤트 종류.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SfxEvent {
    EnemyHit,   // 적 피격 (단발 틱음)
    EnemyDie,   // 적 사망 (짧은 저음)
    PlayerHit,  // 플레이어 피격 (중간 충격음)
    LevelUp,    // 레벨업 (상승 화음)
    XpGem,      // 경험치 젬 수집 (고음 핑)
    Pickup,     // 일반 픽업 (밝은 핑)
    Bomb,       // 폭탄 폭발 (저주파 굉음)
    ChestOpen,  // 보물상자 열림
    BossAppear, // 보스 등장
}

impl SfxEvent {
    pub const ALL: &'static [Self] = &[
        Self::EnemyHit,
        Self::EnemyDie,
        Self::PlayerHit,
        Self::LevelUp,
        Self::XpGem,
        Self::Pickup,
        Self::Bomb,
        Self::ChestOpen,
        Self::BossAppear,
    ];

    pub fn file_key(self) -> &'static str {
        match self {
            Self::EnemyHit => "sfx_enemy_hit",
            Self::EnemyDie => "sfx_enemy_die",
            Self::PlayerHit => "sfx_player_hit",
            Self::LevelUp => "sfx_levelup",
            Self::XpGem => "sfx_xp",
            Self::Pickup => "sfx_pickup",
            Self::Bomb => "sfx_bomb",
            Self::ChestOpen => "sfx_chest_open",
            Self::BossAppear => "sfx_boss_appear",
        }
    }
}

/// 프레임 단위 SFX 이벤트 버퍼. ECS 리소스로 삽입.
#[derive(Default)]
pub struct SfxQueue {
    pub events: Vec<SfxEvent>,
}

impl SfxQueue {
    pub fn push(&mut self, e: SfxEvent) {
        self.events.push(e);
    }
}

/// SfxQueue 를 drain 해서 AudioManager 로 재생하는 시스템.
/// HudSystem 다음(최후)에 등록한다.
#[derive(Default)]
pub struct SfxSystem {
    events: Vec<SfxEvent>,
}

impl System for SfxSystem {
    fn run(&mut self, world: &mut World, _dt: f32) {
        // 1) 큐 drain — AudioManager 와의 이중 borrow 를 피하기 위해 먼저 빼낸다
        self.events.clear();
        if let Some(q) = world.resource_mut::<SfxQueue>() {
            self.events.append(&mut q.events);
        } else {
            return;
        }
        if self.events.is_empty() {
            return;
        }

        let volume = world
            .resource::<MetaSave>()
            .map(|m| m.sfx_volume)
            .unwrap_or(1.0);
        let Some(audio) = world.resource_mut::<AudioManager>() else {
            return;
        };
        audio.set_bus_volume(SFX_BUS, volume);

        // 2) 카테고리별 프레임 상한으로 스팸 방지
        let mut hit_n = 0u8;
        let mut die_n = 0u8;
        let mut xp_n = 0u8;

        for event in self.events.drain(..) {
            match event {
                SfxEvent::EnemyHit => {
                    if hit_n < 2 {
                        // 짧은 고음 틱 — 여러 무기가 동시에 때려도 2회로 제한
                        let ch = format!("sfx_hit_{hit_n}");
                        let gain = base_gain(event, 0);
                        if !play_file(audio, &ch, event.file_key(), gain) {
                            play_tone(audio, &ch, 420.0, 0.045, gain);
                        }
                        hit_n += 1;
                    }
                }
                SfxEvent::EnemyDie => {
                    if die_n < 2 {
                        let ch = format!("sfx_die_{die_n}");
                        let gain = base_gain(event, 0);
                        if !play_file(audio, &ch, event.file_key(), gain) {
                            play_tone(audio, &ch, 160.0, 0.13, gain);
                        }
                        die_n += 1;
                    }
                }
                SfxEvent::PlayerHit => {
                    // 플레이어 피격은 게임당 cooldown 이 있어 많아야 1회/초
                    let gain = base_gain(event, 0);
                    if !play_file(audio, "sfx_player_hit", event.file_key(), gain) {
                        play_tone(audio, "sfx_player_hit", 200.0, 0.12, gain);
                    }
                }
                SfxEvent::LevelUp => {
                    // 상승하는 두 음 — 두 번째 톤으로 즉시 덮어쓰지 않도록 다른 채널
                    let gains = base_gains(event);
                    if !play_file(audio, "sfx_levelup", event.file_key(), gains[0]) {
                        play_tone(audio, "sfx_lvl_lo", 440.0, 0.18, gains[0]);
                        play_tone(audio, "sfx_lvl_hi", 660.0, 0.25, gains[1]);
                    }
                }
                SfxEvent::XpGem => {
                    if xp_n < 3 {
                        let ch = format!("sfx_xp_{xp_n}");
                        let gain = base_gain(event, 0);
                        if !play_file(audio, &ch, event.file_key(), gain) {
                            play_tone(audio, &ch, 880.0, 0.04, gain);
                        }
                        xp_n += 1;
                    }
                }
                SfxEvent::Pickup => {
                    let gain = base_gain(event, 0);
                    if !play_file(audio, "sfx_pickup", event.file_key(), gain) {
                        play_tone(audio, "sfx_pickup", 1040.0, 0.09, gain);
                    }
                }
                SfxEvent::Bomb => {
                    // 저주파 굉음 + 하모닉
                    let gains = base_gains(event);
                    if !play_file(audio, "sfx_bomb", event.file_key(), gains[0]) {
                        play_tone(audio, "sfx_bomb_lo", 60.0, 0.45, gains[0]);
                        play_tone(audio, "sfx_bomb_hi", 120.0, 0.30, gains[1]);
                    }
                }
                SfxEvent::ChestOpen => {
                    let gains = base_gains(event);
                    if !play_file(audio, "sfx_chest_open", event.file_key(), gains[0]) {
                        play_tone(audio, "sfx_chest_lo", 520.0, 0.12, gains[0]);
                        play_tone(audio, "sfx_chest_hi", 780.0, 0.18, gains[1]);
                    }
                }
                SfxEvent::BossAppear => {
                    let gains = base_gains(event);
                    if !play_file(audio, "sfx_boss_appear", event.file_key(), gains[0]) {
                        play_tone(audio, "sfx_boss_lo", 90.0, 0.55, gains[0]);
                        play_tone(audio, "sfx_boss_hi", 180.0, 0.35, gains[1]);
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    fn test_audio_file_exists(key: &str) -> bool {
        EXTENSIONS.iter().any(|ext| {
            std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../../assets/audio")
                .join(format!("{key}.{ext}"))
                .exists()
        })
    }

    #[test]
    fn sfx_events_have_unique_file_keys() {
        let mut keys = HashSet::new();
        for event in SfxEvent::ALL {
            assert!(keys.insert(event.file_key()), "duplicate key {event:?}");
        }
        assert!(keys.contains("sfx_chest_open"));
        assert!(keys.contains("sfx_boss_appear"));
    }

    #[test]
    fn sfx_placeholder_files_exist_for_release_package() {
        for event in SfxEvent::ALL {
            assert!(
                test_audio_file_exists(event.file_key()),
                "missing SFX asset for key {}",
                event.file_key()
            );
        }
    }

    #[test]
    fn sfx_assets_are_documented_for_release() {
        let licenses = include_str!("../../../../docs/ASSET_LICENSES.md");
        let audio_docs = include_str!("../../../../docs/audio_assets.md");

        for event in SfxEvent::ALL {
            let wav_name = format!("{}.wav", event.file_key());
            assert!(
                licenses.contains(&wav_name),
                "missing {wav_name} from ASSET_LICENSES.md"
            );
            assert!(
                audio_docs.contains(&wav_name),
                "missing {wav_name} from audio_assets.md"
            );
        }
    }

    #[test]
    fn sfx_base_gains_are_valid() {
        for event in SfxEvent::ALL {
            let gains = base_gains(*event);
            assert!(!gains.is_empty(), "missing base gain for {event:?}");
            for &gain in gains {
                assert!(
                    gain > 0.0 && gain <= 1.0,
                    "invalid base gain {gain} for {event:?}"
                );
            }
        }
    }
}
