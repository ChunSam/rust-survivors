use engine::{CollisionLayer, Entity, SpatialGrid, System, Transform, World};
use glam::Vec2;

use super::health::Health;
use super::player::Player;
use super::LAYER_ENEMY;

/// Whip 무기 컴포넌트. 플레이어에 부착.
pub struct Whip {
    pub level:       u8,
    pub damage:      f32,
    pub area_width:  f32,
    pub area_height: f32,
    pub cooldown:    f32,
}

impl Default for Whip {
    fn default() -> Self {
        Self {
            level:       1,
            damage:      10.0,
            area_width:  120.0,
            area_height: 60.0,
            cooldown:    1.0,
        }
    }
}

/// Whip 발화 시스템. SpatialGrid 를 직접 소유 (PhysicsSystem 패턴).
pub struct WhipSystem {
    pub grid:    SpatialGrid,
    pub elapsed: f32,
}

impl Default for WhipSystem {
    fn default() -> Self {
        Self {
            grid:    SpatialGrid::new(128.0),
            elapsed: 0.0,
        }
    }
}

impl System for WhipSystem {
    fn run(&mut self, world: &mut World, dt: f32) {
        self.elapsed += dt;

        // Player + Transform + Whip 을 동시에 가진 엔티티에서 위치와 Whip 수치를 캐시
        let player_data: Option<(Vec2, f32, f32, f32, f32)> = world
            .query3::<Player, Transform, Whip>()
            .next()
            .map(|(_, _, t, w)| (t.position, w.damage, w.area_width, w.area_height, w.cooldown));

        let (player_pos, damage, area_width, area_height, cooldown) = match player_data {
            Some(d) => d,
            None => return,
        };

        if self.elapsed < cooldown {
            return;
        }
        self.elapsed -= cooldown;

        // 발화 시점에만 grid 갱신 — 매 프레임 rebuild 비용 절감
        self.grid.rebuild(world);

        let half_h = area_height * 0.5;
        let mask = CollisionLayer(LAYER_ENEMY);

        // 좌측 AABB
        let left_min = player_pos + Vec2::new(-area_width, -half_h);
        let left_max = player_pos + Vec2::new(0.0, half_h);

        // 우측 AABB
        let right_min = player_pos + Vec2::new(0.0, -half_h);
        let right_max = player_pos + Vec2::new(area_width, half_h);

        let mut hits: Vec<Entity> = self
            .grid
            .query_aabb(left_min, left_max, mask)
            .into_iter()
            .chain(self.grid.query_aabb(right_min, right_max, mask))
            .collect();

        // 좌/우 양쪽에 걸쳐도 한 번만 처리
        hits.sort_by_key(|e| e.0);
        hits.dedup();

        let mut to_kill: Vec<Entity> = Vec::new();
        for entity in hits {
            if let Some(h) = world.get_mut::<Health>(entity) {
                if h.take_damage(damage) {
                    to_kill.push(entity);
                }
            }
        }

        for e in to_kill {
            world.despawn(e);
        }
    }
}
