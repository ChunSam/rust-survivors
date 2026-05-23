//! 시각 검증용 디버그 입력 시스템.
//! F5 = Bomb 픽업 플레이어 앞 스폰
//! F6 = Rosary 픽업 플레이어 앞 스폰
//! B  = GiantSlime 보스 즉시 소환 (F7은 macOS 미디어키 충돌)

use super::boss::{spawn_boss, BossKind};
use super::health::Health;
use super::pickup::{Pickup, PickupKind};
use super::player::Player;
use engine::{Entity, InputState, Sprite, System, Transform, World};
use glam::Vec2;
use winit::keyboard::KeyCode;

#[derive(Default)]
pub struct DebugInputSystem;

impl System for DebugInputSystem {
    fn run(&mut self, world: &mut World, _dt: f32) {
        let (f5, f6, boss_key) = {
            let Some(input) = world.resource::<InputState>() else {
                return;
            };
            (
                input.just_pressed(KeyCode::F5),
                input.just_pressed(KeyCode::F6),
                input.just_pressed(KeyCode::KeyB),
            )
        };

        if !f5 && !f6 && !boss_key {
            return;
        }

        let player_pos = find_player_pos(world);

        if f5 {
            spawn_pickup_near(world, player_pos, PickupKind::Bomb);
        }
        if f6 {
            spawn_pickup_near(world, player_pos, PickupKind::Rosary);
        }
        if boss_key {
            let pos = player_pos + Vec2::new(0.0, -200.0);
            spawn_boss(world, pos, BossKind::GiantSlime);
            // 디버그용 — 즉시 처치되지 않도록 HP를 높게 덮어씀 (query → collect → get_mut 순으로 borrow 분리)
            let boss_entity = world.query::<super::boss::Boss>().next().map(|(e, _)| e);
            if let Some(e) = boss_entity {
                if let Some(h) = world.get_mut::<Health>(e) {
                    h.current = 3000.0;
                    h.max = 3000.0;
                }
            }
            println!("[DEBUG] GiantSlime 보스 즉시 소환 (HP 99999)");
        }
    }
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
    let color = kind.color();
    let size = kind.scale();
    let e: Entity = world.spawn();
    world.add_component(
        e,
        Transform {
            position: pos + Vec2::new(40.0, 0.0),
            scale: Vec2::splat(size),
            rotation: 0.0,
            z: 0.5,
        },
    );
    world.add_component(e, Sprite::colored(color[0], color[1], color[2]));
    world.add_component(e, Pickup { kind });
    println!("[DEBUG] {:?} 픽업 스폰 at {:?}", kind, pos);
}
