//! 지속 area-damage 무기 시스템.
//!
//! - [`GarlicSystem`]   — 플레이어 중심 오라. cooldown 마다 반경 안 모든 적에게 tick 데미지.
//! - [`HolyWaterSystem`] — cooldown 마다 플레이어 주변 랜덤 위치에 [`HolyWaterPool`] 드롭.
//! - [`HolyWaterPoolSystem`] — Pool 의 lifetime 감소 + tick 마다 area-damage.

use engine::components::GameState;
use engine::{CollisionLayer, Entity, SpatialGrid, System, Transform, World};
use glam::Vec2;

use super::damage::apply_damage_to_enemy;
use super::hud::GameStats;
use super::inventory::{WeaponInventory, WeaponKind};
use super::player::Player;
use super::sprites::{add_tinted_sprite, SurvivorSprite};
use super::stats::read_player_stats;
use super::LAYER_ENEMY;

// ─── HolyWaterPool 컴포넌트 ──────────────────────────────────────────────────

/// HolyWater 가 드롭한 지속 풀. 시간이 지나면 사라지고, tick_cooldown 마다 area-damage.
#[derive(Debug, Clone)]
pub struct HolyWaterPool {
    pub damage: f32,        // 풀 tick 당 데미지
    pub radius: f32,        // 풀의 area-damage 반경
    pub lifetime: f32,      // 남은 시간 (감소)
    pub tick_cooldown: f32, // 다음 tick 까지 간격
    pub tick_elapsed: f32,  // 누적 시간
}

// ─── GarlicSystem ─────────────────────────────────────────────────────────────

/// Garlic 오라 시스템 — 플레이어 중심 반경 안 적에게 tick 데미지.
/// cooldown 마다 1회 발화. WhipSystem 의 area-damage 흐름 답습.
pub struct GarlicSystem {
    pub grid: SpatialGrid,
}

impl Default for GarlicSystem {
    fn default() -> Self {
        Self {
            grid: SpatialGrid::new(128.0),
        }
    }
}

impl System for GarlicSystem {
    fn run(&mut self, world: &mut World, dt: f32) {
        if !matches!(world.resource::<GameState>(), Some(GameState::Playing)) {
            return;
        }

        // 1) Player + Garlic slot 캐시 (borrow 를 블록 안에서 즉시 끊음)
        let Some((player_entity, player_pos)) = world
            .query2::<Player, Transform>()
            .next()
            .map(|(e, _, t)| (e, t.position))
        else {
            return;
        };

        // stats 캐시
        let stats = read_player_stats(world);

        let fire_info: Option<(f32, f32)> = {
            let Some(inv) = world.get_mut::<WeaponInventory>(player_entity) else {
                return;
            };
            let Some(slot) = inv.garlic_slot_mut() else {
                return;
            };
            if !slot.tick_with_cooldown_multiplier(dt, stats.cooldown) {
                return;
            }
            if let WeaponKind::Garlic { damage, radius } = slot.kind {
                Some((damage, radius))
            } else {
                None
            }
        };
        let Some((base_damage, base_radius)) = fire_info else {
            return;
        };

        // stats 적용
        let damage = base_damage + stats.might;
        let radius = base_radius * stats.area;

        // 2) grid rebuild + 원형 query
        self.grid.rebuild(world);
        let hits = self
            .grid
            .query_radius(player_pos, radius, CollisionLayer(LAYER_ENEMY));
        if hits.is_empty() {
            return;
        }

        // 3) 각 hit zombie 에 데미지 + HitFlash + 사망 처리
        let mut killed: u32 = 0;
        for zombie in hits {
            if apply_damage_to_enemy(world, zombie, damage) {
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

// ─── HolyWaterSystem ──────────────────────────────────────────────────────────

/// HolyWater 시스템 — cooldown 마다 플레이어 주변 랜덤 위치에 Pool 드롭.
pub struct HolyWaterSystem;

impl System for HolyWaterSystem {
    fn run(&mut self, world: &mut World, dt: f32) {
        if !matches!(world.resource::<GameState>(), Some(GameState::Playing)) {
            return;
        }

        let Some((player_entity, player_pos)) = world
            .query2::<Player, Transform>()
            .next()
            .map(|(e, _, t)| (e, t.position))
        else {
            return;
        };

        // stats 캐시
        let stats = read_player_stats(world);

        let fire_info: Option<(f32, f32, f32, f32, u8)> = {
            let Some(inv) = world.get_mut::<WeaponInventory>(player_entity) else {
                return;
            };
            let Some(slot) = inv.holy_water_slot_mut() else {
                return;
            };
            if !slot.tick_with_cooldown_multiplier(dt, stats.cooldown) {
                return;
            }
            if let WeaponKind::HolyWater {
                damage,
                radius,
                pool_lifetime,
                tick_cooldown,
                drop_count,
            } = slot.kind
            {
                Some((damage, radius, pool_lifetime, tick_cooldown, drop_count))
            } else {
                None
            }
        };
        let Some((base_damage, base_radius, base_lifetime, tick_cooldown, base_drop_count)) =
            fire_info
        else {
            return;
        };

        // stats 적용
        let damage = base_damage + stats.might;
        let radius = base_radius * stats.area;
        let pool_lifetime = base_lifetime * stats.duration;
        let drop_count = ((base_drop_count as i32) + stats.amount).max(1) as u8;

        // drop_count 개의 풀을 플레이어 주변 랜덤 위치에 스폰
        use rand::Rng;
        let mut rng = rand::thread_rng();
        for _ in 0..drop_count {
            // 플레이어로부터 50~150px 거리 랜덤 각도
            let angle = rng.gen_range(0.0..std::f32::consts::TAU);
            let dist = rng.gen_range(50.0..150.0);
            let pos = player_pos + Vec2::new(angle.cos(), angle.sin()) * dist;
            spawn_holy_water_pool(world, pos, damage, radius, pool_lifetime, tick_cooldown);
        }
    }
}

// ─── HolyWaterPoolSystem ──────────────────────────────────────────────────────

/// Pool 들의 lifetime + tick 처리.
pub struct HolyWaterPoolSystem {
    pub grid: SpatialGrid,
}

impl Default for HolyWaterPoolSystem {
    fn default() -> Self {
        Self {
            grid: SpatialGrid::new(128.0),
        }
    }
}

impl System for HolyWaterPoolSystem {
    fn run(&mut self, world: &mut World, dt: f32) {
        if !matches!(world.resource::<GameState>(), Some(GameState::Playing)) {
            return;
        }

        self.grid.rebuild(world);

        // 1) Pool 상태 수집 (borrow 를 Vec 으로 분리 — grid 와 world get_mut 충돌 회피)
        let pools: Vec<(Entity, Vec2, f32, f32, f32, f32, bool)> = world
            .query2::<HolyWaterPool, Transform>()
            .map(|(e, pool, t)| {
                let new_life = pool.lifetime - dt;
                let mut new_tick = pool.tick_elapsed + dt;
                let mut fired = false;
                if new_tick >= pool.tick_cooldown {
                    new_tick -= pool.tick_cooldown;
                    fired = true;
                }
                (
                    e,
                    t.position,
                    pool.damage,
                    pool.radius,
                    new_life,
                    new_tick,
                    fired,
                )
            })
            .collect();

        let mut to_despawn: Vec<Entity> = Vec::new();
        let mut tick_hits: Vec<(Entity, f32)> = Vec::new(); // (zombie_entity, damage)

        for (e, pos, damage, radius, life, tick_elapsed, fired) in pools {
            if life <= 0.0 {
                to_despawn.push(e);
                continue;
            }
            // pool 상태 갱신
            if let Some(pool) = world.get_mut::<HolyWaterPool>(e) {
                pool.lifetime = life;
                pool.tick_elapsed = tick_elapsed;
            }
            if fired {
                let zombies = self
                    .grid
                    .query_radius(pos, radius, CollisionLayer(LAYER_ENEMY));
                for z in zombies {
                    tick_hits.push((z, damage));
                }
            }
        }

        // 2) tick_hits 적용
        let mut killed: u32 = 0;
        for (zombie, damage) in tick_hits {
            if apply_damage_to_enemy(world, zombie, damage) {
                killed += 1;
            }
        }

        for e in to_despawn {
            world.despawn(e);
        }

        if killed > 0 {
            if let Some(stats) = world.resource_mut::<GameStats>() {
                stats.kills += killed;
            }
        }
    }
}

// ─── 헬퍼 ────────────────────────────────────────────────────────────────────

/// 지정 위치에 HolyWaterPool 엔티티를 스폰한다.
///
/// 연한 파랑 사각형으로 시각화. z = 0.2 (zombie(0.5)·XpGem(0.3) 아래).
pub fn spawn_holy_water_pool(
    world: &mut World,
    pos: Vec2,
    damage: f32,
    radius: f32,
    lifetime: f32,
    tick_cooldown: f32,
) {
    let e = world.spawn();
    world.add_component(
        e,
        Transform {
            position: pos,
            scale: Vec2::new(radius * 2.0, radius * 2.0), // 풀 시각화: 반경을 폭으로 표현
            rotation: 0.0,
            z: 0.2, // zombie(0.5) · XpGem(0.3) 아래에 렌더
        },
    );
    add_tinted_sprite(world, e, SurvivorSprite::Vacuum, [0.45, 0.65, 1.0, 0.65]);
    world.add_component(
        e,
        HolyWaterPool {
            damage,
            radius,
            lifetime,
            tick_cooldown,
            tick_elapsed: 0.0,
        },
    );
}
