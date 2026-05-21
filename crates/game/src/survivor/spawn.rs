use engine::{Collider, CollisionLayer, Sprite, Transform, World};
use glam::Vec2;
use super::enemy::{Enemy, EnemyAi, EnemyAiKind, EnemyKind, Zombie};
use super::health::Health;
use super::LAYER_ENEMY;

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
