/// Phase 6: 보물상자 컴포넌트 + 픽업 시스템 + 진화 로직.
///
/// 엘리트 적 사망 시 spawn_chest() 로 스폰. 플레이어 pickup_radius 안에 들어오면
/// ChestPickupSystem 이 자동 픽업 후 try_evolve() 를 호출한다.
use engine::{Entity, GameState, System, Transform, World};
use glam::Vec2;

use super::inventory::{WeaponInventory, WeaponKind};
use super::passive::{PassiveInventory, PassiveKind};
use super::player::Player;
use super::sfx::{SfxEvent, SfxQueue};
use super::sprites::{add_sprite, SurvivorSprite};

/// 보물상자 태그 컴포넌트.
#[derive(Debug, Clone, Copy)]
pub struct Chest;

/// Chest 픽업 시스템 — 플레이어와 거리 < pickup_radius 인 Chest 를 자동 픽업.
/// 픽업 시 try_evolve() 로 진화 시도.
pub struct ChestPickupSystem {
    pub pickup_radius: f32,
    chests: Vec<Entity>,
}

impl Default for ChestPickupSystem {
    fn default() -> Self {
        Self {
            pickup_radius: 40.0,
            chests: Vec::new(),
        }
    }
}

impl System for ChestPickupSystem {
    fn run(&mut self, world: &mut World, _dt: f32) {
        if !matches!(world.resource::<GameState>(), Some(GameState::Playing)) {
            return;
        }

        // Player 위치 + entity 캐시
        let Some((player_entity, player_pos)) = world
            .query2::<Player, Transform>()
            .next()
            .map(|(e, _, t)| (e, t.position))
        else {
            return;
        };

        // 픽업 반경 안에 있는 Chest entity 수집
        let pickup_radius_sq = self.pickup_radius * self.pickup_radius;
        self.chests.clear();
        self.chests.extend(
            world
                .query2::<Chest, Transform>()
                .filter(|(_, _, t)| (t.position - player_pos).length_squared() <= pickup_radius_sq)
                .map(|(e, _, _)| e),
        );

        if self.chests.is_empty() {
            return;
        }

        for chest in &self.chests {
            world.despawn(*chest);
        }
        if let Some(q) = world.resource_mut::<SfxQueue>() {
            for _ in 0..self.chests.len() {
                q.push(SfxEvent::ChestOpen);
            }
        }

        // 픽업 개수만큼 진화 시도
        for _ in 0..self.chests.len() {
            try_evolve(world, player_entity);
        }
    }
}

/// 진화 레시피 — 무기 종류 식별자 ↔ 페어 패시브.
#[derive(Debug, Clone, Copy)]
pub struct EvolutionRule {
    pub weapon: &'static str, // WeaponKind 식별자 문자열
    pub passive: PassiveKind,
    pub min_weapon_level: u8,  // 보통 8
    pub min_passive_level: u8, // 보통 5
}

/// 8가지 진화 레시피 테이블.
pub const EVOLUTION_RULES: &[EvolutionRule] = &[
    EvolutionRule {
        weapon: "Whip",
        passive: PassiveKind::HollowHeart,
        min_weapon_level: 8,
        min_passive_level: 5,
    },
    EvolutionRule {
        weapon: "MagicWand",
        passive: PassiveKind::EmptyTome,
        min_weapon_level: 8,
        min_passive_level: 5,
    },
    EvolutionRule {
        weapon: "Knife",
        passive: PassiveKind::Bracer,
        min_weapon_level: 8,
        min_passive_level: 5,
    },
    EvolutionRule {
        weapon: "Axe",
        passive: PassiveKind::Candelabrador,
        min_weapon_level: 8,
        min_passive_level: 5,
    },
    EvolutionRule {
        weapon: "Cross",
        passive: PassiveKind::Clover,
        min_weapon_level: 8,
        min_passive_level: 5,
    },
    EvolutionRule {
        weapon: "FireWand",
        passive: PassiveKind::Spinach,
        min_weapon_level: 8,
        min_passive_level: 5,
    },
    EvolutionRule {
        weapon: "Garlic",
        passive: PassiveKind::Pummarola,
        min_weapon_level: 8,
        min_passive_level: 5,
    },
    EvolutionRule {
        weapon: "HolyWater",
        passive: PassiveKind::Duplicator,
        min_weapon_level: 8,
        min_passive_level: 5,
    },
];

/// 진화 가능한 첫 무기 슬롯을 찾아 evolved=true 표시 + 스탯 강화.
///
/// 반환값: 진화한 무기 이름 식별자 (Some). 진화 불가 시 None.
pub fn try_evolve(world: &mut World, player: Entity) -> Option<&'static str> {
    // 1) PassiveInventory 에서 보유 패시브 레벨 캐시
    //    (borrow 를 즉시 끊어야 이후 get_mut::<WeaponInventory> 와 충돌하지 않음)
    let passive_levels: std::collections::HashMap<PassiveKind, u8> = world
        .get::<PassiveInventory>(player)
        .map(|inv| inv.passives.iter().map(|s| (s.kind, s.level)).collect())
        .unwrap_or_default();

    // 2) WeaponInventory 순회 — 진화 조건 매칭 시 evolved 표시 + 스탯 강화
    let mut evolved_name: Option<&'static str> = None;
    if let Some(inv) = world.get_mut::<WeaponInventory>(player) {
        'outer: for rule in EVOLUTION_RULES {
            let passive_lv = passive_levels.get(&rule.passive).copied().unwrap_or(0);
            if passive_lv < rule.min_passive_level {
                continue;
            }

            for slot in inv.slots.iter_mut() {
                if slot.evolved {
                    continue;
                }
                if slot.level < rule.min_weapon_level {
                    continue;
                }

                let matches_kind = matches_weapon_kind(rule.weapon, &slot.kind);
                if !matches_kind {
                    continue;
                }

                // 진화! cooldown 가속
                slot.evolved = true;
                slot.cooldown *= 0.7;

                // 무기별 스탯 강화
                apply_evolution_stats(&mut slot.kind);

                println!(
                    "EVOLVED: {} (Lv {} + {:?} Lv {})",
                    rule.weapon, slot.level, rule.passive, passive_lv
                );
                evolved_name = Some(rule.weapon);
                break 'outer;
            }
        }
    }

    if evolved_name.is_none() {
        println!("Chest opened — no eligible evolution");
    }
    evolved_name
}

/// rule.weapon 문자열과 WeaponKind variant 매칭.
fn matches_weapon_kind(weapon_name: &str, kind: &WeaponKind) -> bool {
    matches!(
        (weapon_name, kind),
        ("Whip", WeaponKind::Whip { .. })
            | ("MagicWand", WeaponKind::MagicWand { .. })
            | ("Knife", WeaponKind::Knife { .. })
            | ("Axe", WeaponKind::Axe { .. })
            | ("Cross", WeaponKind::Cross { .. })
            | ("FireWand", WeaponKind::FireWand { .. })
            | ("Garlic", WeaponKind::Garlic { .. })
            | ("HolyWater", WeaponKind::HolyWater { .. })
    )
}

/// 진화 스탯 강화 — 각 variant 별 수치 적용.
fn apply_evolution_stats(kind: &mut WeaponKind) {
    match kind {
        WeaponKind::Whip {
            damage,
            area_width,
            area_height,
        } => {
            *damage *= 2.0;
            *area_width *= 1.5;
            *area_height *= 1.5;
        }
        WeaponKind::MagicWand {
            damage,
            projectile_speed,
            lifetime,
            pierce,
        } => {
            *damage *= 2.0;
            *projectile_speed *= 1.3;
            *lifetime *= 1.5;
            *pierce += 2;
        }
        WeaponKind::Knife {
            damage,
            amount,
            pierce,
            ..
        } => {
            *damage *= 2.0;
            *amount += 2;
            *pierce += 1;
        }
        WeaponKind::Axe {
            damage,
            pierce,
            amount,
            ..
        } => {
            *damage *= 2.0;
            *pierce += 3;
            *amount += 1;
        }
        WeaponKind::Cross {
            damage,
            pierce,
            amount,
            ..
        } => {
            *damage *= 2.0;
            *pierce += 2;
            *amount += 1;
        }
        WeaponKind::FireWand { damage, .. } => {
            *damage *= 2.0;
        }
        WeaponKind::Garlic { damage, radius } => {
            *damage *= 2.5;
            *radius *= 1.5;
        }
        WeaponKind::HolyWater {
            damage,
            radius,
            pool_lifetime,
            drop_count,
            ..
        } => {
            *damage *= 2.0;
            *radius *= 1.4;
            *drop_count += 1;
            *pool_lifetime *= 1.5;
        }
        // KingBible, LightningRing — 진화 레시피 미정의. 스킵.
        _ => {}
    }
}

/// 보물상자 엔티티를 월드에 스폰. 황금색 사각형으로 렌더.
pub fn spawn_chest(world: &mut World, pos: Vec2) {
    let e = world.spawn();
    world.add_component(
        e,
        Transform {
            position: pos,
            scale: Vec2::new(28.0, 24.0),
            rotation: 0.0,
            z: 0.4,
        },
    );
    add_sprite(world, e, SurvivorSprite::Chest);
    world.add_component(e, Chest);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::survivor::inventory::{WeaponInventory, WeaponKind};
    use crate::survivor::passive::PassiveInventory;
    use crate::survivor::player::Player;
    use crate::survivor::world_setup::spawn_player;
    use engine::{GameState, World};

    fn make_playing_world_with_player() -> (World, Entity) {
        let mut world = World::new();
        world.insert_resource(GameState::Playing);
        spawn_player(&mut world);
        let player_entity = world
            .query::<Player>()
            .next()
            .map(|(e, _)| e)
            .expect("Player 엔티티가 없음");
        (world, player_entity)
    }

    #[test]
    fn evolution_rules_count_is_eight() {
        assert_eq!(EVOLUTION_RULES.len(), 8, "진화 레시피는 정확히 8개여야 함");
    }

    #[test]
    fn try_evolve_with_full_setup_marks_whip_evolved() {
        let (mut world, player_entity) = make_playing_world_with_player();

        // Whip 슬롯 level 을 8 로 강제 설정
        {
            let inv = world.get_mut::<WeaponInventory>(player_entity).unwrap();
            let whip = inv.whip_slot_mut().expect("Whip 슬롯이 있어야 함");
            whip.level = 8;
        }

        // HollowHeart passive level 5 로 설정
        {
            let inv = world.get_mut::<PassiveInventory>(player_entity).unwrap();
            for _ in 0..5 {
                inv.add_or_levelup(PassiveKind::HollowHeart);
            }
        }

        let result = try_evolve(&mut world, player_entity);
        assert_eq!(result, Some("Whip"), "Whip 진화가 성공해야 함");

        // evolved == true 확인
        let inv = world.get_mut::<WeaponInventory>(player_entity).unwrap();
        let whip = inv.whip_slot().expect("Whip 슬롯이 있어야 함");
        assert!(whip.evolved, "진화 후 evolved 가 true 이어야 함");

        // damage 가 2배 (10.0 * 2 = 20.0) 이상 확인
        if let WeaponKind::Whip { damage, .. } = whip.kind {
            assert!(
                damage >= 20.0,
                "진화 후 Whip damage 가 20.0 이상이어야 함 (실제: {damage})"
            );
        } else {
            panic!("Whip variant 이어야 함");
        }
    }

    #[test]
    fn chest_pickup_despawns_chest() {
        let (mut world, _player_entity) = make_playing_world_with_player();

        // 픽업 반경(40) 안에 상자 스폰
        spawn_chest(&mut world, Vec2::new(20.0, 0.0));

        // 상자가 1개 스폰됐는지 확인
        assert_eq!(
            world.query::<Chest>().count(),
            1,
            "Chest 가 1개 스폰되어야 함"
        );

        // ChestPickupSystem 실행
        ChestPickupSystem::default().run(&mut world, 0.0);

        // 픽업 후 상자가 없어야 함
        assert_eq!(
            world.query::<Chest>().count(),
            0,
            "픽업 후 Chest 가 없어야 함"
        );
    }
}
