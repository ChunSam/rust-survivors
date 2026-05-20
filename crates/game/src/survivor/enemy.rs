use engine::{Entity, System, Transform, World};
use engine::components::GameState;
use glam::Vec2;
use super::player::Player;

/// 좀비 적 태그 컴포넌트
pub struct Zombie;

/// 적 AI 파라미터
pub struct EnemyAi {
    pub move_speed: f32,
}

impl Default for EnemyAi {
    fn default() -> Self {
        Self { move_speed: 80.0 } // 플레이어(200)보다 느림
    }
}

/// 좀비가 플레이어를 향해 이동하는 시스템
pub struct EnemyAiSystem;

impl System for EnemyAiSystem {
    fn run(&mut self, world: &mut World, dt: f32) {
        if !matches!(world.resource::<GameState>(), Some(GameState::Playing)) {
            return;
        }

        // 1) 플레이어 위치 캐시 — borrow 를 여기서 끝냄
        let Some(player_pos) = world
            .query2::<Player, Transform>()
            .next()
            .map(|(_, _, t)| t.position)
        else {
            return;
        };

        // 2) Zombie + Transform + EnemyAi 를 가진 엔티티 목록 수집 (불변 query 종료)
        let targets: Vec<(Entity, Vec2, f32)> = world
            .query3::<Zombie, Transform, EnemyAi>()
            .map(|(e, _, t, ai)| (e, t.position, ai.move_speed))
            .collect();

        // 3) Transform 갱신 — query 가 끝난 뒤 get_mut 호출 (borrow checker 회피)
        for (entity, pos, speed) in targets {
            let dir = player_pos - pos;
            let dir = if dir.length_squared() > 0.0 { dir.normalize() } else { Vec2::ZERO };
            if let Some(t) = world.get_mut::<Transform>(entity) {
                t.position += dir * speed * dt;
            }
        }
    }
}
