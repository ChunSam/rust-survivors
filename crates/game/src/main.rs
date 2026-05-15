use engine::{App, Entity, InputState, PhysicsBody, Sprite, System, Transform, World};
use glam::Vec2;
use rapier2d::prelude::*;
use winit::keyboard::KeyCode;

// ─── 좌표 스케일 ──────────────────────────────────────────────────────────────
//  물리 엔진은 SI 단위(미터)를 사용하고 화면은 픽셀을 사용한다.
//  1 물리 단위 = SCALE 픽셀로 변환한다.
const SCALE: f32 = 50.0;

/// 픽셀 → 물리 단위
fn p(px: f32) -> f32 { px / SCALE }

// ─── 플레이어 태그 ────────────────────────────────────────────────────────────
struct Player;

// ─── 플랫포머 시스템 ──────────────────────────────────────────────────────────
/// 입력 처리 + 물리 step + Transform 동기화를 하나의 시스템에서 담당한다.
///
/// `PhysicsWorld`를 직접 소유해 borrow checker 문제를 피한다.
/// (PhysicsWorld를 ECS 리소스로 넣으면 &mut World 와 동시 대여 충돌이 생긴다.)
struct PlatformerSystem {
    physics:     engine::PhysicsWorld,
    player_body: RigidBodyHandle,
    player_col:  ColliderHandle,
    speed:       f32,   // 수평 이동 속도 (물리 단위/s)
    jump_force:  f32,   // 점프 충격량 (N·s)
}

impl System for PlatformerSystem {
    fn run(&mut self, world: &mut World, dt: f32) {
        // ── 1. 입력 읽기 ───────────────────────────────────────────────────
        let (move_x, do_jump) = {
            let input = world.resource::<InputState>().unwrap();
            let left  = input.is_pressed(KeyCode::KeyA) || input.is_pressed(KeyCode::ArrowLeft);
            let right = input.is_pressed(KeyCode::KeyD) || input.is_pressed(KeyCode::ArrowRight);
            let jump  = input.just_pressed(KeyCode::Space);
            (right as i32 as f32 - left as i32 as f32, jump)
        };

        // ── 2. 플레이어 속도·점프 적용 ────────────────────────────────────
        //  is_grounded: 플레이어 콜라이더가 다른 바디와 접촉 중이면 착지 상태
        let grounded = self.physics.has_contact(self.player_col);

        if let Some(body) = self.physics.rigid_body_set.get_mut(self.player_body) {
            let vel = *body.linvel();
            // 수평 속도를 입력에 따라 직접 설정 (수직은 유지)
            body.set_linvel(vector![move_x * self.speed, vel.y], true);
            // 착지 상태에서만 점프 허용
            if do_jump && grounded {
                body.apply_impulse(vector![0.0, -self.jump_force], true);
            }
        }

        // ── 3. 물리 시뮬레이션 진행 ───────────────────────────────────────
        self.physics.step(dt);

        // ── 4. 물리 결과 → ECS Transform 동기화 ──────────────────────────
        //  (entity, handle) 쌍을 먼저 수집해야 world를 다시 빌릴 수 있다
        let handles: Vec<(Entity, RigidBodyHandle)> = world
            .query::<PhysicsBody>()
            .map(|(e, b)| (e, b.rigid_body_handle))
            .collect();

        for (entity, handle) in handles {
            if let Some(body) = self.physics.rigid_body_set.get(handle) {
                let t = *body.translation();
                if let Some(tr) = world.get_mut::<Transform>(entity) {
                    tr.position = Vec2::new(t.x * SCALE, t.y * SCALE);
                }
            }
        }
    }
}

// ─── 편의 함수: 정적 오브젝트(바닥·플랫폼) 생성 ─────────────────────────────
fn spawn_static(
    world:   &mut World,
    physics: &mut engine::PhysicsWorld,
    px: f32, py: f32,   // 중심 위치 (픽셀)
    sw: f32, sh: f32,   // 크기 (픽셀)
    r: f32, g: f32, b: f32,
) {
    let (rb, col) = physics.add_static_box(
        Vec2::new(p(px), p(py)),
        p(sw * 0.5),
        p(sh * 0.5),
    );
    let e = world.spawn();
    world.add_component(e, Transform::new(Vec2::new(px, py), Vec2::new(sw, sh), 0.0));
    world.add_component(e, Sprite::colored(r, g, b));
    world.add_component(e, PhysicsBody { rigid_body_handle: rb, collider_handle: col });
}

// ─── 메인 ─────────────────────────────────────────────────────────────────────
fn main() {
    env_logger::init();

    let mut app = App::new();

    // 물리 세계 생성 (중력: Y+ 아래, 9.8 m/s²)
    let mut physics = engine::PhysicsWorld::new(Vec2::new(0.0, 9.8));

    // ── 지형 (정적 바디) ────────────────────────────────────────────────────
    //   바닥
    spawn_static(&mut app.world, &mut physics,
        400.0, 572.0, 800.0, 40.0,
        0.28, 0.62, 0.22);

    //   플랫폼 1 (왼쪽 아래)
    spawn_static(&mut app.world, &mut physics,
        200.0, 440.0, 220.0, 24.0,
        0.55, 0.38, 0.18);

    //   플랫폼 2 (오른쪽 중간)
    spawn_static(&mut app.world, &mut physics,
        580.0, 320.0, 200.0, 24.0,
        0.55, 0.38, 0.18);

    //   플랫폼 3 (왼쪽 위)
    spawn_static(&mut app.world, &mut physics,
        160.0, 200.0, 160.0, 24.0,
        0.55, 0.38, 0.18);

    //   오른쪽 벽 (화면 밖으로 떨어지지 않도록)
    spawn_static(&mut app.world, &mut physics,
        800.0, 300.0, 20.0, 600.0,
        0.2, 0.2, 0.2);

    //   왼쪽 벽
    spawn_static(&mut app.world, &mut physics,
        0.0, 300.0, 20.0, 600.0,
        0.2, 0.2, 0.2);

    // ── 플레이어 (동적 바디) ────────────────────────────────────────────────
    let sprite_size = 36.0_f32;
    let half = p(sprite_size * 0.5 - 1.0); // 콜라이더를 스프라이트보다 1px 작게
    let (player_body, player_col) = physics.add_dynamic_box(
        Vec2::new(p(400.0), p(100.0)),
        half, half,
        true, // 회전 고정
    );
    let player_e = app.world.spawn();
    app.world.add_component(player_e, Transform::new(
        Vec2::new(400.0, 100.0),
        Vec2::splat(sprite_size),
        0.0,
    ));
    app.world.add_component(player_e, Sprite::colored(0.92, 0.22, 0.22));
    app.world.add_component(player_e, PhysicsBody {
        rigid_body_handle: player_body,
        collider_handle:   player_col,
    });
    app.world.add_component(player_e, Player);

    // ── 시스템 등록 ─────────────────────────────────────────────────────────
    app.add_system(PlatformerSystem {
        physics,
        player_body,
        player_col,
        speed:      5.5,   // m/s  → 275 px/s 수평 이동
        jump_force: 3.8,   // N·s  → 착지에서 약 3.7m(185px) 도달
    });

    println!("A / ← → D : 이동    Space : 점프    ESC : 종료");
    app.run();
}
