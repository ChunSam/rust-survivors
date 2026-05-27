use engine::{
    AnimationPlayer, CollisionLayer, Entity, GameState, SpatialGrid, Sprite, System, Transform,
    World,
};
use glam::Vec2;
use rand::seq::SliceRandom;

use super::damage::apply_damage_to_enemy;
use super::hud::GameStats;
use super::inventory::{WeaponInventory, WeaponKind};
use super::lightning::spawn_timed_effect;
use super::player::Player;
use super::projectile::{spawn_projectile, spawn_projectile_ex, ProjectileBehavior};
use super::sprites::SurvivorSprite;
use super::stats::read_player_stats;
use super::LAYER_ENEMY;

/// 히트 플래시 총 지속 시간(초). 흰색 → 원래 색으로 페이드하는 구간.
pub const FLASH_DURATION: f32 = 0.14;
/// 피격 순간 스케일 배율. 1.0 → SCALE_BUMP 로 순간 확대 후 빠르게 복귀.
pub const SCALE_BUMP: f32 = 1.18;

/// 히트플래시 컴포넌트. 피격된 엔티티에 일시적으로 부착.
///
/// `remaining > 0`: 플래시 진행 중.
/// Phase 11-B: 흰색 순간 플래시 → 원래 색 페이드 + 스케일 펄스.
/// 최신 엔진의 `World::remove_component` 로 만료 시 컴포넌트를 제거한다.
pub struct HitFlash {
    pub remaining: f32,       // 남은 시간
    pub duration: f32,        // 총 지속 시간 (lerp 기준)
    pub original: [f32; 4],   // 원래 스프라이트 색상
    pub original_scale: Vec2, // 원래 Transform 스케일
}

/// 히트플래시 갱신 시스템. WhipSystem 다음에 등록.
pub struct HitFlashSystem;

impl System for HitFlashSystem {
    fn run(&mut self, world: &mut World, dt: f32) {
        // 두 단계 패턴: 먼저 수집(불변) → 그 다음 get_mut(가변)
        let flashes: Vec<(Entity, f32, f32, [f32; 4], Vec2)> = world
            .query::<HitFlash>()
            .map(|(e, f)| (e, f.remaining, f.duration, f.original, f.original_scale))
            .collect();

        for (entity, remaining, duration, original, original_scale) in flashes {
            let new_remaining = remaining - dt;
            if new_remaining <= 0.0 {
                // 만료 — 색·스케일 원복 후 컴포넌트 제거
                if let Some(s) = world.get_mut::<Sprite>(entity) {
                    s.color = original;
                }
                if let Some(player) = world.get_mut::<AnimationPlayer>(entity) {
                    player.play(0);
                }
                if let Some(t) = world.get_mut::<Transform>(entity) {
                    t.scale = original_scale;
                }
                world.remove_component::<HitFlash>(entity);
            } else {
                // t_norm: 1.0(방금 피격) → 0.0(만료 직전)
                let t_norm = (new_remaining / duration).clamp(0.0, 1.0);

                if let Some(player) = world.get_mut::<AnimationPlayer>(entity) {
                    player.play(1);
                }

                // 색상: 흰색 → 원래 색 페이드
                if let Some(s) = world.get_mut::<Sprite>(entity) {
                    s.color = [
                        original[0] + (1.0 - original[0]) * t_norm,
                        original[1] + (1.0 - original[1]) * t_norm,
                        original[2] + (1.0 - original[2]) * t_norm,
                        original[3],
                    ];
                }
                // 스케일 펄스: t_norm 0.6~1.0 구간에서만 bump, 이후 즉시 원복
                let scale_p = ((t_norm - 0.6) / 0.4).clamp(0.0, 1.0);
                if let Some(t) = world.get_mut::<Transform>(entity) {
                    t.scale = original_scale.lerp(original_scale * SCALE_BUMP, scale_p);
                }
                if let Some(f) = world.get_mut::<HitFlash>(entity) {
                    f.remaining = new_remaining;
                }
            }
        }
    }
}

/// Whip 발화 시스템. SpatialGrid 를 직접 소유 (PhysicsSystem 패턴).
///
/// Phase 2-A: cooldown/elapsed 트래킹이 WhipSystem.elapsed 에서
/// WeaponInventory 슬롯의 elapsed 로 이전됨.
pub struct WhipSystem {
    pub grid: SpatialGrid,
}

impl Default for WhipSystem {
    fn default() -> Self {
        Self {
            grid: SpatialGrid::new(128.0),
        }
    }
}

impl System for WhipSystem {
    fn run(&mut self, world: &mut World, dt: f32) {
        if !matches!(world.resource::<GameState>(), Some(GameState::Playing)) {
            return;
        }

        // 1) Player 위치 캐시 (borrow 를 즉시 끊음)
        let player_e_and_pos = world
            .query2::<Player, Transform>()
            .next()
            .map(|(e, _, t)| (e, t.position));
        let Some((player_entity, player_pos)) = player_e_and_pos else {
            return;
        };

        // 2) stats 캐시 (불변 빌림만 → 이후 get_mut 과 충돌 없음)
        let stats = read_player_stats(world);

        // 3) WeaponInventory 의 Whip 슬롯 tick + 파라미터 복사
        //    (get_mut borrow 를 블록 안에서 끊어야 이후 world 접근 가능)
        let fire_info: Option<(f32, f32, f32)> = {
            let Some(inv) = world.get_mut::<WeaponInventory>(player_entity) else {
                return;
            };
            let Some(slot) = inv.whip_slot_mut() else {
                return;
            };
            if !slot.tick_with_cooldown_multiplier(dt, stats.cooldown) {
                return;
            }
            // 발화. 파라미터를 값으로 복사해 반환 (borrow 해제).
            #[allow(irrefutable_let_patterns)]
            if let WeaponKind::Whip {
                damage,
                area_width,
                area_height,
            } = slot.kind
            {
                Some((damage, area_width, area_height))
            } else {
                None
            }
        };
        let Some((base_damage, area_width, area_height)) = fire_info else {
            return;
        };

        // stats 적용
        let damage = base_damage + stats.might;
        let area_width = area_width * stats.area;
        let area_height = area_height * stats.area;

        // 4) 발화 시점에만 grid 갱신 — 매 프레임 rebuild 비용 절감
        self.grid.rebuild(world);

        let half_h = area_height * 0.5;
        let mask = CollisionLayer(LAYER_ENEMY);

        // 좌측 AABB
        let left_min = player_pos + Vec2::new(-area_width, -half_h);
        let left_max = player_pos + Vec2::new(0.0, half_h);

        // 우측 AABB
        let right_min = player_pos + Vec2::new(0.0, -half_h);
        let right_max = player_pos + Vec2::new(area_width, half_h);

        spawn_timed_effect(
            world,
            player_pos + Vec2::new(-area_width * 0.5, 0.0),
            Vec2::new(area_width, area_height * 1.2),
            SurvivorSprite::WhipSlashLeft,
            [1.0, 0.96, 0.72, 0.88],
            0.12,
            0.65,
        );
        spawn_timed_effect(
            world,
            player_pos + Vec2::new(area_width * 0.5, 0.0),
            Vec2::new(area_width, area_height * 1.2),
            SurvivorSprite::WhipSlashRight,
            [1.0, 0.96, 0.72, 0.88],
            0.12,
            0.65,
        );

        let mut hits: Vec<Entity> = self
            .grid
            .query_aabb(left_min, left_max, mask)
            .into_iter()
            .chain(self.grid.query_aabb(right_min, right_max, mask))
            .collect();

        // 좌/우 양쪽에 걸쳐도 한 번만 처리
        hits.sort_by_key(|e| e.0);
        hits.dedup();

        let mut killed: u32 = 0;
        for entity in hits {
            if apply_damage_to_enemy(world, entity, damage) {
                killed += 1;
            }
        }
        if killed > 0 {
            if let Some(stats) = world.resource_mut::<GameStats>() {
                stats.kills += killed;
            }
        }
    }
}

/// Magic Wand 발화 시스템.
///
/// 매 cooldown 마다 가장 가까운 적 1마리를 향해 직선 투사체를 발사한다.
/// `spawn_projectile` 헬퍼를 통해 투사체를 생성하고,
/// 이동·충돌·데미지는 `ProjectileSystem` 이 처리한다.
pub struct MagicWandSystem {
    pub grid: SpatialGrid,
    pub target_radius: f32, // 가장 가까운 적 탐색 반경 (px)
}

impl Default for MagicWandSystem {
    fn default() -> Self {
        Self {
            grid: SpatialGrid::new(128.0),
            target_radius: 400.0,
        }
    }
}

impl System for MagicWandSystem {
    fn run(&mut self, world: &mut World, dt: f32) {
        if !matches!(world.resource::<GameState>(), Some(GameState::Playing)) {
            return;
        }

        // 1) Player 위치 + entity 캐시 (borrow 를 즉시 끊음)
        let Some((player_entity, player_pos)) = world
            .query2::<Player, Transform>()
            .next()
            .map(|(e, _, t)| (e, t.position))
        else {
            return;
        };

        // 2) stats 캐시
        let stats = read_player_stats(world);

        // 3) MagicWand 슬롯 tick — cooldown 미달이면 즉시 반환
        let fire_info: Option<(f32, f32, f32, u8)> = {
            let Some(inv) = world.get_mut::<WeaponInventory>(player_entity) else {
                return;
            };
            let Some(slot) = inv.magic_wand_slot_mut() else {
                return;
            };
            if !slot.tick_with_cooldown_multiplier(dt, stats.cooldown) {
                return;
            }
            if let WeaponKind::MagicWand {
                damage,
                projectile_speed,
                lifetime,
                pierce,
            } = slot.kind
            {
                Some((damage, projectile_speed, lifetime, pierce))
            } else {
                None
            }
        };
        let Some((base_damage, base_speed, base_lifetime, pierce)) = fire_info else {
            return;
        };

        // stats 적용
        let damage = base_damage + stats.might;
        let projectile_speed = base_speed * stats.projectile_speed;
        let lifetime = base_lifetime * stats.duration;

        // 4) grid rebuild + 가장 가까운 적 탐색
        self.grid.rebuild(world);
        let candidates =
            self.grid
                .query_radius(player_pos, self.target_radius, CollisionLayer(LAYER_ENEMY));
        if candidates.is_empty() {
            return;
        }

        let nearest = candidates
            .into_iter()
            .filter_map(|e| world.get::<Transform>(e).map(|t| (e, t.position)))
            .min_by(|a, b| {
                let da = (a.1 - player_pos).length_squared();
                let db = (b.1 - player_pos).length_squared();
                da.partial_cmp(&db).unwrap_or(std::cmp::Ordering::Equal)
            });
        let Some((_target_entity, target_pos)) = nearest else {
            return;
        };

        // 4) 방향 벡터 정규화 + 투사체 스폰 (연보라색)
        let dir = target_pos - player_pos;
        let dir = if dir.length_squared() > 0.0 {
            dir.normalize()
        } else {
            Vec2::new(1.0, 0.0)
        };
        let velocity = dir * projectile_speed;

        spawn_projectile(
            world,
            player_pos,
            velocity,
            lifetime,
            pierce,
            damage,
            [0.9, 0.7, 1.0], // 연보라색 — Magic Wand 컬러
        );
    }
}

/// Knife 발화 시스템.
///
/// 매 cooldown 마다 가장 가까운 적 방향으로 `amount` 개의 직선 투사체를 동시 발사한다.
/// `amount > 1` 이면 `spread_radians` 범위로 부채꼴 분산.
pub struct KnifeSystem {
    pub grid: SpatialGrid,
    pub target_radius: f32,
}

impl Default for KnifeSystem {
    fn default() -> Self {
        Self {
            grid: SpatialGrid::new(128.0),
            target_radius: 400.0,
        }
    }
}

impl System for KnifeSystem {
    fn run(&mut self, world: &mut World, dt: f32) {
        if !matches!(world.resource::<GameState>(), Some(GameState::Playing)) {
            return;
        }

        // 1) Player 위치 + entity 캐시
        let Some((player_entity, player_pos)) = world
            .query2::<Player, Transform>()
            .next()
            .map(|(e, _, t)| (e, t.position))
        else {
            return;
        };

        // 2) stats 캐시
        let stats = read_player_stats(world);

        // 3) Knife 슬롯 tick — cooldown 미달이면 즉시 반환
        let fire_info: Option<(f32, f32, f32, u8, u8, f32)> = {
            let Some(inv) = world.get_mut::<WeaponInventory>(player_entity) else {
                return;
            };
            let Some(slot) = inv.knife_slot_mut() else {
                return;
            };
            if !slot.tick_with_cooldown_multiplier(dt, stats.cooldown) {
                return;
            }
            if let WeaponKind::Knife {
                damage,
                projectile_speed,
                lifetime,
                pierce,
                amount,
                spread_radians,
            } = slot.kind
            {
                Some((
                    damage,
                    projectile_speed,
                    lifetime,
                    pierce,
                    amount,
                    spread_radians,
                ))
            } else {
                None
            }
        };
        let Some((base_damage, base_speed, base_lifetime, pierce, base_amount, spread_radians)) =
            fire_info
        else {
            return;
        };

        // stats 적용
        let damage = base_damage + stats.might;
        let projectile_speed = base_speed * stats.projectile_speed;
        let lifetime = base_lifetime * stats.duration;
        let amount = ((base_amount as i32) + stats.amount).max(1) as u8;

        // 3) grid rebuild + 가장 가까운 적 탐색
        self.grid.rebuild(world);
        let candidates =
            self.grid
                .query_radius(player_pos, self.target_radius, CollisionLayer(LAYER_ENEMY));
        if candidates.is_empty() {
            return; // 적 없으면 발사 skip
        }

        let nearest = candidates
            .into_iter()
            .filter_map(|e| world.get::<Transform>(e).map(|t| (e, t.position)))
            .min_by(|a, b| {
                let da = (a.1 - player_pos).length_squared();
                let db = (b.1 - player_pos).length_squared();
                da.partial_cmp(&db).unwrap_or(std::cmp::Ordering::Equal)
            });
        let Some((_target_entity, target_pos)) = nearest else {
            return;
        };

        // 4) 방향 벡터 계산
        let dir = target_pos - player_pos;
        let base_dir = if dir.length_squared() > 0.0 {
            dir.normalize()
        } else {
            Vec2::new(1.0, 0.0)
        };
        let base_angle = base_dir.y.atan2(base_dir.x);

        // 5) amount 개 투사체 발사 (흰색) — spread_radians 부채꼴 분산
        for i in 0..amount {
            let angle = if amount == 1 {
                base_angle
            } else {
                base_angle - spread_radians * 0.5
                    + spread_radians * (i as f32) / ((amount - 1) as f32)
            };
            let velocity = Vec2::new(angle.cos(), angle.sin()) * projectile_speed;
            spawn_projectile_ex(
                world,
                player_pos,
                velocity,
                lifetime,
                pierce,
                damage,
                [0.9, 0.9, 0.9], // 흰색 — Knife 컬러
                ProjectileBehavior::Straight,
            );
        }
    }
}

/// Axe 발화 시스템.
///
/// 매 cooldown 마다 위로 던져 중력으로 떨어지는 포물선 투사체를 발사한다.
/// `ProjectileBehavior::Arc { gravity }` 를 사용한다.
///
/// Y 축 방향 주의: y 클수록 아래 → "위로 던지기" = `velocity.y = -initial_speed`.
pub struct AxeSystem {
    pub grid: SpatialGrid,
}

impl Default for AxeSystem {
    fn default() -> Self {
        Self {
            grid: SpatialGrid::new(128.0),
        }
    }
}

impl System for AxeSystem {
    fn run(&mut self, world: &mut World, dt: f32) {
        if !matches!(world.resource::<GameState>(), Some(GameState::Playing)) {
            return;
        }

        // 1) Player 위치 + entity 캐시
        let Some((player_entity, player_pos)) = world
            .query2::<Player, Transform>()
            .next()
            .map(|(e, _, t)| (e, t.position))
        else {
            return;
        };

        // 2) stats 캐시
        let stats = read_player_stats(world);

        // 3) Axe 슬롯 tick — cooldown 미달이면 즉시 반환
        let fire_info: Option<(f32, f32, f32, f32, u8, u8)> = {
            let Some(inv) = world.get_mut::<WeaponInventory>(player_entity) else {
                return;
            };
            let Some(slot) = inv.axe_slot_mut() else {
                return;
            };
            if !slot.tick_with_cooldown_multiplier(dt, stats.cooldown) {
                return;
            }
            if let WeaponKind::Axe {
                damage,
                initial_speed,
                gravity,
                lifetime,
                pierce,
                amount,
            } = slot.kind
            {
                Some((damage, initial_speed, gravity, lifetime, pierce, amount))
            } else {
                None
            }
        };
        let Some((base_damage, base_speed, gravity, base_lifetime, pierce, base_amount)) =
            fire_info
        else {
            return;
        };

        // stats 적용
        let damage = base_damage + stats.might;
        let initial_speed = base_speed * stats.projectile_speed;
        let lifetime = base_lifetime * stats.duration;
        let amount = ((base_amount as i32) + stats.amount).max(1) as u8;

        // 4) amount 개 투사체 발사 (갈색)
        //    각 투사체마다 약간의 x 오프셋을 부여해 시각적으로 분산
        let x_offsets: &[f32] = &[-8.0, 0.0, 8.0, -16.0, 16.0, -24.0];
        for i in 0..amount {
            let x_offset = x_offsets
                .get(i as usize)
                .copied()
                .unwrap_or((i as f32) * 8.0 - (amount as f32) * 4.0);
            let velocity = Vec2::new(x_offset, -initial_speed); // 음수 y = 위 방향
            spawn_projectile_ex(
                world,
                player_pos,
                velocity,
                lifetime,
                pierce,
                damage,
                [0.8, 0.5, 0.2], // 갈색 — Axe 컬러
                ProjectileBehavior::Arc { gravity },
            );
        }
    }
}

/// Cross 발화 시스템.
///
/// 매 cooldown 마다 가장 가까운 적 방향으로 `amount` 개의 부메랑 투사체를 발사한다.
/// `ProjectileBehavior::Boomerang { return_at, elapsed: 0.0, returned: false }` 를 사용한다.
/// 황금색. pierce 가 크므로 적을 관통하며 갔다가 돌아온다.
pub struct CrossSystem {
    pub grid: SpatialGrid,
    pub target_radius: f32,
}

impl Default for CrossSystem {
    fn default() -> Self {
        Self {
            grid: SpatialGrid::new(128.0),
            target_radius: 400.0,
        }
    }
}

impl System for CrossSystem {
    fn run(&mut self, world: &mut World, dt: f32) {
        if !matches!(world.resource::<GameState>(), Some(GameState::Playing)) {
            return;
        }

        // 1) Player 위치 + entity 캐시
        let Some((player_entity, player_pos)) = world
            .query2::<Player, Transform>()
            .next()
            .map(|(e, _, t)| (e, t.position))
        else {
            return;
        };

        // 2) stats 캐시
        let stats = read_player_stats(world);

        // 3) Cross 슬롯 tick — cooldown 미달이면 즉시 반환
        let fire_info: Option<(f32, f32, f32, u8, u8, f32)> = {
            let Some(inv) = world.get_mut::<WeaponInventory>(player_entity) else {
                return;
            };
            let Some(slot) = inv.cross_slot_mut() else {
                return;
            };
            if !slot.tick_with_cooldown_multiplier(dt, stats.cooldown) {
                return;
            }
            if let WeaponKind::Cross {
                damage,
                projectile_speed,
                lifetime,
                pierce,
                amount,
                return_at,
            } = slot.kind
            {
                Some((
                    damage,
                    projectile_speed,
                    lifetime,
                    pierce,
                    amount,
                    return_at,
                ))
            } else {
                None
            }
        };
        let Some((base_damage, base_speed, base_lifetime, pierce, base_amount, return_at)) =
            fire_info
        else {
            return;
        };

        // stats 적용
        let damage = base_damage + stats.might;
        let projectile_speed = base_speed * stats.projectile_speed;
        let lifetime = base_lifetime * stats.duration;
        let amount = ((base_amount as i32) + stats.amount).max(1) as u8;

        // 3) grid rebuild + 가장 가까운 적 탐색
        self.grid.rebuild(world);
        let candidates =
            self.grid
                .query_radius(player_pos, self.target_radius, CollisionLayer(LAYER_ENEMY));
        if candidates.is_empty() {
            return;
        }

        let nearest = candidates
            .into_iter()
            .filter_map(|e| world.get::<Transform>(e).map(|t| (e, t.position)))
            .min_by(|a, b| {
                let da = (a.1 - player_pos).length_squared();
                let db = (b.1 - player_pos).length_squared();
                da.partial_cmp(&db).unwrap_or(std::cmp::Ordering::Equal)
            });
        let Some((_target_entity, target_pos)) = nearest else {
            return;
        };

        // 4) 방향 벡터 계산
        let dir = target_pos - player_pos;
        let base_dir = if dir.length_squared() > 0.0 {
            dir.normalize()
        } else {
            Vec2::new(1.0, 0.0)
        };
        let base_angle = base_dir.y.atan2(base_dir.x);

        // 5) amount 개 부메랑 투사체 발사 (황금색)
        for i in 0..amount {
            let angle = if amount == 1 {
                base_angle
            } else {
                // 부채꼴 분산 (Cross 는 spread 작게)
                let spread = 0.2_f32;
                base_angle - spread * 0.5 + spread * (i as f32) / ((amount - 1) as f32)
            };
            let velocity = Vec2::new(angle.cos(), angle.sin()) * projectile_speed;
            spawn_projectile_ex(
                world,
                player_pos,
                velocity,
                lifetime,
                pierce,
                damage,
                [0.9, 0.9, 0.5], // 황금색 — Cross 컬러
                ProjectileBehavior::Boomerang {
                    return_at,
                    elapsed: 0.0,
                    returned: false,
                },
            );
        }
    }
}

/// Fire Wand 발화 시스템.
///
/// 매 cooldown 마다 탐색 범위 안의 적 중 *랜덤* 1마리를 향해 고데미지 단발 투사체를 발사한다.
/// MagicWand(가장 가까운 적) 와 달리 무작위 타깃 선택.
/// 주황색. pierce 없음, 데미지 큼.
pub struct FireWandSystem {
    pub grid: SpatialGrid,
    pub target_radius: f32,
}

impl Default for FireWandSystem {
    fn default() -> Self {
        Self {
            grid: SpatialGrid::new(128.0),
            target_radius: 400.0,
        }
    }
}

impl System for FireWandSystem {
    fn run(&mut self, world: &mut World, dt: f32) {
        if !matches!(world.resource::<GameState>(), Some(GameState::Playing)) {
            return;
        }

        // 1) Player 위치 + entity 캐시
        let Some((player_entity, player_pos)) = world
            .query2::<Player, Transform>()
            .next()
            .map(|(e, _, t)| (e, t.position))
        else {
            return;
        };

        // 2) stats 캐시
        let stats = read_player_stats(world);

        // 3) FireWand 슬롯 tick — cooldown 미달이면 즉시 반환
        let fire_info: Option<(f32, f32, f32, u8)> = {
            let Some(inv) = world.get_mut::<WeaponInventory>(player_entity) else {
                return;
            };
            let Some(slot) = inv.fire_wand_slot_mut() else {
                return;
            };
            if !slot.tick_with_cooldown_multiplier(dt, stats.cooldown) {
                return;
            }
            if let WeaponKind::FireWand {
                damage,
                projectile_speed,
                lifetime,
                pierce,
            } = slot.kind
            {
                Some((damage, projectile_speed, lifetime, pierce))
            } else {
                None
            }
        };
        let Some((base_damage, base_speed, base_lifetime, pierce)) = fire_info else {
            return;
        };

        // stats 적용
        let damage = base_damage + stats.might;
        let projectile_speed = base_speed * stats.projectile_speed;
        let lifetime = base_lifetime * stats.duration;

        // 3) grid rebuild + 후보 적 수집
        self.grid.rebuild(world);
        let candidates =
            self.grid
                .query_radius(player_pos, self.target_radius, CollisionLayer(LAYER_ENEMY));
        if candidates.is_empty() {
            return;
        }

        // 4) 후보 중 랜덤 1마리 선택
        let mut rng = rand::thread_rng();
        let Some(&target_entity) = candidates.choose(&mut rng) else {
            return;
        };
        let Some(target_pos) = world.get::<Transform>(target_entity).map(|t| t.position) else {
            return;
        };

        // 5) 방향 벡터 정규화 + 투사체 스폰 (주황색)
        let dir = target_pos - player_pos;
        let dir = if dir.length_squared() > 0.0 {
            dir.normalize()
        } else {
            Vec2::new(1.0, 0.0)
        };
        let velocity = dir * projectile_speed;

        spawn_projectile_ex(
            world,
            player_pos,
            velocity,
            lifetime,
            pierce,
            damage,
            [1.0, 0.4, 0.1], // 주황색 — Fire Wand 컬러
            ProjectileBehavior::Straight,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hit_flash_removes_component_after_expiry() {
        let mut world = World::new();
        let entity = world.spawn();
        let original = [0.2, 0.4, 0.6, 1.0];
        let original_scale = Vec2::splat(42.0);
        let mut sprite = Sprite::colored(1.0, 1.0, 1.0);
        sprite.color = [1.0, 1.0, 1.0, 1.0];

        world.add_component(entity, sprite);
        world.add_component(
            entity,
            Transform {
                position: Vec2::ZERO,
                scale: original_scale * SCALE_BUMP,
                rotation: 0.0,
                z: 0.0,
            },
        );
        world.add_component(
            entity,
            HitFlash {
                remaining: 0.01,
                duration: FLASH_DURATION,
                original,
                original_scale,
            },
        );

        let mut system = HitFlashSystem;
        system.run(&mut world, 0.02);

        assert!(world.get::<HitFlash>(entity).is_none());
        assert_eq!(world.get::<Sprite>(entity).unwrap().color, original);
        assert_eq!(
            world.get::<Transform>(entity).unwrap().scale,
            original_scale
        );
    }
}
