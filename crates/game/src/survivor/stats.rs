//! PlayerStats 재계산 시스템 및 헬퍼.
//!
//! Phase 3-A: StatRecalcSystem 은 stub (no-op). 패시브가 없으므로 default 값 그대로.
//! Phase 3-B: PassiveInventory 합산 로직 추가.
//!   1) default 로 시작
//!   2) 각 패시브 효과 누적
//!   3) PlayerStats 컴포넌트에 write
//! Phase 8-B: PowerUp 효과 합산 추가.
//!   패시브 합산 후 MetaSave.powerup_levels 기반으로 apply_powerups 호출.

use engine::{System, World};

use super::character::SelectedCharacter;
use super::meta::MetaSave;
use super::passive::{PassiveInventory, PassiveKind};
use super::player::{Player, PlayerStats};

/// PlayerStats 를 매 프레임 재계산.
///
/// Phase 3-B: PassiveInventory 를 읽어 PlayerStats 를 합산·갱신한다.
pub struct StatRecalcSystem;

impl System for StatRecalcSystem {
    fn run(&mut self, world: &mut World, _dt: f32) {
        // Player + PlayerStats + PassiveInventory 를 가진 엔티티 조회.
        // borrow 를 즉시 끊기 위해 (entity, inv.clone()) 만 캐시한다.
        let info = world
            .query3::<Player, PlayerStats, PassiveInventory>()
            .next()
            .map(|(e, _, _, inv)| (e, inv.clone()));
        let Some((player_entity, inv)) = info else {
            return;
        };

        // 1) default 로 시작
        let mut s = PlayerStats::default();

        // 2) 각 패시브 효과 누적
        for slot in &inv.passives {
            let lv = slot.level as f32;
            match slot.kind {
                PassiveKind::Spinach => s.might += lv,
                PassiveKind::Armor => s.armor += lv,
                PassiveKind::HollowHeart => s.max_health += 20.0 * lv,
                PassiveKind::Pummarola => s.recovery += 0.5 * lv,
                PassiveKind::EmptyTome => s.cooldown *= 0.92_f32.powf(lv),
                PassiveKind::Candelabrador => s.area *= 1.10_f32.powf(lv),
                PassiveKind::Bracer => s.projectile_speed *= 1.10_f32.powf(lv),
                PassiveKind::Spellbinder => s.duration *= 1.10_f32.powf(lv),
                PassiveKind::Duplicator => s.amount += lv as i32,
                PassiveKind::Wings => s.move_speed *= 1.10_f32.powf(lv),
                PassiveKind::Attractorb => s.magnet *= 1.25_f32.powf(lv),
                PassiveKind::Clover => s.luck *= 1.10_f32.powf(lv),
                PassiveKind::Crown => s.growth *= 1.08_f32.powf(lv),
                PassiveKind::StoneMask => s.greed *= 1.10_f32.powf(lv),
                PassiveKind::SkullOManiac => s.curse *= 1.10_f32.powf(lv),
                PassiveKind::Tiragisu => s.revival += lv as u32,
            }
        }

        // 3) PowerUp 효과 누적 (Phase 8-B)
        // MetaSave 를 clone 해서 borrow 분리 — query3 로 얻은 world 빌림과 충돌 방지.
        let meta_clone = world.resource::<MetaSave>().cloned();
        if let Some(meta) = meta_clone {
            super::powerup::apply_powerups(&mut s, &meta);
        }

        // 4) 캐릭터 스탯 보정 (Phase 9) — 패시브/파워업 합산 위에 추가 적용.
        let selected = world
            .resource::<SelectedCharacter>()
            .copied()
            .unwrap_or_default();
        selected.0.apply_stats(&mut s);

        // 5) PlayerStats 컴포넌트에 write
        if let Some(stats) = world.get_mut::<PlayerStats>(player_entity) {
            *stats = s;
        }
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::survivor::passive::PassiveKind;
    use crate::survivor::player::Player;
    use crate::survivor::world_setup::spawn_player;
    use engine::World;

    #[test]
    fn stat_recalc_applies_spinach_might() {
        let mut world = World::new();
        world.insert_resource(engine::components::GameState::Playing);
        spawn_player(&mut world);

        let player_entity = world
            .query::<Player>()
            .next()
            .map(|(e, _)| e)
            .expect("Player 엔티티가 없음");

        // Spinach 3레벨
        if let Some(inv) = world.get_mut::<PassiveInventory>(player_entity) {
            inv.add_or_levelup(PassiveKind::Spinach);
            inv.add_or_levelup(PassiveKind::Spinach);
            inv.add_or_levelup(PassiveKind::Spinach);
        }

        StatRecalcSystem.run(&mut world, 0.0);

        let stats = world.get_mut::<PlayerStats>(player_entity).unwrap();
        assert!(
            (stats.might - 3.0).abs() < 0.001,
            "Spinach 3레벨 적용 후 might == 3.0 이어야 함 (현재 {})",
            stats.might
        );
    }
}
