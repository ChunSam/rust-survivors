//! PlayerStats 재계산 시스템 및 헬퍼.
//!
//! Phase 3-A: StatRecalcSystem 은 stub (no-op). 패시브가 없으므로 default 값 그대로.
//! Phase 3-B 에서 PassiveInventory 합산 로직이 여기에 추가된다.

use engine::{System, World};

use super::player::{Player, PlayerStats};

/// PlayerStats 를 매 프레임 재계산.
///
/// Phase 3-A: stub — 아직 패시브가 없으므로 default 값 그대로. 변경 없음.
/// Phase 3-B 에서 패시브 합산 로직 추가.
pub struct StatRecalcSystem;

impl System for StatRecalcSystem {
    fn run(&mut self, _world: &mut World, _dt: f32) {
        // 현재는 no-op. 패시브 도입 시 여기서 합산.
    }
}

/// World 에서 Player 의 PlayerStats 를 읽어 복사본을 반환.
///
/// 각 무기 시스템이 매 프레임 시작에 호출하여 stats 캐시를 확보한다.
/// `&World` 불변 빌림만 사용하므로 이후 `get_mut` 호출과 안전하게 공존.
pub fn read_player_stats(world: &World) -> PlayerStats {
    world
        .query2::<Player, PlayerStats>()
        .next()
        .map(|(_, _, s)| s.clone())
        .unwrap_or_default()
}
