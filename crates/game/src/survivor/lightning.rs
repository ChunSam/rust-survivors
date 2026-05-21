use engine::{CollisionLayer, Entity, Sprite, SpatialGrid, System, Transform, World};
use engine::components::GameState;
use glam::Vec2;
use rand::seq::SliceRandom;

use super::damage::apply_damage_to_zombie;
use super::hud::GameStats;
use super::inventory::{WeaponInventory, WeaponKind};
use super::player::Player;
use super::stats::read_player_stats;
use super::LAYER_ENEMY;

/// 짧게 보이는 번개 시각 효과. lifetime 감소 → despawn.
#[derive(Debug, Clone)]
pub struct LightningFlash {
    pub lifetime: f32,
}

/// LightningRing 슬롯 — cooldown 마다 strike_count 마리의 *랜덤 zombie* 위치에 즉시 area damage.
pub struct LightningRingSystem {
    pub grid:          SpatialGrid,
    pub target_radius: f32,  // 후보 적 탐색 반경
}

impl Default for LightningRingSystem {
    fn default() -> Self {
        Self { grid: SpatialGrid::new(128.0), target_radius: 600.0 }
    }
}

impl System for LightningRingSystem {
    fn run(&mut self, world: &mut World, dt: f32) {
        if !matches!(world.resource::<GameState>(), Some(GameState::Playing)) { return; }

        let Some((player_entity, player_pos)) = world
            .query2::<Player, Transform>()
            .next()
            .map(|(e, _, t)| (e, t.position))
        else { return };

        // stats 캐시
        let stats = read_player_stats(world);

        let fire_info = {
            let Some(inv) = world.get_mut::<WeaponInventory>(player_entity) else { return };
            let Some(slot) = inv.lightning_ring_slot_mut() else { return };
            if !slot.tick_with_cooldown_multiplier(dt, stats.cooldown) { return; }
            if let WeaponKind::LightningRing { damage, strike_count, hit_radius } = slot.kind {
                Some((damage, strike_count, hit_radius))
            } else { None }
        };
        let Some((base_damage, base_strike_count, base_hit_radius)) = fire_info else { return };

        // stats 적용
        let damage = base_damage + stats.might;
        let strike_count = ((base_strike_count as i32) + stats.amount).max(1) as u8;
        let hit_radius = base_hit_radius * stats.area;

        // 후보 zombie 들
        self.grid.rebuild(world);
        let candidates = self.grid.query_radius(player_pos, self.target_radius, CollisionLayer(LAYER_ENEMY));
        if candidates.is_empty() { return; }

        // strike_count 마리 랜덤 선택
        let mut rng = rand::thread_rng();
        let targets: Vec<Entity> = candidates
            .choose_multiple(&mut rng, strike_count as usize)
            .copied()
            .collect();

        let mut killed: u32 = 0;
        for target in targets {
            let target_pos = match world.get::<Transform>(target) {
                Some(t) => t.position,
                None => continue,
            };
            // 시각 flash 잠깐
            spawn_lightning_flash(world, target_pos);
            // area damage at target_pos
            let hits = self.grid.query_radius(target_pos, hit_radius, CollisionLayer(LAYER_ENEMY));
            for z in hits {
                if apply_damage_to_zombie(world, z, damage) {
                    killed += 1;
                }
            }
        }

        if killed > 0 {
            if let Some(stats) = world.resource_mut::<GameStats>() { stats.kills += killed; }
        }
    }
}

/// 번개 flash 의 lifetime 감소 + despawn.
pub struct LightningFlashSystem;

impl System for LightningFlashSystem {
    fn run(&mut self, world: &mut World, dt: f32) {
        if !matches!(world.resource::<GameState>(), Some(GameState::Playing)) { return; }
        let updates: Vec<(Entity, f32)> = world
            .query::<LightningFlash>()
            .map(|(e, f)| (e, f.lifetime - dt))
            .collect();
        let mut to_despawn: Vec<Entity> = Vec::new();
        for (e, new_life) in updates {
            if new_life <= 0.0 {
                to_despawn.push(e);
            } else if let Some(f) = world.get_mut::<LightningFlash>(e) {
                f.lifetime = new_life;
            }
        }
        for e in to_despawn { world.despawn(e); }
    }
}

/// 번개 flash 엔티티를 스폰한다.
pub fn spawn_lightning_flash(world: &mut World, pos: Vec2) {
    let e = world.spawn();
    world.add_component(e, Transform {
        position: pos,
        scale:    Vec2::new(40.0, 40.0),
        rotation: 0.0,
        z:        0.7,
    });
    world.add_component(e, Sprite::colored(1.0, 1.0, 0.6)); // 노란 flash
    world.add_component(e, LightningFlash { lifetime: 0.15 });
}
