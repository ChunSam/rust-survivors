#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Health {
    pub current: f32,
    pub max: f32,
}

impl Health {
    pub fn new(max: f32) -> Self {
        Self { current: max, max }
    }

    /// 데미지를 차감. 반환값: 차감 후 사망(<= 0) 여부.
    pub fn take_damage(&mut self, amount: f32) -> bool {
        self.current -= amount;
        self.current <= 0.0
    }
}

// ─── HealthRegenSystem ────────────────────────────────────────────────────────

use super::player::{Player, PlayerStats};
use engine::components::GameState;
use engine::{Entity, System, World};

/// 매 프레임 PlayerStats.recovery 만큼 Player HP 를 자연 회복.
///
/// recovery = 0.0 (default) 이면 변화 없음.
/// Playing 상태에서만 동작. HP 는 max 를 초과하지 않음.
pub struct HealthRegenSystem;

impl System for HealthRegenSystem {
    fn run(&mut self, world: &mut World, dt: f32) {
        if !matches!(world.resource::<GameState>(), Some(GameState::Playing)) {
            return;
        }

        // Player + PlayerStats + Health 를 가진 엔티티 수집 (borrow 즉시 종료)
        let targets: Vec<(Entity, f32, f32)> = world
            .query2::<Player, PlayerStats>()
            .filter_map(|(e, _, stats)| {
                if stats.recovery > 0.0 {
                    Some((e, stats.recovery, stats.max_health))
                } else {
                    None
                }
            })
            .collect();

        for (entity, recovery, max_health) in targets {
            if let Some(h) = world.get_mut::<Health>(entity) {
                h.current = (h.current + recovery * dt).min(max_health);
            }
        }
    }
}
