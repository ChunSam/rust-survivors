use engine::{Camera, Collider, CollisionLayer, Entity, Sprite, System, Transform, World};
use engine::components::GameState;
use glam::Vec2;
use rand::Rng;
use super::enemy::{EnemyAi, Zombie};
use super::health::Health;
use super::LAYER_ENEMY;

/// 적 스폰 주기 타이머 (ECS 리소스)
pub struct SpawnTimer {
    pub interval: f32,
    pub elapsed:  f32,
}

impl Default for SpawnTimer {
    fn default() -> Self {
        Self { interval: 1.0, elapsed: 0.0 }
    }
}

/// 카메라 외곽 원형 경계에서 좀비를 스폰하고, 너무 멀어진 좀비를 정리하는 시스템.
///
/// `viewport` 는 Camera 리소스에 없으므로 CameraFollowSystem 과 동일하게 필드로 보관한다.
pub struct EnemySpawnSystem {
    pub spawn_radius:   f32,
    pub despawn_radius: f32,
    pub viewport:       Vec2,
}

impl Default for EnemySpawnSystem {
    fn default() -> Self {
        Self {
            spawn_radius:   500.0,
            despawn_radius: 900.0,
            viewport:       Vec2::new(800.0, 600.0),
        }
    }
}

impl System for EnemySpawnSystem {
    fn run(&mut self, world: &mut World, dt: f32) {
        if !matches!(world.resource::<GameState>(), Some(GameState::Playing)) {
            return;
        }

        // 카메라 좌상단 + 뷰포트 절반 = 화면 중심
        let camera_pos = world
            .resource::<Camera>()
            .map(|c| c.position)
            .unwrap_or(Vec2::ZERO);
        let center = camera_pos + self.viewport * 0.5;

        // 타이머 갱신 — resource_mut borrow 를 블록 안에서 즉시 종료
        let should_spawn = {
            let timer = world.resource_mut::<SpawnTimer>().unwrap();
            timer.elapsed += dt;
            if timer.elapsed >= timer.interval {
                timer.elapsed -= timer.interval;
                true
            } else {
                false
            }
        };

        if should_spawn {
            let angle = rand::thread_rng().gen_range(0.0..std::f32::consts::TAU);
            let spawn_pos = center + Vec2::new(angle.cos(), angle.sin()) * self.spawn_radius;
            spawn_zombie(world, spawn_pos);
        }

        // despawn_radius 밖 좀비 수집 후 정리
        let to_despawn: Vec<Entity> = world
            .query2::<Zombie, Transform>()
            .filter(|(_, _, t)| (t.position - center).length() > self.despawn_radius)
            .map(|(e, _, _)| e)
            .collect();
        for e in to_despawn {
            world.despawn(e);
        }
    }
}

/// 좀비 엔티티를 주어진 위치에 스폰한다.
pub fn spawn_zombie(world: &mut World, pos: Vec2) {
    let e = world.spawn();
    world.add_component(e, Transform {
        position: pos,
        scale:    Vec2::new(40.0, 40.0),
        rotation: 0.0,
        z:        0.5,
    });
    world.add_component(e, Sprite::colored(0.4, 0.7, 0.4));
    world.add_component(e, Zombie);
    world.add_component(e, EnemyAi::default());
    world.add_component(e, CollisionLayer(LAYER_ENEMY));
    world.add_component(e, Collider::Circle { radius: 20.0 });
    world.add_component(e, Health::new(30.0));
}
