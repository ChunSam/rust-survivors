use engine::{Entity, InputState, System, Transform, World};
use engine::components::GameState;
use glam::Vec2;
use winit::keyboard::KeyCode;

/// 플레이어 태그 컴포넌트
pub struct Player;

/// 이동 속도(픽셀/초). System 이 매 프레임 Transform 에 누적.
pub struct Velocity(pub Vec2);

/// 캐릭터 스탯. Phase 1-A 에서는 이동 속도만, 이후 확장.
#[derive(Debug, Clone, Copy)]
pub struct PlayerStats {
    pub move_speed: f32,
}

impl Default for PlayerStats {
    fn default() -> Self {
        Self { move_speed: 200.0 }
    }
}

/// WASD/방향키 → Velocity → Transform.position 갱신.
pub struct PlayerMovementSystem;

impl System for PlayerMovementSystem {
    fn run(&mut self, world: &mut World, dt: f32) {
        if !matches!(world.resource::<GameState>(), Some(GameState::Playing)) {
            return;
        }

        // 1. 입력 읽기 — 정규화된 방향 벡터 계산
        let mut dir = Vec2::ZERO;
        if let Some(input) = world.resource::<InputState>() {
            if input.is_pressed(KeyCode::KeyA) || input.is_pressed(KeyCode::ArrowLeft) {
                dir.x -= 1.0;
            }
            if input.is_pressed(KeyCode::KeyD) || input.is_pressed(KeyCode::ArrowRight) {
                dir.x += 1.0;
            }
            if input.is_pressed(KeyCode::KeyW) || input.is_pressed(KeyCode::ArrowUp) {
                dir.y -= 1.0; // y 작을수록 위
            }
            if input.is_pressed(KeyCode::KeyS) || input.is_pressed(KeyCode::ArrowDown) {
                dir.y += 1.0;
            }
        }
        let dir = if dir.length_squared() > 0.0 {
            dir.normalize()
        } else {
            Vec2::ZERO
        };

        // 2. Player + PlayerStats + Transform 모두 가진 엔티티에 적용
        // 두 단계로 분리(query 결과를 모은 뒤 mut 접근)해서 borrow checker 회피
        let targets: Vec<(Entity, f32)> = world
            .query2::<Player, PlayerStats>()
            .map(|(e, _, stats)| (e, stats.move_speed))
            .collect();

        for (entity, speed) in targets {
            if let Some(t) = world.get_mut::<Transform>(entity) {
                t.position += dir * speed * dt;
            }
        }
    }
}
