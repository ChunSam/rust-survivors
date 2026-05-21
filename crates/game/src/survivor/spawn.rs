use engine::{Camera, Collider, CollisionLayer, Entity, Sprite, System, Transform, World};
use engine::components::GameState;
use glam::Vec2;
use rand::Rng;
use rand::seq::IteratorRandom;
use super::enemy::{Enemy, EnemyAi, EnemyAiKind, EnemyKind, Zombie};
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

/// 카메라 외곽 원형 경계에서 적을 스폰하고, 너무 멀어진 적을 정리하는 시스템.
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
            let mut rng   = rand::thread_rng();
            let angle     = rng.gen_range(0.0..std::f32::consts::TAU);
            let spawn_pos = center + Vec2::new(angle.cos(), angle.sin()) * self.spawn_radius;
            let kind      = pick_random_enemy_kind(&mut rng);
            spawn_enemy(world, spawn_pos, kind);
        }

        // despawn_radius 밖 적 수집 후 정리
        let to_despawn: Vec<Entity> = world
            .query2::<Enemy, Transform>()
            .filter(|(_, _, t)| (t.position - center).length() > self.despawn_radius)
            .map(|(e, _, _)| e)
            .collect();
        for e in to_despawn {
            world.despawn(e);
        }
    }
}

/// 10종 EnemyKind 중 균등 랜덤 선택.
fn pick_random_enemy_kind(rng: &mut impl Rng) -> EnemyKind {
    *[
        EnemyKind::Zombie,
        EnemyKind::Bat,
        EnemyKind::Ghost,
        EnemyKind::Skeleton,
        EnemyKind::Mage,
        EnemyKind::Mantis,
        EnemyKind::Plant,
        EnemyKind::Slime,
        EnemyKind::Mummy,
        EnemyKind::Knight,
    ]
    .iter()
    .choose(rng)
    .unwrap()
}

/// 지정 위치에 EnemyKind 에 맞는 적 엔티티를 스폰한다.
///
/// Slime 은 split_remaining = 2 로 초기화.
pub fn spawn_enemy(world: &mut World, pos: Vec2, kind: EnemyKind) {
    // Slime 기본 split 횟수 = 2
    let splits = if matches!(kind, EnemyKind::Slime) { 2 } else { 0 };
    spawn_enemy_with_splits(world, pos, kind, splits);
}

/// split_remaining 을 직접 지정해 스폰 (Slime split child 스폰에 사용).
pub fn spawn_enemy_with_splits(world: &mut World, pos: Vec2, kind: EnemyKind, split_remaining: u8) {
    let stats = kind.stats();

    let ai_kind = match kind {
        EnemyKind::Ghost  => EnemyAiKind::Hover { stop_at: 80.0 },
        EnemyKind::Mage   => EnemyAiKind::Kite  { min_dist: 200.0 },
        EnemyKind::Mantis => EnemyAiKind::Dash {
            cooldown:      2.5,
            elapsed:       0.0,
            dash_speed:    350.0,
            dash_lifetime: 0.4,
            dashing:       0.0,
        },
        EnemyKind::Plant  => EnemyAiKind::Stay,
        EnemyKind::Slime  => EnemyAiKind::Split,
        _ => EnemyAiKind::Chase,
    };

    let e = world.spawn();
    world.add_component(e, Transform {
        position: pos,
        scale:    Vec2::new(stats.scale, stats.scale),
        rotation: 0.0,
        z:        0.5,
    });
    world.add_component(e, Sprite::colored(stats.color[0], stats.color[1], stats.color[2]));
    world.add_component(e, Enemy {
        kind,
        contact_damage: stats.contact_damage,
        split_remaining,
    });
    world.add_component(e, EnemyAi { move_speed: stats.move_speed, ai: ai_kind });
    world.add_component(e, Health::new(stats.hp));
    world.add_component(e, CollisionLayer(LAYER_ENEMY));
    world.add_component(e, Collider::Circle { radius: stats.scale / 2.0 });

    // 하위 호환: Zombie 종류에는 Zombie 태그도 부착 (기존 코드 호환)
    if matches!(kind, EnemyKind::Zombie) {
        world.add_component(e, Zombie);
    }
}

/// 좀비 엔티티를 주어진 위치에 스폰한다 (하위 호환 유지).
///
/// 내부적으로 `spawn_enemy(world, pos, EnemyKind::Zombie)` 를 호출한다.
pub fn spawn_zombie(world: &mut World, pos: Vec2) {
    spawn_enemy(world, pos, EnemyKind::Zombie);
}
