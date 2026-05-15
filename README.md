# Rust 2D Game Engine

Rust로 처음부터 만드는 학습용 2D 게임 엔진.  
`wgpu` 렌더링 · 직접 구현한 ECS · `rapier2d` 물리 엔진을 단계적으로 쌓아올린다.

---

## 실행

```bash
# Rust 미설치 시
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# 플랫포머 데모 실행
cargo run -p game

# 엔진 단위 테스트
cargo test -p engine
```

**조작법** — `A` / `D` 또는 `←` / `→` 이동 · `Space` 점프 · `ESC` 종료

---

## 데모 화면

```
┌────────────────────────────────────┐
│                                    │
│        ■  ← 플레이어 (빨강)         │
│                                    │
│   ▬▬▬▬▬▬▬      ▬▬▬▬▬▬▬           │
│              ▬▬▬▬▬▬               │
│                                    │
│████████████████████████████████████│ ← 바닥
└────────────────────────────────────┘
```

- 중력 낙하, 플랫폼 착지, 점프 (착지 상태에서만)
- 1 물리 단위 = 50 픽셀 · 중력 9.8 m/s²

---

## 프로젝트 구조

```
Rust-GameEngine/
├── Cargo.toml                    # Cargo workspace
├── assets/
│   └── shaders/sprite.wgsl       # WGSL 스프라이트 셰이더
└── crates/
    ├── engine/                   # 엔진 라이브러리
    │   └── src/
    │       ├── app.rs            # 게임 루프 (winit ApplicationHandler)
    │       ├── components.rs     # Transform, Sprite
    │       ├── input.rs          # InputState 리소스
    │       ├── ecs/
    │       │   ├── world.rs      # World, Entity, 컴포넌트 저장소
    │       │   └── system.rs     # System trait
    │       ├── renderer/
    │       │   ├── context.rs    # wgpu Device / Queue / Surface
    │       │   ├── sprite.rs     # 인스턴스 렌더링 파이프라인
    │       │   └── texture.rs    # PNG 텍스처 로딩
    │       └── physics/
    │           └── system.rs     # PhysicsWorld, PhysicsBody, PhysicsSystem
    └── game/
        └── src/main.rs           # 플랫포머 데모
```

---

## 아키텍처

### ECS (Entity Component System)

외부 크레이트 없이 직접 구현. `TypeId` 기반 타입 소거로 컴포넌트를 저장한다.

```rust
let mut world = World::new();
let e = world.spawn();
world.add_component(e, Transform::new(Vec2::new(400.0, 300.0), Vec2::splat(64.0), 0.0));
world.add_component(e, Sprite::colored(0.9, 0.2, 0.2));

// 컴포넌트 쿼리
for (entity, sprite) in world.query::<Sprite>() {
    // ...
}
```

### 렌더링

wgpu 인스턴스 렌더링 — `Sprite` + `Transform` 을 가진 모든 엔티티를 한 번의 draw call로 처리한다.

```
카메라 유니폼 (직교 투영)
    └── 스프라이트 파이프라인
            ├── 버텍스 버퍼 (단위 쿼드)
            └── 인스턴스 버퍼 (모델 행렬 + 색상 × N개)
```

### 물리

rapier2d를 ECS와 연결. `PhysicsWorld` 를 시스템 구조체가 직접 소유해 borrow checker 문제를 피한다.

```rust
// 1. 동적 바디 생성
let (body, col) = physics.add_dynamic_box(position, half_w, half_h, true);

// 2. 매 프레임: 입력 → 속도 적용 → step → Transform 동기화
body.set_linvel(vector![move_x * speed, vel.y], true);
physics.step(dt);
```

### 시스템 흐름

```
EventLoop::run_app()
    │
    ├── about_to_wait()   → window.request_redraw()
    │
    └── RedrawRequested
            ├── InputState 갱신
            ├── System::run() × N      ← 등록된 시스템 순서대로 실행
            ├── wgpu Clear (배경색)
            └── SpriteRenderer::render()
```

---

## 의존 크레이트

| 크레이트 | 버전 | 역할 |
|---------|------|------|
| `wgpu` | 0.20 | GPU 렌더링 (Metal / Vulkan / DX12 추상화) |
| `winit` | 0.30 | 창 생성 · 이벤트 루프 |
| `rapier2d` | 0.22 | 2D 물리 시뮬레이션 |
| `glam` | 0.28 | 수학 (Vec2, Mat4) |
| `bytemuck` | 1.x | GPU 버퍼 캐스팅 |
| `image` | 0.25 | PNG 텍스처 로딩 |

---

## 커스텀 시스템 만들기

```rust
use engine::{App, InputState, System, Transform, World};

struct MySystem;

impl System for MySystem {
    fn run(&mut self, world: &mut World, dt: f32) {
        for (entity, _) in world.query::<MyTag>().collect::<Vec<_>>() {
            if let Some(t) = world.get_mut::<Transform>(entity) {
                t.position.x += 100.0 * dt;
            }
        }
    }
}

fn main() {
    let mut app = App::new();
    app.add_system(MySystem);
    app.run();
}
```

---

## 라이선스

MIT
