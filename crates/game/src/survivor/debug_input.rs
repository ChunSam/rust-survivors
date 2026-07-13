//! 시각 검증용 디버그 입력 시스템.
//! H  = 플레이 중 디버그 도움말 표시/숨김
//! F5 = Bomb 픽업 플레이어 앞 스폰
//! F6 = Rosary 픽업 플레이어 앞 스폰
//! B  = GiantSlime 보스 즉시 소환 (F7은 macOS 미디어키 충돌)
//! L  = Level-up 카드 화면 즉시 진입

use super::boss::{spawn_boss, BossKind};
use super::health::Health;
use super::pickup::{spawn_pickup, PickupKind};
use super::player::Player;
use super::xp::XpAccumulator;
use engine::{InputState, KeyCode, System, Transform, World};
use glam::Vec2;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct DebugOverlay {
    pub visible: bool,
}

#[derive(Default)]
pub struct DebugInputSystem;

impl System for DebugInputSystem {
    fn run(&mut self, world: &mut World, _dt: f32) {
        let (help_key, f5, f6, boss_key, level_key) = {
            let Some(input) = world.resource::<InputState>() else {
                return;
            };
            (
                input.just_pressed(KeyCode::KeyH),
                input.just_pressed(KeyCode::F5),
                input.just_pressed(KeyCode::F6),
                input.just_pressed(KeyCode::KeyB),
                input.just_pressed(KeyCode::KeyL),
            )
        };

        if !help_key && !f5 && !f6 && !boss_key && !level_key {
            return;
        }

        if help_key {
            toggle_debug_overlay(world);
        }

        let player_pos = find_player_pos(world);

        if f5 {
            spawn_pickup_near(world, player_pos, PickupKind::Bomb);
        }
        if f6 {
            spawn_pickup_near(world, player_pos, PickupKind::Rosary);
        }
        if boss_key {
            let pos = player_pos + Vec2::new(0.0, -320.0);
            spawn_boss(world, pos, BossKind::GiantSlime);
            // 디버그용 — 즉시 처치되지 않도록 HP를 높게 덮어씀 (query → collect → get_mut 순으로 borrow 분리)
            let boss_entity = world.query::<super::boss::Boss>().next().map(|(e, _)| e);
            if let Some(e) = boss_entity {
                if let Some(h) = world.get_mut::<Health>(e) {
                    h.current = 3000.0;
                    h.max = 3000.0;
                }
            }
            println!("[DEBUG] GiantSlime 보스 즉시 소환 (HP 3000)");
        }
        if level_key && force_next_level_up(world) {
            println!("[DEBUG] Level-up 카드 화면 즉시 진입");
        }
    }
}

fn force_next_level_up(world: &mut World) -> bool {
    let Some(player_entity) = world.query::<Player>().next().map(|(e, _)| e) else {
        return false;
    };
    let Some(xp) = world.get_mut::<XpAccumulator>(player_entity) else {
        return false;
    };

    xp.current = xp.current.max(xp.next_threshold);
    true
}

fn find_player_pos(world: &World) -> Vec2 {
    world
        .query::<Player>()
        .next()
        .and_then(|(e, _)| world.get::<Transform>(e))
        .map(|t| t.position)
        .unwrap_or(Vec2::ZERO)
}

fn spawn_pickup_near(world: &mut World, pos: Vec2, kind: PickupKind) {
    let spawn_pos = pos + Vec2::new(40.0, 0.0);
    spawn_pickup(world, spawn_pos, kind);
    println!("[DEBUG] {kind:?} 픽업 스폰 at {pos:?}");
}

pub fn toggle_debug_overlay(world: &mut World) {
    let next_visible = !world
        .resource::<DebugOverlay>()
        .map(|overlay| overlay.visible)
        .unwrap_or(false);
    world.insert_resource(DebugOverlay {
        visible: next_visible,
    });
    println!(
        "[DEBUG] 테스트 도움말 {}",
        if next_visible { "표시" } else { "숨김" }
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::survivor::pickup::Pickup;
    use crate::survivor::world_setup::spawn_player;
    use engine::{Sprite, UvRect};

    #[test]
    fn debug_overlay_toggles_on_and_off() {
        let mut world = World::new();

        toggle_debug_overlay(&mut world);
        assert_eq!(
            world.resource::<DebugOverlay>(),
            Some(&DebugOverlay { visible: true })
        );

        toggle_debug_overlay(&mut world);
        assert_eq!(
            world.resource::<DebugOverlay>(),
            Some(&DebugOverlay { visible: false })
        );
    }

    #[test]
    fn force_next_level_up_sets_xp_to_threshold() {
        let mut world = World::new();
        spawn_player(&mut world);

        assert!(force_next_level_up(&mut world));

        let player_entity = world.query::<Player>().next().map(|(e, _)| e).unwrap();
        let xp = world.get::<XpAccumulator>(player_entity).unwrap();
        assert_eq!(xp.current, xp.next_threshold);
    }

    #[test]
    fn debug_pickups_use_atlas_sprites() {
        let mut world = World::new();

        spawn_pickup_near(&mut world, Vec2::ZERO, PickupKind::Bomb);
        spawn_pickup_near(&mut world, Vec2::new(100.0, 0.0), PickupKind::Rosary);

        let pickups: Vec<_> = world.query::<Pickup>().collect();
        assert_eq!(pickups.len(), 2);

        for (entity, pickup) in pickups {
            assert!(matches!(pickup.kind, PickupKind::Bomb | PickupKind::Rosary));
            let sprite = world.get::<Sprite>(entity).unwrap();
            assert!(sprite.texture.is_some());
            assert!(world.get::<UvRect>(entity).is_some());
        }
    }
}
