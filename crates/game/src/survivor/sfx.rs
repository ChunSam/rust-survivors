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

use engine::{System, World};
use engine::audio::AudioManager;

/// SFX 이벤트 종류.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SfxEvent {
    EnemyHit,    // 적 피격 (단발 틱음)
    EnemyDie,    // 적 사망 (짧은 저음)
    PlayerHit,   // 플레이어 피격 (중간 충격음)
    LevelUp,     // 레벨업 (상승 화음)
    XpGem,       // 경험치 젬 수집 (고음 핑)
    Pickup,      // 일반 픽업 (밝은 핑)
    Bomb,        // 폭탄 폭발 (저주파 굉음)
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
pub struct SfxSystem;

impl System for SfxSystem {
    fn run(&mut self, world: &mut World, _dt: f32) {
        // 1) 큐 drain — AudioManager 와의 이중 borrow 를 피하기 위해 먼저 빼낸다
        let events: Vec<SfxEvent> = if let Some(q) = world.resource_mut::<SfxQueue>() {
            q.events.drain(..).collect()
        } else {
            return;
        };
        if events.is_empty() { return; }

        let Some(audio) = world.resource_mut::<AudioManager>() else { return };

        // 2) 카테고리별 프레임 상한으로 스팸 방지
        let mut hit_n  = 0u8;
        let mut die_n  = 0u8;
        let mut xp_n   = 0u8;

        for event in events {
            match event {
                SfxEvent::EnemyHit => {
                    if hit_n < 2 {
                        // 짧은 고음 틱 — 여러 무기가 동시에 때려도 2회로 제한
                        let ch = format!("sfx_hit_{hit_n}");
                        audio.play_tone(&ch, 420.0, 0.045, 0.22);
                        hit_n += 1;
                    }
                }
                SfxEvent::EnemyDie => {
                    if die_n < 2 {
                        let ch = format!("sfx_die_{die_n}");
                        audio.play_tone(&ch, 160.0, 0.13, 0.35);
                        die_n += 1;
                    }
                }
                SfxEvent::PlayerHit => {
                    // 플레이어 피격은 게임당 cooldown 이 있어 많아야 1회/초
                    audio.play_tone("sfx_player_hit", 200.0, 0.12, 0.50);
                }
                SfxEvent::LevelUp => {
                    // 상승하는 두 음 — 두 번째 톤으로 즉시 덮어쓰지 않도록 다른 채널
                    audio.play_tone("sfx_lvl_lo", 440.0, 0.18, 0.55);
                    audio.play_tone("sfx_lvl_hi", 660.0, 0.25, 0.55);
                }
                SfxEvent::XpGem => {
                    if xp_n < 3 {
                        let ch = format!("sfx_xp_{xp_n}");
                        audio.play_tone(&ch, 880.0, 0.04, 0.12);
                        xp_n += 1;
                    }
                }
                SfxEvent::Pickup => {
                    audio.play_tone("sfx_pickup", 1040.0, 0.09, 0.40);
                }
                SfxEvent::Bomb => {
                    // 저주파 굉음 + 하모닉
                    audio.play_tone("sfx_bomb_lo", 60.0,  0.45, 0.70);
                    audio.play_tone("sfx_bomb_hi", 120.0, 0.30, 0.45);
                }
            }
        }
    }
}
