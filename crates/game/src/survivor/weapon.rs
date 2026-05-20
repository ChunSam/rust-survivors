use engine::{CollisionLayer, Entity, SpatialGrid, Sprite, System, Transform, World};
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

/// 히트플래시 컴포넌트. 피격된 엔티티에 일시적으로 부착.
///
/// `remaining > 0`: 아직 빨간 색 유지 중.
/// `remaining <= 0` (sentinel `f32::NEG_INFINITY`): 이미 원래 색으로 복원 완료 — skip.
///
/// `World::remove_component` 가 없으므로, 복원 완료 후 `remaining = f32::NEG_INFINITY` 로
/// sentinel 처리하여 이후 프레임에서 재처리되지 않도록 한다.
pub struct HitFlash {
    pub remaining: f32,
    pub original:  [f32; 4],
}

/// 히트플래시 갱신 시스템. WhipSystem 다음에 등록.
pub struct HitFlashSystem;

impl System for HitFlashSystem {
    fn run(&mut self, world: &mut World, dt: f32) {
        // 두 단계 패턴: 먼저 수집 → 그 다음 get_mut (borrow checker 충돌 회피)
        let flashes: Vec<(Entity, f32, [f32; 4])> = world
            .query::<HitFlash>()
            .map(|(e, f)| (e, f.remaining, f.original))
            .collect();

        for (entity, remaining, original) in flashes {
            // sentinel: 이미 복원 완료
            if remaining <= 0.0 {
                continue;
            }

            let new_remaining = remaining - dt;
            if new_remaining > 0.0 {
                // 아직 플래시 유지 — remaining 만 감소
                if let Some(f) = world.get_mut::<HitFlash>(entity) {
                    f.remaining = new_remaining;
                }
            } else {
                // 만료 — 원래 색으로 복원하고 sentinel 설정
                if let Some(s) = world.get_mut::<Sprite>(entity) {
                    s.color = original;
                }
                if let Some(f) = world.get_mut::<HitFlash>(entity) {
                    f.remaining = f32::NEG_INFINITY; // sentinel: 처리 완료
                }
            }
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

        let mut killed = 0usize;
        let mut hit_actual = 0usize;

        for entity in hits {
            // 1) 현재 sprite 색 캐시 + 빨강으로 변경
            let original = if let Some(s) = world.get_mut::<Sprite>(entity) {
                let orig = s.color;
                s.color = [1.0, 0.3, 0.3, 1.0];
                orig
            } else {
                continue;
            };

            // 2) Health 차감 + 사망 처리
            let died = if let Some(h) = world.get_mut::<Health>(entity) {
                h.take_damage(damage)
            } else {
                false
            };

            hit_actual += 1;
            if died {
                killed += 1;
                // despawn 전에 위치를 캐치해 XpGem 을 드롭
                let drop_pos = world.get::<Transform>(entity).map(|t| t.position);
                world.despawn(entity);
                if let Some(p) = drop_pos {
                    super::xp::spawn_xp_gem(world, p, 1);
                }
            } else {
                // 살아있으면 HitFlash 부착 (덮어쓰기 — 기존 있어도 갱신)
                world.add_component(entity, HitFlash { remaining: 0.1, original });
            }
        }

        if hit_actual > 0 {
            println!("Whip: {} hits, {} killed", hit_actual, killed);
        }
    }
}
