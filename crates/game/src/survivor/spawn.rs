use super::enemy::{Enemy, EnemyAi, EnemyAiKind, EnemyKind, Zombie};
use super::health::Health;
use super::sprites::{add_sprite, add_tinted_sprite, SurvivorSprite, ENEMY_VISUAL_SCALE};
use super::LAYER_ENEMY;
use engine::{Collider, CollisionLayer, Transform, World};
use glam::Vec2;

/// 지정 위치에 EnemyKind 에 맞는 적 엔티티를 스폰한다.
///
/// Slime 은 split_remaining = 2 로 초기화.
pub fn spawn_enemy(world: &mut World, pos: Vec2, kind: EnemyKind) {
    spawn_enemy_elite(world, pos, kind, false);
}

/// 엘리트 여부를 지정해 스폰. 엘리트는 HP×5 / scale×1.5 / contact_damage×1.5 + 노란빛 색.
pub fn spawn_enemy_elite(world: &mut World, pos: Vec2, kind: EnemyKind, is_elite: bool) {
    let splits = if matches!(kind, EnemyKind::Slime) {
        2
    } else {
        0
    };
    spawn_enemy_full(world, pos, kind, splits, is_elite);
}

/// split_remaining 을 직접 지정해 스폰 (Slime split child 스폰에 사용).
pub fn spawn_enemy_with_splits(world: &mut World, pos: Vec2, kind: EnemyKind, split_remaining: u8) {
    spawn_enemy_full(world, pos, kind, split_remaining, false);
}

/// 모든 옵션을 받는 내부 헬퍼.
pub fn spawn_enemy_full(
    world: &mut World,
    pos: Vec2,
    kind: EnemyKind,
    split_remaining: u8,
    is_elite: bool,
) {
    let mut stats = kind.stats();
    let mut collision_radius = stats.scale * 0.5;
    stats.scale *= ENEMY_VISUAL_SCALE;
    if is_elite {
        stats.hp *= 5.0;
        stats.scale *= 1.5;
        collision_radius *= 1.5;
        stats.color = [
            (stats.color[0] * 1.2).clamp(0.0, 1.0),
            (stats.color[1] * 0.9).clamp(0.0, 1.0),
            (stats.color[2] * 0.6).clamp(0.0, 1.0),
        ];
        stats.contact_damage *= 1.5;
    }

    let ai_kind = match kind {
        EnemyKind::Ghost => EnemyAiKind::Hover { stop_at: 80.0 },
        EnemyKind::Mage => EnemyAiKind::Kite { min_dist: 200.0 },
        EnemyKind::Mantis => EnemyAiKind::Dash {
            cooldown: 2.5,
            elapsed: 0.0,
            dash_speed: 350.0,
            dash_lifetime: 0.4,
            dashing: 0.0,
        },
        EnemyKind::Plant => EnemyAiKind::Stay,
        EnemyKind::Slime => EnemyAiKind::Split,
        _ => EnemyAiKind::Chase,
    };

    let e = world.spawn();
    world.add_component(
        e,
        Transform {
            position: pos,
            scale: Vec2::new(stats.scale, stats.scale),
            rotation: 0.0,
            z: 0.5,
        },
    );
    let sprite = match kind {
        EnemyKind::Zombie => SurvivorSprite::Zombie,
        EnemyKind::Bat => SurvivorSprite::Bat,
        EnemyKind::Ghost => SurvivorSprite::Ghost,
        EnemyKind::Skeleton => SurvivorSprite::Skeleton,
        EnemyKind::Mage => SurvivorSprite::Mage,
        EnemyKind::Mantis => SurvivorSprite::Mantis,
        EnemyKind::Plant => SurvivorSprite::Plant,
        EnemyKind::Slime => SurvivorSprite::Slime,
        EnemyKind::Mummy => SurvivorSprite::Mummy,
        EnemyKind::Knight => SurvivorSprite::Knight,
    };
    if is_elite {
        add_tinted_sprite(world, e, sprite, [1.25, 1.05, 0.55, 1.0]);
    } else {
        add_sprite(world, e, sprite);
    }
    world.add_component(
        e,
        Enemy {
            kind,
            contact_damage: stats.contact_damage,
            split_remaining,
            is_elite,
        },
    );
    world.add_component(
        e,
        EnemyAi {
            move_speed: stats.move_speed,
            ai: ai_kind,
        },
    );
    world.add_component(e, Health::new(stats.hp));
    world.add_component(e, CollisionLayer(LAYER_ENEMY));
    world.add_component(
        e,
        Collider::Circle {
            radius: collision_radius,
        },
    );

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn enemy_collider_uses_gameplay_size_not_visual_scale() {
        let mut world = World::new();
        spawn_enemy(&mut world, Vec2::ZERO, EnemyKind::Zombie);

        let (_, transform, collider) = world
            .query2::<Transform, Collider>()
            .next()
            .expect("enemy should have transform and collider");
        let Collider::Circle { radius } = collider else {
            panic!("enemy collider should be a circle");
        };

        assert!(
            *radius < transform.scale.x * 0.25,
            "contact collider should not grow with enlarged render scale"
        );
        assert_eq!(*radius, EnemyKind::Zombie.stats().scale * 0.5);
    }

    #[test]
    fn elite_enemy_collider_scales_only_by_elite_multiplier() {
        let mut world = World::new();
        spawn_enemy_elite(&mut world, Vec2::ZERO, EnemyKind::Zombie, true);

        let (_, _, collider) = world
            .query2::<Transform, Collider>()
            .next()
            .expect("elite enemy should have transform and collider");
        let Collider::Circle { radius } = collider else {
            panic!("enemy collider should be a circle");
        };

        assert_eq!(*radius, EnemyKind::Zombie.stats().scale * 0.5 * 1.5);
    }
}
