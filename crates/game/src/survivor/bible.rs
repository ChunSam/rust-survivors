use engine::{CollisionLayer, Entity, GameState, SpatialGrid, System, Transform, World};
use glam::Vec2;

use super::damage::apply_damage_to_enemy;
use super::hud::GameStats;
use super::inventory::{WeaponInventory, WeaponKind};
use super::player::Player;
use super::sprites::{add_tinted_sprite, SurvivorSprite};
use super::stats::read_player_stats;
use super::LAYER_ENEMY;

/// 플레이어 주변을 회전하는 책 엔티티.
/// active 동안 책이 N권 회전, lifetime 만료 시 모두 despawn.
#[derive(Debug, Clone)]
pub struct OrbitingBook {
    pub angle: f32,
    pub angular_speed: f32, // 라디안/초
    pub radius: f32,
    pub damage: f32,
    pub hit_radius: f32,
    pub lifetime: f32, // 남은 활성 시간
    pub tick_cooldown: f32,
    pub tick_elapsed: f32,
}

/// KingBible 슬롯 — cooldown 마다 book_count 권의 책을 활성화 (스폰).
pub struct KingBibleSystem;

impl System for KingBibleSystem {
    fn run(&mut self, world: &mut World, dt: f32) {
        if !matches!(world.resource::<GameState>(), Some(GameState::Playing)) {
            return;
        }

        // 1) Player + slot 캐시
        let Some((player_entity, player_pos)) = world
            .query2::<Player, Transform>()
            .next()
            .map(|(e, _, t)| (e, t.position))
        else {
            return;
        };

        // stats 캐시
        let stats = read_player_stats(world);

        let fire_info = {
            let Some(inv) = world.get_mut::<WeaponInventory>(player_entity) else {
                return;
            };
            let Some(slot) = inv.king_bible_slot_mut() else {
                return;
            };
            if !slot.tick_with_cooldown_multiplier(dt, stats.cooldown) {
                return;
            }
            if let WeaponKind::KingBible {
                damage,
                book_count,
                radius,
                angular_speed,
                lifetime,
                tick_cooldown,
                hit_radius,
            } = slot.kind
            {
                Some((
                    damage,
                    book_count,
                    radius,
                    angular_speed,
                    lifetime,
                    tick_cooldown,
                    hit_radius,
                ))
            } else {
                None
            }
        };
        let Some((
            base_damage,
            base_book_count,
            base_radius,
            angular_speed,
            base_lifetime,
            tick_cooldown,
            base_hit_radius,
        )) = fire_info
        else {
            return;
        };

        // stats 적용
        let damage = base_damage + stats.might;
        let book_count = ((base_book_count as i32) + stats.amount).max(1) as u8;
        let radius = base_radius * stats.area;
        let hit_radius = base_hit_radius * stats.area;
        let lifetime = base_lifetime * stats.duration;

        // 2) book_count 권의 책을 angle 분산해서 스폰
        for i in 0..book_count {
            let angle = (i as f32 / book_count as f32) * std::f32::consts::TAU;
            spawn_book(
                world,
                player_pos,
                angle,
                angular_speed,
                radius,
                damage,
                hit_radius,
                lifetime,
                tick_cooldown,
            );
        }
    }
}

/// 회전 + lifetime + tick damage 처리.
pub struct OrbitingBookSystem {
    pub grid: SpatialGrid,
}

impl Default for OrbitingBookSystem {
    fn default() -> Self {
        Self {
            grid: SpatialGrid::new(128.0),
        }
    }
}

impl System for OrbitingBookSystem {
    fn run(&mut self, world: &mut World, dt: f32) {
        if !matches!(world.resource::<GameState>(), Some(GameState::Playing)) {
            return;
        }
        self.grid.rebuild(world);

        // Player 위치 캐시
        let Some(player_pos) = world
            .query2::<Player, Transform>()
            .next()
            .map(|(_, _, t)| t.position)
        else {
            return;
        };

        // 책 상태 수집
        let books: Vec<(Entity, OrbitingBook)> = world
            .query::<OrbitingBook>()
            .map(|(e, b)| (e, b.clone()))
            .collect();

        let mut to_despawn: Vec<Entity> = Vec::new();
        let mut tick_hits: Vec<(Vec2, f32, f32)> = Vec::new(); // (book_pos, hit_radius, damage)
        let mut book_updates: Vec<(Entity, f32, f32, Vec2)> = Vec::new(); // (entity, new_angle, new_tick_elapsed, new_pos)

        for (e, b) in &books {
            let new_life = b.lifetime - dt;
            if new_life <= 0.0 {
                to_despawn.push(*e);
                continue;
            }
            let new_angle = b.angle + b.angular_speed * dt;
            let new_pos = player_pos + Vec2::new(new_angle.cos(), new_angle.sin()) * b.radius;
            let mut new_tick = b.tick_elapsed + dt;
            let mut fired = false;
            if new_tick >= b.tick_cooldown {
                new_tick -= b.tick_cooldown;
                fired = true;
            }
            book_updates.push((*e, new_angle, new_tick, new_pos));
            if fired {
                tick_hits.push((new_pos, b.hit_radius, b.damage));
            }
            // lifetime 갱신
            if let Some(book) = world.get_mut::<OrbitingBook>(*e) {
                book.lifetime = new_life;
            }
        }

        // 책 위치/상태 갱신
        for (e, new_angle, new_tick, new_pos) in book_updates {
            if let Some(b) = world.get_mut::<OrbitingBook>(e) {
                b.angle = new_angle;
                b.tick_elapsed = new_tick;
            }
            if let Some(t) = world.get_mut::<Transform>(e) {
                t.position = new_pos;
            }
        }

        // hit 적용
        let mut killed: u32 = 0;
        for (pos, hit_radius, damage) in tick_hits {
            let zombies = self
                .grid
                .query_radius(pos, hit_radius, CollisionLayer(LAYER_ENEMY));
            for z in zombies {
                if apply_damage_to_enemy(world, z, damage) {
                    killed += 1;
                }
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

/// 지정 위치에 OrbitingBook 엔티티를 스폰한다.
pub fn spawn_book(
    world: &mut World,
    player_pos: Vec2,
    angle: f32,
    angular_speed: f32,
    radius: f32,
    damage: f32,
    hit_radius: f32,
    lifetime: f32,
    tick_cooldown: f32,
) {
    let e = world.spawn();
    let pos = player_pos + Vec2::new(angle.cos(), angle.sin()) * radius;
    world.add_component(
        e,
        Transform {
            position: pos,
            scale: Vec2::new(18.0, 24.0), // 책 모양
            rotation: 0.0,
            z: 0.6,
        },
    );
    add_tinted_sprite(world, e, SurvivorSprite::HolyBook, [0.9, 0.95, 1.0, 1.0]);
    world.add_component(
        e,
        OrbitingBook {
            angle,
            angular_speed,
            radius,
            damage,
            hit_radius,
            lifetime,
            tick_cooldown,
            tick_elapsed: 0.0,
        },
    );
}
