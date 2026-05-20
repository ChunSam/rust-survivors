use engine::{Collider, CollisionLayer, Entity, Sprite, System, Transform, World};
use glam::Vec2;

use super::player::Player;
use super::LAYER_XP;

/// 적이 떨어뜨리는 경험치 보석.
pub struct XpGem {
    pub value: u32,
}

/// Player 에 부착되는 경험치 누적기.
///
/// 1-D 에서는 누적만. 레벨업·임계치 처리는 1-E.
#[derive(Default)]
pub struct XpAccumulator {
    pub current: u32,
}

/// Magnet 흡수 + 픽업 처리 시스템.
pub struct MagnetSystem {
    pub pickup_radius:  f32, // 이 거리 안에 들어오면 픽업
    pub magnet_radius:  f32, // 이 거리 안에 들어오면 플레이어 쪽으로 끌어당김
    pub attract_speed:  f32, // 끌어당기는 속도 (px/s)
}

impl Default for MagnetSystem {
    fn default() -> Self {
        Self {
            pickup_radius: 20.0,
            magnet_radius: 80.0,
            attract_speed: 400.0,
        }
    }
}

impl System for MagnetSystem {
    fn run(&mut self, world: &mut World, dt: f32) {
        // 1) Player 위치 캐시 (없으면 종료)
        // borrow checker: query 결과를 즉시 collect/map 해서 드롭 후 get_mut 호출
        let Some(player_pos) = world
            .query2::<Player, Transform>()
            .next()
            .map(|(_, _, t)| t.position)
        else {
            return;
        };

        // 2) XpGem 들의 (entity, position, value) 수집
        let gems: Vec<(Entity, Vec2, u32)> = world
            .query2::<XpGem, Transform>()
            .map(|(e, gem, t)| (e, t.position, gem.value))
            .collect();

        // 3) 분류: 픽업할 것, 끌어당길 것
        let mut to_pickup: Vec<(Entity, u32)> = Vec::new();
        let mut to_attract: Vec<(Entity, Vec2)> = Vec::new();

        for (e, pos, value) in gems {
            let to_player = player_pos - pos;
            let dist = to_player.length();
            if dist <= self.pickup_radius {
                to_pickup.push((e, value));
            } else if dist <= self.magnet_radius {
                let dir = if dist > 0.0 { to_player / dist } else { Vec2::ZERO };
                let new_pos = pos + dir * self.attract_speed * dt;
                to_attract.push((e, new_pos));
            }
        }

        // 4) Player 의 XpAccumulator 컴포넌트 갱신
        // XpAccumulator 는 Player 컴포넌트로만 사용. 리소스로는 insert 하지 않음.
        if !to_pickup.is_empty() {
            let total: u32 = to_pickup.iter().map(|(_, v)| v).sum();
            let player_entity = world
                .query2::<Player, XpAccumulator>()
                .next()
                .map(|(e, _, _)| e);
            if let Some(pe) = player_entity {
                if let Some(acc) = world.get_mut::<XpAccumulator>(pe) {
                    acc.current += total;
                    println!("XP: {}", acc.current);
                }
            }
        }

        // 5) 픽업된 gem 은 despawn
        for (e, _) in &to_pickup {
            world.despawn(*e);
        }

        // 6) 끌어당길 gem 의 Transform 갱신
        for (e, new_pos) in to_attract {
            if let Some(t) = world.get_mut::<Transform>(e) {
                t.position = new_pos;
            }
        }
    }
}

/// XpGem 을 월드에 스폰한다.
///
/// WhipSystem 이 zombie 사망 직전 위치에서 호출.
pub fn spawn_xp_gem(world: &mut World, pos: Vec2, value: u32) {
    let e = world.spawn();
    world.add_component(e, Transform {
        position: pos,
        scale:    Vec2::new(12.0, 12.0),
        rotation: 0.0,
        z:        0.3, // zombie(0.5) 아래, player(1.0) 보다 낮음
    });
    world.add_component(e, Sprite::colored(0.4, 0.9, 1.0)); // 청록색
    world.add_component(e, XpGem { value });
    world.add_component(e, CollisionLayer(LAYER_XP));
    world.add_component(e, Collider::Circle { radius: 6.0 });
}
