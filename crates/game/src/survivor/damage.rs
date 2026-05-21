use engine::{Sprite, Transform, World};
use engine::Entity;

use super::enemy::{Enemy, EnemyKind};
use super::health::Health;
use super::weapon::HitFlash;
use super::xp::spawn_xp_gem;

/// 적 엔티티에 데미지 적용 + 사망 처리 + XpGem 드롭 + HitFlash 부착.
///
/// Slime 사망 시 split_remaining > 0 이면 작은 슬라임 2마리 스폰 (split_remaining = 0 으로 감소).
/// 반환값은 *사망 여부*. 호출자는 반환값으로 killed 카운트 누적.
pub fn apply_damage_to_enemy(world: &mut World, enemy: Entity, damage: f32) -> bool {
    let original = if let Some(s) = world.get_mut::<Sprite>(enemy) {
        let o = s.color;
        s.color = [1.0, 0.3, 0.3, 1.0];
        o
    } else {
        return false;
    };

    let died = if let Some(h) = world.get_mut::<Health>(enemy) {
        h.take_damage(damage)
    } else {
        false
    };

    if died {
        let drop_pos = world.get::<Transform>(enemy).map(|t| t.position);
        let kind     = world.get::<Enemy>(enemy).map(|e| e.kind);
        let splits   = world.get::<Enemy>(enemy).map(|e| e.split_remaining).unwrap_or(0);
        let is_elite = world.get::<Enemy>(enemy).map(|e| e.is_elite).unwrap_or(false);

        world.despawn(enemy);

        if let Some(p) = drop_pos {
            if is_elite {
                // 엘리트 드롭: 큰 XpGem(value 10) + 작은 XpGem 3개 + 보물상자 1개
                spawn_xp_gem(world, p, 10);
                for _ in 0..3 {
                    let offset = Vec2::new(
                        rand::random::<f32>() * 30.0 - 15.0,
                        rand::random::<f32>() * 30.0 - 15.0,
                    );
                    spawn_xp_gem(world, p + offset, 1);
                }
                // Phase 6: 보물상자 드롭 (플레이어가 밟으면 자동 픽업 → 진화 시도)
                crate::survivor::chest::spawn_chest(world, p + Vec2::new(0.0, 30.0));
            } else {
                spawn_xp_gem(world, p, 1);
                // Phase 7: 낮은 확률 픽업 드롭 (Coin/Chicken/Rosary)
                crate::survivor::pickup::try_drop_normal_pickup(world, p);
            }

            // Slime split — split_remaining > 0 이면 자식 2마리 스폰
            // 자식 슬라임은 split_remaining = 0 으로 설정 → 무한 split 방지
            if matches!(kind, Some(EnemyKind::Slime)) && splits > 0 {
                super::spawn::spawn_enemy_with_splits(
                    world,
                    p + Vec2::new(-20.0, 0.0),
                    EnemyKind::Slime,
                    0, // split_remaining = 0 → 더 이상 split 안 함
                );
                super::spawn::spawn_enemy_with_splits(
                    world,
                    p + Vec2::new(20.0, 0.0),
                    EnemyKind::Slime,
                    0,
                );
            }
        }
    } else {
        world.add_component(enemy, HitFlash { remaining: 0.1, original });
    }
    died
}

use glam::Vec2;
