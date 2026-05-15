# CLAUDE.md — Rust 2D Game Engine

## 프로젝트 개요

학습 목적으로 Rust로 처음부터 만드는 2D 게임 엔진.
wgpu 렌더링 · 직접 구현한 ECS · rapier2d 물리를 단계적으로 구축했다.

## 크레이트 구조

```
Rust-GameEngine/          ← Cargo workspace
├── crates/engine/        ← 엔진 라이브러리 (lib)
└── crates/game/          ← 플랫포머 데모 (bin)
```

## 주요 명령어

```bash
cargo run -p game              # 데모 실행 (debug)
cargo run -p game --release    # 데모 실행 (최적화)
cargo test -p engine           # 엔진 단위 테스트
cargo build --release          # 릴리즈 빌드 (~28초 풀빌드, 캐시 시 0.3초)
```

데모 조작: `A/D` 또는 `←/→` 이동 · `Space` 점프 · `ESC` 종료

## 의존 크레이트

| 크레이트 | 버전 | 역할 |
|---------|------|------|
| `wgpu` | 0.20 | GPU 렌더링 |
| `winit` | 0.30 | 창·이벤트 루프 |
| `rapier2d` | 0.22 | 2D 물리 |
| `glam` | 0.28 | 수학 (Vec2, Mat4) |
| `bytemuck` | 1.x | GPU 버퍼 캐스팅 |
| `image` | 0.25 | PNG 텍스처 로딩 |
| `pollster` | 0.3 | wgpu async 초기화 |

## 아키텍처

### ECS (`crates/engine/src/ecs/`)

외부 크레이트 없이 직접 구현. `TypeId` + `Box<dyn Any>` 타입 소거로 컴포넌트 저장.

- `World` — 엔티티·컴포넌트·리소스 저장소 (`world.rs`)
- `Entity` — `u32` newtype
- `System` trait — `fn run(&mut self, world: &mut World, dt: f32)` (`system.rs`)
- 컴포넌트: `HashMap<TypeId, Vec<Option<Box<dyn Any>>>>`  (entity ID로 인덱싱)
- 리소스: `HashMap<TypeId, Box<dyn Any>>` (전역 싱글턴)

```rust
let e = world.spawn();
world.add_component(e, Transform::default());
for (entity, sprite) in world.query::<Sprite>() { ... }
world.resource::<InputState>()
```

### 렌더링 (`crates/engine/src/renderer/`)

- `GpuContext` (`context.rs`) — wgpu Surface/Device/Queue/Config 초기화·관리
- `SpriteRenderer` (`sprite.rs`) — 인스턴스 렌더링. `Transform + Sprite` 엔티티를 한 draw call로 처리
- `Texture` (`texture.rs`) — PNG 로딩, 기본 1×1 흰색 텍스처
- 셰이더: `assets/shaders/sprite.wgsl` (WGSL, 런타임 파일 로드)
- 카메라: 직교 투영 `(0,0)` = 좌상단, `(width,height)` = 우하단 (픽셀 좌표계)

### 입력 (`crates/engine/src/input.rs`)

`InputState` 리소스를 ECS World에 삽입. winit 이벤트 → `press/release/flush`.

```rust
input.is_pressed(KeyCode::KeyA)    // 누르고 있는 동안
input.just_pressed(KeyCode::Space) // 누른 순간만 (1프레임)
```

### 물리 (`crates/engine/src/physics/system.rs`)

- `PhysicsWorld` — rapier2d 래퍼. `add_dynamic_box`, `add_static_box`, `step`, `has_contact`
- `PhysicsBody` — 엔티티에 붙이는 컴포넌트 (`RigidBodyHandle` + `ColliderHandle`)
- `PhysicsSystem` — 범용 step + Transform 동기화 시스템

**좌표 스케일**: `SCALE = 50.0` (1 물리 단위 = 50 픽셀). 픽셀↔물리 변환 필수.

```rust
// 픽셀 → 물리 단위
physics.add_dynamic_box(Vec2::new(px / 50.0, py / 50.0), hw / 50.0, hh / 50.0, true);
// 물리 → 픽셀 (Transform 동기화 시)
transform.position = Vec2::new(t.x * 50.0, t.y * 50.0);
```

**borrow checker 주의**: `PhysicsWorld`를 ECS 리소스로 넣으면 `&mut World`와 동시 대여 충돌.
→ `PhysicsSystem` 또는 커스텀 시스템 구조체가 직접 소유한다.

### 게임 루프 (`crates/engine/src/app.rs`)

winit `ApplicationHandler` 구현.

```
resumed()          → 창·GpuContext·SpriteRenderer 초기화
about_to_wait()    → window.request_redraw() (매 프레임)
RedrawRequested    → update(dt) → render()
  update(dt)       → System::run() × N → InputState::flush()
  render()         → Clear → SpriteRenderer::render()
```

`dt` 는 최대 0.1초로 클램프 (스파이크 방지).

## 주요 설계 결정 및 배경

| 결정 | 이유 |
|------|------|
| ECS 직접 구현 | 학습 목적 — `hecs`, `bevy_ecs` 대신 TypeId/Any 원리 이해 |
| `PhysicsWorld`를 시스템이 소유 | borrow checker: ECS 리소스로 넣으면 `&mut World` 충돌 |
| SCALE=50 좌표계 분리 | rapier2d SI 단위 안정성. 픽셀 직접 사용 시 질량·임펄스 값이 비정상적으로 커짐 |
| wgpu `entry_point: &str` | wgpu 0.20은 `Option<&str>` 아닌 `&str`. 0.21+에서 변경됨 |
| `has_any_active_contact` 필드 | rapier2d 0.22에서 메서드가 아닌 필드 |
| `cargo clean` 대신 `rm -rf target` | macOS `._CACHEDIR.TAG` 메타파일로 인해 `cargo clean` 실패 |

## 알려진 이슈 / 확장 포인트

- [ ] 텍스처 로딩 (`Texture::from_path`) 구현됐으나 `SpriteRenderer`에서 미연결 (단색만 동작)
- [ ] 씬 전환 / 게임 오버 로직 없음
- [ ] 오디오 시스템 미구현
- [ ] 스프라이트 애니메이션 미구현
- [ ] `cargo clean` macOS 버그 → `rm -rf target` 사용

## Git / GitHub

- 저장소: https://github.com/ChunSam/rust-game-engine
- 최신 릴리즈: [v0.1.0](https://github.com/ChunSam/rust-game-engine/releases/tag/v0.1.0)
- 브랜치: `main`

## 사용자 정보

- Rust 초급자 — borrow checker, lifetime 설명 시 이유를 한 줄 덧붙일 것
- 학습 목적 우선 — 성능보다 코드 가독성·원리 이해를 중시
