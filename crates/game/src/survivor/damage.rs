use engine::Entity;
use engine::{Sprite, Transform, World};

use super::damage_number::spawn_damage_number;
use super::enemy::{Enemy, EnemyKind};
use super::health::Health;
use super::particle::spawn_death_burst;
use super::sfx::{SfxEvent, SfxQueue};
use super::weapon::{HitFlash, FLASH_DURATION, SCALE_BUMP};
use super::xp::spawn_xp_gem;

/// 적 엔티티에 데미지 적용 + 사망 처리 + XpGem 드롭 + HitFlash 부착.
///
/// Slime 사망 시 split_remaining > 0 이면 작은 슬라임 2마리 스폰 (split_remaining = 0 으로 감소).
/// 반환값은 *사망 여부*. 호출자는 반환값으로 killed 카운트 누적.
pub fn apply_damage_to_enemy(world: &mut World, enemy: Entity, damage: f32) -> bool {
    // 읽기 패스 — 불변 borrow 를 값으로 복사한 뒤 해제 (가변 borrow 충돌 방지)
    let maybe_flash = world
        .get::<HitFlash>(enemy)
        .map(|f| (f.original, f.original_scale));
    let sprite_color = world.get::<Sprite>(enemy).map(|s| s.color);
    let orig_scale = world.get::<Transform>(enemy).map(|t| t.scale);

    // Sprite 없으면 유효하지 않은 엔티티
    let current_color = match sprite_color {
        Some(c) => c,
        None => return false,
    };

    // 재피격 시 원본 색·스케일 누적 방지: 이미 플래시 중이면 원본 값 보존
    let (original, original_scale) =
        maybe_flash.unwrap_or((current_color, orig_scale.unwrap_or(Vec2::ONE)));

    // 쓰기 패스 — 흰색 순간 플래시 + 스케일 bump
    if let Some(s) = world.get_mut::<Sprite>(enemy) {
        s.color = [1.0, 1.0, 1.0, original[3]];
    }
    if let Some(t) = world.get_mut::<Transform>(enemy) {
        t.scale = original_scale * SCALE_BUMP;
    }

    // 데미지 숫자 스폰 — 적 위치를 먼저 캐시한 뒤 spawn 호출
    // (Health borrow 와 World mut 가 겹치지 않도록 순서 분리)
    let hit_pos = world.get::<Transform>(enemy).map(|t| t.position);

    let died = if let Some(h) = world.get_mut::<Health>(enemy) {
        h.take_damage(damage)
    } else {
        false
    };

    if let Some(p) = hit_pos {
        spawn_damage_number(world, p, damage);
    }

    // SFX 이벤트 push
    if let Some(q) = world.resource_mut::<SfxQueue>() {
        q.push(if died {
            SfxEvent::EnemyDie
        } else {
            SfxEvent::EnemyHit
        });
    }

    if died {
        let drop_pos = world.get::<Transform>(enemy).map(|t| t.position);
        let enemy_color = original; // 피격 전 원래 색 = 적 고유 색
        let kind = world.get::<Enemy>(enemy).map(|e| e.kind);
        let splits = world
            .get::<Enemy>(enemy)
            .map(|e| e.split_remaining)
            .unwrap_or(0);
        let is_elite = world
            .get::<Enemy>(enemy)
            .map(|e| e.is_elite)
            .unwrap_or(false);

        world.despawn(enemy);

        // 사망 파티클 (despawn 후 스폰)
        if let Some(p) = drop_pos {
            spawn_death_burst(
                world,
                p,
                [enemy_color[0], enemy_color[1], enemy_color[2], 1.0],
            );
        }

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
        world.add_component(
            enemy,
            HitFlash {
                remaining: FLASH_DURATION,
                duration: FLASH_DURATION,
                original,
                original_scale,
            },
        );
    }
    died
}

use glam::Vec2;
