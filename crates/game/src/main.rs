use engine::{
    AnimationSystem, App, AudioManager, BodyHandle, ColliderHandle, Entity, GameState, InputState,
    KeyCode, PhysicsBody, Sprite, System, Transform, World,
};
use glam::Vec2;
use rapier2d::prelude::*;

// ─── 좌표 스케일 ──────────────────────────────────────────────────────────────
//  물리 엔진은 SI 단위(미터)를 사용하고 화면은 픽셀을 사용한다.
//  1 물리 단위 = SCALE 픽셀로 변환한다.
const SCALE: f32 = 50.0;

/// 픽셀 → 물리 단위
fn p(px: f32) -> f32 {
    px / SCALE
}

// ─── 플레이어 태그 ────────────────────────────────────────────────────────────
struct Player;

// ─── 플랫포머 시스템 ──────────────────────────────────────────────────────────
/// 입력 처리 + 물리 step + Transform 동기화를 하나의 시스템에서 담당한다.
///
/// `PhysicsWorld`를 직접 소유해 borrow checker 문제를 피한다.
/// (PhysicsWorld를 ECS 리소스로 넣으면 &mut World 와 동시 대여 충돌이 생긴다.)
struct PlatformerSystem {
    physics: engine::PhysicsWorld,
    player_body: BodyHandle,
    player_col: ColliderHandle,
    player_entity: Entity, // 낙하 감지와 재시작 시 Transform 리셋에 사용
    speed: f32,            // 수평 이동 속도 (물리 단위/s)
    jump_force: f32,       // 점프 충격량 (N·s)
}

impl System for PlatformerSystem {
    fn run(&mut self, world: &mut World, dt: f32) {
        // ── 0. 게임 상태 확인 ──────────────────────────────────────────────
        let state = world
            .resource::<GameState>()
            .cloned()
            .unwrap_or(GameState::Playing);

        if state == GameState::GameOver {
            // R 키로 재시작
            let restart = world
                .resource::<InputState>()
                .map(|i| i.just_pressed(KeyCode::KeyR))
                .unwrap_or(false);
            if restart {
                // 물리 바디를 초기 위치로 리셋
                if let Some(body) = self.physics.rigid_body_mut(self.player_body) {
                    body.set_translation(vector![p(400.0), p(100.0)], true);
                    body.set_linvel(vector![0.0, 0.0], true);
                }
                // Transform도 즉시 반영
                if let Some(tr) = world.get_mut::<Transform>(self.player_entity) {
                    tr.position = Vec2::new(400.0, 100.0);
                }
                if let Some(gs) = world.resource_mut::<GameState>() {
                    *gs = GameState::Playing;
                }
                println!("재시작!");
            }
            return;
        }

        // ── 1. 입력 읽기 ───────────────────────────────────────────────────
        let (move_x, do_jump) = {
            let input = world.resource::<InputState>().unwrap();
            let left = input.is_pressed(KeyCode::KeyA) || input.is_pressed(KeyCode::ArrowLeft);
            let right = input.is_pressed(KeyCode::KeyD) || input.is_pressed(KeyCode::ArrowRight);
            let jump = input.just_pressed(KeyCode::Space);
            (right as i32 as f32 - left as i32 as f32, jump)
        };

        // ── 2. 플레이어 속도·점프 적용 ────────────────────────────────────
        let grounded = self.physics.has_contact(self.player_col);

        if let Some(body) = self.physics.rigid_body_mut(self.player_body) {
            let vel = *body.linvel();
            body.set_linvel(vector![move_x * self.speed, vel.y], true);
            if do_jump && grounded {
                body.apply_impulse(vector![0.0, -self.jump_force], true);
                // 점프 효과음 (assets/audio/jump.wav 가 있을 때만 재생됨)
                if let Some(audio) = world.resource_mut::<AudioManager>() {
                    audio.play("sfx_jump", "assets/audio/jump.wav", false);
                }
            }
        }

        // ── 3. 물리 시뮬레이션 진행 ───────────────────────────────────────
        self.physics.step(dt);

        // ── 4. 물리 결과 → ECS Transform 동기화 ──────────────────────────
        //  (entity, handle) 쌍을 먼저 수집해야 world를 다시 빌릴 수 있다
        let handles: Vec<(Entity, BodyHandle)> = world
            .query::<PhysicsBody>()
            .map(|(e, b)| (e, b.rigid_body_handle))
            .collect();

        for (entity, handle) in handles {
            if let Some(body) = self.physics.rigid_body(handle) {
                let t = *body.translation();
                if let Some(tr) = world.get_mut::<Transform>(entity) {
                    tr.position = Vec2::new(t.x * SCALE, t.y * SCALE);
                }
            }
        }

        // ── 5. 낙하 감지 → 게임 오버 ─────────────────────────────────────
        let fell = world
            .get::<Transform>(self.player_entity)
            .map(|tr| tr.position.y > 700.0)
            .unwrap_or(false);
        if fell {
            println!("게임 오버! R 키로 재시작하세요.");
            if let Some(gs) = world.resource_mut::<GameState>() {
                *gs = GameState::GameOver;
            }
        }
    }
}

// ─── 편의 함수: 정적 오브젝트(바닥·플랫폼) 생성 ─────────────────────────────
#[allow(clippy::too_many_arguments)]
fn spawn_static(
    world: &mut World,
    physics: &mut engine::PhysicsWorld,
    px: f32,
    py: f32, // 중심 위치 (픽셀)
    sw: f32,
    sh: f32, // 크기 (픽셀)
    r: f32,
    g: f32,
    b: f32,
) {
    let (rb, col) = physics.add_static_box(Vec2::new(p(px), p(py)), p(sw * 0.5), p(sh * 0.5));
    let e = world.spawn();
    world.add_component(e, Transform::new(Vec2::new(px, py), Vec2::new(sw, sh), 0.0));
    world.add_component(e, Sprite::colored(r, g, b));
    world.add_component(
        e,
        PhysicsBody {
            rigid_body_handle: rb,
            collider_handle: col,
        },
    );
}

// ─── 메인 ─────────────────────────────────────────────────────────────────────
fn main() {
    env_logger::init();

    let mut app = App::new();

    // 물리 세계 생성 (중력: Y+ 아래, 9.8 m/s²)
    let mut physics = engine::PhysicsWorld::new(Vec2::new(0.0, 9.8));

    // ── 지형 (정적 바디) ────────────────────────────────────────────────────
    //   바닥 왼쪽 (x: 0~350)
    spawn_static(
        &mut app.world,
        &mut physics,
        175.0,
        572.0,
        350.0,
        40.0,
        0.28,
        0.62,
        0.22,
    );
    //   바닥 오른쪽 (x: 490~800) — 350~490 구간이 낙하 구멍
    spawn_static(
        &mut app.world,
        &mut physics,
        645.0,
        572.0,
        310.0,
        40.0,
        0.28,
        0.62,
        0.22,
    );

    //   플랫폼 1 (왼쪽 아래)
    spawn_static(
        &mut app.world,
        &mut physics,
        200.0,
        440.0,
        220.0,
        24.0,
        0.55,
        0.38,
        0.18,
    );

    //   플랫폼 2 (오른쪽 중간)
    spawn_static(
        &mut app.world,
        &mut physics,
        580.0,
        320.0,
        200.0,
        24.0,
        0.55,
        0.38,
        0.18,
    );

    //   플랫폼 3 (왼쪽 위)
    spawn_static(
        &mut app.world,
        &mut physics,
        160.0,
        200.0,
        160.0,
        24.0,
        0.55,
        0.38,
        0.18,
    );

    //   오른쪽 벽 (화면 밖으로 떨어지지 않도록)
    spawn_static(
        &mut app.world,
        &mut physics,
        800.0,
        300.0,
        20.0,
        600.0,
        0.2,
        0.2,
        0.2,
    );

    //   왼쪽 벽
    spawn_static(
        &mut app.world,
        &mut physics,
        0.0,
        300.0,
        20.0,
        600.0,
        0.2,
        0.2,
        0.2,
    );

    // ── 플레이어 (동적 바디) ────────────────────────────────────────────────
    let sprite_size = 36.0_f32;
    let half = p(sprite_size * 0.5 - 1.0); // 콜라이더를 스프라이트보다 1px 작게
    let (player_body, player_col) = physics.add_dynamic_box(
        Vec2::new(p(200.0), p(100.0)),
        half,
        half,
        true, // 회전 고정
    );
    let player_e = app.world.spawn();
    app.world.add_component(
        player_e,
        Transform::new(Vec2::new(200.0, 100.0), Vec2::splat(sprite_size), 0.0),
    );
    app.world
        .add_component(player_e, Sprite::colored(0.92, 0.22, 0.22));
    app.world.add_component(
        player_e,
        PhysicsBody {
            rigid_body_handle: player_body,
            collider_handle: player_col,
        },
    );
    app.world.add_component(player_e, Player);

    // ── 오디오 초기화 (장치가 없어도 게임은 실행됨) ────────────────────────
    if let Some(audio) = AudioManager::new() {
        // BGM 예시: audio.play("bgm", "assets/audio/bgm.wav", true);
        app.world.insert_resource(audio);
    }

    // ── 시스템 등록 ─────────────────────────────────────────────────────────
    // AnimationSystem은 AnimationPlayer 컴포넌트가 없으면 아무것도 하지 않는다.
    app.add_system(AnimationSystem::new());
    app.add_system(PlatformerSystem {
        physics,
        player_body,
        player_col,
        player_entity: player_e,
        speed: 5.5,      // m/s  → 275 px/s 수평 이동
        jump_force: 3.8, // N·s  → 착지에서 약 3.7m(185px) 도달
    });

    println!("A / ← → D : 이동    Space : 점프    ESC : 종료");
    println!("플레이어가 떨어지면 게임 오버 → R 키로 재시작");
    app.run();
}
