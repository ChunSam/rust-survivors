use engine::{CollisionLayer, Entity, GameState, SpatialGrid, System, Transform, World};
use glam::Vec2;

use super::combat::{apply_damage_to_targets, random_enemies_in_radius, weapon_fire_context};
use super::inventory::{WeaponInventory, WeaponKind};
use super::sprites::{add_tinted_sprite, SurvivorSprite};
use super::LAYER_ENEMY;

/// 짧게 보이는 번개 시각 효과. lifetime 감소 → despawn.
#[derive(Debug, Clone)]
pub struct LightningFlash {
    pub lifetime: f32,
}

/// LightningRing 슬롯 — cooldown 마다 strike_count 마리의 *랜덤 zombie* 위치에 즉시 area damage.
pub struct LightningRingSystem {
    pub grid: SpatialGrid,
    pub target_radius: f32, // 후보 적 탐색 반경
}

impl Default for LightningRingSystem {
    fn default() -> Self {
        Self {
            grid: SpatialGrid::new(128.0),
            target_radius: 600.0,
        }
    }
}

impl System for LightningRingSystem {
    fn run(&mut self, world: &mut World, dt: f32) {
        if !matches!(world.resource::<GameState>(), Some(GameState::Playing)) {
            return;
        }

        let Some(ctx) = weapon_fire_context(world) else {
            return;
        };
        let player_entity = ctx.player_entity;
        let player_pos = ctx.player_pos;
        let stats = ctx.stats;

        let fire_info = {
            let Some(inv) = world.get_mut::<WeaponInventory>(player_entity) else {
                return;
            };
            let Some(slot) = inv.lightning_ring_slot_mut() else {
                return;
            };
            if !slot.tick_with_cooldown_multiplier(dt, stats.cooldown) {
                return;
            }
            if let WeaponKind::LightningRing {
                damage,
                strike_count,
                hit_radius,
            } = slot.kind
            {
                Some((damage, strike_count, hit_radius))
            } else {
                None
            }
        };
        let Some((base_damage, base_strike_count, base_hit_radius)) = fire_info else {
            return;
        };

        // stats 적용
        let damage = base_damage + stats.might;
        let strike_count = ((base_strike_count as i32) + stats.amount).max(1) as u8;
        let hit_radius = base_hit_radius * stats.area;

        // 후보 zombie 들
        self.grid.rebuild(world);
        // strike_count 마리 랜덤 선택
        let targets = random_enemies_in_radius(
            world,
            &self.grid,
            player_pos,
            self.target_radius,
            strike_count as usize,
        );

        for (_target, target_pos) in targets {
            // 시각 flash 잠깐
            spawn_lightning_flash(world, target_pos);
            // area damage at target_pos
            let hits = self
                .grid
                .query_radius(target_pos, hit_radius, CollisionLayer(LAYER_ENEMY));
            apply_damage_to_targets(world, hits, damage);
        }
    }
}

/// 번개 flash 의 lifetime 감소 + despawn.
pub struct LightningFlashSystem;

impl System for LightningFlashSystem {
    fn run(&mut self, world: &mut World, dt: f32) {
        if !matches!(world.resource::<GameState>(), Some(GameState::Playing)) {
            return;
        }
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
        for e in to_despawn {
            world.despawn(e);
        }
    }
}

/// 번개 flash 엔티티를 스폰한다.
pub fn spawn_lightning_flash(world: &mut World, pos: Vec2) {
    spawn_timed_effect(
        world,
        pos,
        Vec2::new(72.0, 108.0),
        SurvivorSprite::LightningStrike,
        [1.0, 1.0, 0.7, 0.95],
        0.15,
        0.7,
    );
}

/// 짧게 보였다 사라지는 전투 이펙트 엔티티를 스폰한다.
pub fn spawn_timed_effect(
    world: &mut World,
    pos: Vec2,
    scale: Vec2,
    sprite: SurvivorSprite,
    color: [f32; 4],
    lifetime: f32,
    z: f32,
) {
    let e = world.spawn();
    world.add_component(
        e,
        Transform {
            position: pos,
            scale,
            rotation: 0.0,
            z,
        },
    );
    add_tinted_sprite(world, e, sprite, color);
    if let Some(t) = world.get_mut::<Transform>(e) {
        t.scale = sprite.fit_inside(scale);
    }
    world.add_component(e, LightningFlash { lifetime });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn timed_effect_preserves_sprite_aspect_inside_requested_bounds() {
        let mut world = World::new();
        let requested = Vec2::new(72.0, 108.0);

        spawn_timed_effect(
            &mut world,
            Vec2::ZERO,
            requested,
            SurvivorSprite::LightningStrike,
            [1.0, 1.0, 1.0, 1.0],
            0.2,
            0.7,
        );

        let (_, transform) = world.query::<Transform>().next().unwrap();
        assert!(transform.scale.x <= requested.x);
        assert!(transform.scale.y <= requested.y);
        assert!((transform.scale.x / transform.scale.y - 1.0).abs() < 0.0001);
    }
}
