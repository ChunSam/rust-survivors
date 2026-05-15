# Rust 2D 게임 엔진 학습 가이드

이 문서는 이 엔진의 소스 코드를 읽으며 **Rust + 게임 엔진 설계**를 배우려는 학습자를 위한 가이드입니다.
코드를 직접 읽고, 수정하고, 실행해 보면서 따라가세요.

---

## 목차

1. [프로젝트 전체 구조 보기](#1-프로젝트-전체-구조-보기)
2. [게임 루프 — 엔진이 돌아가는 원리](#2-게임-루프--엔진이-돌아가는-원리)
3. [ECS — 게임 오브젝트를 다루는 방식](#3-ecs--게임-오브젝트를-다루는-방식)
4. [컴포넌트 — 데이터 설계](#4-컴포넌트--데이터-설계)
5. [입력 시스템 — 키보드 상태 추적](#5-입력-시스템--키보드-상태-추적)
6. [렌더링 — GPU에 그림 그리기](#6-렌더링--gpu에-그림-그리기)
7. [WGSL 셰이더 — GPU 코드 읽기](#7-wgsl-셰이더--gpu-코드-읽기)
8. [물리 시스템 — rapier2d 연동](#8-물리-시스템--rapier2d-연동)
9. [애니메이션 시스템 — 스프라이트시트 활용](#9-애니메이션-시스템--스프라이트시트-활용)
10. [오디오 시스템 — 사운드 재생](#10-오디오-시스템--사운드-재생)
11. [게임 샘플 완독 — 모든 것을 합치기](#11-게임-샘플-완독--모든-것을-합치기)
12. [실습 과제](#12-실습-과제)

---

## 1. 프로젝트 전체 구조 보기

### 디렉토리 한눈에 보기

```
Rust-GameEngine/
├── Cargo.toml                    ← 워크스페이스 설정 (두 crate 묶기)
├── assets/
│   └── shaders/sprite.wgsl      ← GPU에서 실행되는 셰이더 코드
├── crates/
│   ├── engine/                  ← 엔진 라이브러리 (핵심)
│   │   └── src/
│   │       ├── lib.rs           ← 외부에 공개할 API 정의
│   │       ├── app.rs           ← 메인 루프
│   │       ├── components.rs    ← 게임 데이터 (Transform, Sprite 등)
│   │       ├── input.rs         ← 키보드 입력 상태
│   │       ├── animation.rs     ← 애니메이션 시스템
│   │       ├── audio.rs         ← 오디오 재생
│   │       ├── ecs/             ← Entity-Component-System
│   │       │   ├── world.rs     ← World, Entity (저장소)
│   │       │   └── system.rs    ← System trait (로직 인터페이스)
│   │       ├── renderer/        ← GPU 렌더링
│   │       │   ├── context.rs   ← wgpu 초기화
│   │       │   ├── texture.rs   ← PNG → GPU 텍스처
│   │       │   └── sprite.rs    ← 스프라이트 일괄 렌더링
│   │       └── physics/
│   │           └── system.rs    ← rapier2d 물리 래퍼
│   └── game/
│       └── src/main.rs          ← 플랫포머 샘플 게임
```

### 코드 읽는 순서 추천

초보자라면 아래 순서로 읽는 것이 가장 이해하기 쉽습니다.

```
lib.rs → components.rs → ecs/world.rs → ecs/system.rs
→ input.rs → app.rs → renderer/sprite.rs → animation.rs
→ audio.rs → physics/system.rs → main.rs → sprite.wgsl
```

---

## 2. 게임 루프 — 엔진이 돌아가는 원리

> 읽을 파일: `crates/engine/src/app.rs`

### 게임 루프란?

게임은 매 프레임마다 같은 과정을 반복합니다.

```
┌─────────────────────────────────────────────────┐
│  이벤트 처리 → 상태 업데이트(update) → 화면 그리기(render)  │
│  ↑                                           │  │
│  └───────────────────────────────────────────┘  │
└─────────────────────────────────────────────────┘
```

이 엔진에서는 **winit** 라이브러리가 OS로부터 창 이벤트를 받아서 엔진 함수를 호출합니다.

### App 구조체

```rust
// crates/engine/src/app.rs

pub struct App {
    pub world: World,                       // ECS 세계 (모든 게임 데이터 보관)
    systems: Vec<Box<dyn System>>,          // 등록된 시스템 목록
    window: Option<Arc<Window>>,            // 창 핸들
    gpu: Option<GpuContext>,                // GPU 컨텍스트
    sprite_renderer: Option<SpriteRenderer>,
    last_frame: Option<Instant>,            // 직전 프레임 시각
    pending_textures: Vec<String>,          // GPU 준비 전 대기 중인 텍스처
}
```

`Option<T>`으로 감싸진 필드들은 처음엔 `None`이고, GPU 초기화가 완료된 후에 `Some`이 됩니다.
Rust에서는 값이 없을 수 있는 상황을 `Option`으로 명시해야 합니다.

### 이벤트 처리 흐름

```
OS/winit 이벤트
       │
       ▼
ApplicationHandler::resumed()          ← 창이 처음 활성화될 때 1회 실행
       │  창 생성, GPU 초기화, 텍스처 일괄 로드
       │
ApplicationHandler::window_event()     ← 키보드 입력, 창 크기 변경
       │
ApplicationHandler::about_to_wait()    ← 이벤트 없을 때마다 호출
       │  window.request_redraw()  ← "화면 다시 그려" 요청
       │
ApplicationHandler::window_event()
  └─ RedrawRequested                   ← 렌더 타이밍
           │
           ├── update(dt)              ← 게임 로직 처리
           │     └── 등록된 System 순서대로 실행
           └── render()               ← 화면에 그리기
```

### 델타 타임(dt)이란?

컴퓨터마다 프레임이 처리되는 속도가 다릅니다.
`dt`는 "직전 프레임과 현재 프레임 사이의 시간(초)"으로, 이를 곱해서 속도를 표현하면
프레임레이트에 상관없이 같은 속도로 움직이게 됩니다.

```rust
// 예시: 초당 200픽셀 이동
transform.position.x += 200.0 * dt;
// dt = 0.016 (60fps) → 3.2px 이동
// dt = 0.033 (30fps) → 6.6px 이동
// 둘 다 1초에 200px 이동!
```

이 엔진에서는 갑자기 긴 프레임이 발생해도 게임이 폭주하지 않도록 `dt`를 최대 0.1초로 제한합니다.

```rust
let dt = dt.min(0.1);
```

### 텍스처 지연 로딩

GPU는 `resumed()` 이후에야 사용 가능합니다.
그래서 `load_texture()`는 텍스처 경로만 대기열에 저장해두고, GPU 초기화 완료 후에 일괄 로드합니다.

```rust
// main.rs에서 (GPU 초기화 전)
app.load_texture("assets/textures/player.png");

// 내부에서 (resumed 시점에 일괄 처리)
for path in &self.pending_textures {
    renderer.load_texture(device, queue, &texture_layout, path);
}
```

---

## 3. ECS — 게임 오브젝트를 다루는 방식

> 읽을 파일: `crates/engine/src/ecs/world.rs`, `crates/engine/src/ecs/system.rs`

### 왜 ECS인가?

전통적인 객체지향(OOP) 방식으로 게임을 만들면 이런 문제가 생깁니다.

```
Player extends Character extends GameObject  ← 상속이 깊어질수록 수정 어려움
Enemy extends Character extends GameObject
FlyingEnemy extends Enemy ...               ← 조합 폭발
```

ECS는 **데이터(Component)와 로직(System)을 분리**해서 이 문제를 해결합니다.

```
Entity(0) = Transform + Sprite + PhysicsBody  ← 플레이어
Entity(1) = Transform + Sprite                ← 배경 타일
Entity(2) = Transform + Sprite + PhysicsBody + AnimationPlayer  ← 적
```

게임 오브젝트는 ID 숫자일 뿐이고, 성질은 컴포넌트를 붙여서 표현합니다.

### 저장 구조

```rust
pub struct World {
    next_id: u32,
    entities: Vec<Entity>,

    // 핵심: 타입별로 Vec을 분리해서 저장
    components: HashMap<TypeId, Vec<Option<Box<dyn Any>>>>,
    //          ^^^^^^^^          ^^^^^^^^^^^^^^^^^^^^
    //         타입이 키           엔티티 인덱스 = 컴포넌트

    resources: HashMap<TypeId, Box<dyn Any>>,
}
```

`Box<dyn Any>`는 "어떤 타입이든 담을 수 있는 힙 할당 박스"입니다.
나중에 꺼낼 때는 `downcast_ref::<T>()`로 원래 타입을 복원합니다.
타입 정보는 `TypeId::of::<T>()`로 런타임에 확인합니다.

### 시각화

```
components:
  TypeId(Transform) → [Some(T0), Some(T1), None,     Some(T3)]
  TypeId(Sprite)    → [Some(S0), Some(S1), Some(S2), Some(S3)]
  TypeId(PhysBody)  → [Some(P0), None,     None,     Some(P3)]
                       ^^^^^^^^ ^^^^^^^^  ^^^^^^^^^  ^^^^^^^^^
                       Entity 0  Entity 1  Entity 2   Entity 3
```

### 주요 API

```rust
// 엔티티 생성
let player = world.spawn();

// 컴포넌트 붙이기
world.add_component(player, Transform::default());
world.add_component(player, Sprite::colored(1.0, 0.5, 0.0));

// 특정 컴포넌트를 가진 모든 엔티티 순회
for (entity, sprite) in world.query::<Sprite>() {
    // sprite: &Sprite
}

// 특정 엔티티의 컴포넌트 가져오기
let transform = world.get::<Transform>(player);       // Option<&Transform>
let transform = world.get_mut::<Transform>(player);   // Option<&mut Transform>

// 리소스 (전역 싱글턴)
world.insert_resource(InputState::new());
let input = world.resource::<InputState>().unwrap();
```

### System trait

```rust
pub trait System {
    fn run(&mut self, world: &mut World, dt: f32);
}
```

모든 게임 로직은 이 `run` 메서드 하나로 표현됩니다.
`world`에서 필요한 컴포넌트와 리소스를 꺼내서 처리합니다.

### Borrow Checker와 ECS

Rust의 borrow checker는 같은 변수를 두 번 빌릴 수 없습니다.
ECS에서 자주 마주치는 패턴입니다.

```rust
// 오류: world를 두 번 빌리게 됨
for (entity, transform) in world.query::<Transform>() {  // &world 빌림
    let player = world.get_mut::<AnimationPlayer>(entity); // &mut world 빌림 → 오류!
}

// 해결: 엔티티 ID를 먼저 모아두고 루프를 분리
let entities: Vec<Entity> = world.query::<Transform>()
    .map(|(e, _)| e)
    .collect();  // 여기서 빌림 종료

for entity in entities {
    let player = world.get_mut::<AnimationPlayer>(entity); // OK
}
```

이렇게 ID를 먼저 `collect()`하면 첫 번째 빌림이 끝나고 두 번째 빌림을 시작할 수 있습니다.

---

## 4. 컴포넌트 — 데이터 설계

> 읽을 파일: `crates/engine/src/components.rs`

### Transform — 위치·크기·회전

```rust
pub struct Transform {
    pub position: Vec2,   // (x, y) 픽셀 좌표
    pub scale: Vec2,      // (width, height) 픽셀 크기
    pub rotation: f32,    // 라디안 (0.0 = 위, PI = 아래)
}
```

`to_matrix()` 메서드는 이 세 값을 4×4 행렬로 변환합니다.
GPU는 정점 위치를 계산할 때 이 행렬을 곱해서 스프라이트를 올바른 위치·크기·각도로 그립니다.

변환 순서가 중요합니다: **Scale → Rotation → Translation** (SRT 순서).
반대로 하면 결과가 달라집니다.

```rust
fn to_matrix(&self) -> Mat4 {
    Mat4::from_translation(pos3d)
        * Mat4::from_rotation_z(self.rotation)
        * Mat4::from_scale(scale3d)
    // 행렬 곱셈은 오른쪽부터 적용 → Scale 먼저, Translation 나중
}
```

### Sprite — 시각적 외형

```rust
pub struct Sprite {
    pub texture: Option<String>,   // 텍스처 파일 경로 (없으면 단색)
    pub color: [f32; 4],           // RGBA, 0.0~1.0
}

// 편의 생성자
Sprite::colored(1.0, 0.5, 0.0)              // 주황색 단색
Sprite::textured("assets/player.png")       // 텍스처 사용
```

텍스처가 있어도 `color`는 **배율**로 작용합니다.
`[1.0, 1.0, 1.0, 1.0]` (흰색)이면 원본 텍스처 그대로, `[1.0, 0.5, 0.5, 1.0]`이면 붉게 물듭니다.

### GameState — 상태 머신

```rust
pub enum GameState {
    Playing,
    Paused,
    GameOver,
}
```

리소스로 삽입해서 시스템이 읽습니다.
현재 상태에 따라 로직을 분기하는 간단한 상태 머신입니다.

```rust
let state = world.resource::<GameState>().unwrap();
match state {
    GameState::Playing  => { /* 입력 처리 */ }
    GameState::GameOver => { /* R키 대기 */ }
    _ => {}
}
```

### UvRect와 AnimationPlayer — 스프라이트시트

스프라이트시트(sprite sheet)란 여러 프레임이 한 PNG에 담긴 이미지입니다.

```
┌───┬───┬───┬───┐
│ 0 │ 1 │ 2 │ 3 │  ← 걷기 애니메이션 프레임 0~3
├───┼───┼───┼───┤
│ 4 │ 5 │ 6 │ 7 │  ← 점프 애니메이션 프레임 4~7
└───┴───┴───┴───┘
   4열 × 2행
```

`UvRect`는 "이 텍스처의 어떤 영역을 사용할지"를 0~1 범위로 표현합니다.

```rust
// (col=2, row=1) 프레임 → UV 자동 계산
let uv = UvRect::from_grid(2, 1, 4, 2);
// u_offset = 2/4 = 0.5,  u_size = 1/4 = 0.25
// v_offset = 1/2 = 0.5,  v_size = 1/2 = 0.5
```

`AnimationPlayer`는 여러 `AnimationClip`을 저장하고 현재 재생 상태를 관리합니다.

```rust
let idle = AnimationClip {
    frames: vec![UvRect::from_grid(0, 0, 4, 2), UvRect::from_grid(1, 0, 4, 2)],
    fps: 8.0,
    looping: true,
};
let jump = AnimationClip {
    frames: vec![UvRect::from_grid(0, 1, 4, 2)],
    fps: 1.0,
    looping: false,
};
let mut player = AnimationPlayer::new(vec![idle, jump]);
player.play(0);  // idle 재생
player.play(1);  // jump로 전환
```

---

## 5. 입력 시스템 — 키보드 상태 추적

> 읽을 파일: `crates/engine/src/input.rs`

### 두 가지 입력 방식

```rust
input.is_pressed(KeyCode::KeyD)      // "D키를 누르고 있는가?" → 이동에 사용
input.just_pressed(KeyCode::Space)   // "이번 프레임에 처음 눌렀는가?" → 점프에 사용
```

`just_pressed`는 키를 오래 누르고 있어도 **딱 한 프레임만** `true`를 반환합니다.
매 프레임 끝에 `flush()`가 호출되어 초기화되기 때문입니다.

```rust
pub struct InputState {
    pressed: HashSet<KeyCode>,       // 현재 누른 키 전체
    just_pressed: HashSet<KeyCode>,  // 이번 프레임에 새로 누른 키
    just_released: HashSet<KeyCode>, // 이번 프레임에 뗀 키
}
```

`HashSet` 사용 이유: 동시에 여러 키를 눌러도 각 키를 독립적으로 추적하기 위해서입니다.

---

## 6. 렌더링 — GPU에 그림 그리기

> 읽을 파일: `crates/engine/src/renderer/context.rs`, `texture.rs`, `sprite.rs`

### GPU 초기화 (context.rs)

wgpu에서 GPU를 사용하려면 여러 객체를 순서대로 초기화해야 합니다.

```
Instance    ← "wgpu를 사용하겠다"는 진입점
    │
    ▼
Surface     ← 창에 연결된 드로잉 영역
    │
    ▼
Adapter     ← 사용 가능한 물리 GPU 중 하나를 자동 선택
    │
    ▼
Device      ← GPU와의 논리 연결 (API 호출 대상)
Queue       ← GPU에 명령을 보내는 큐
    │
    ▼
SurfaceConfiguration ← 출력 포맷, 해상도 등 설정
```

비동기(`async`) 함수로 구성된 것을 `pollster::block_on()`으로 동기 방식으로 호출합니다.
wgpu의 초기화 함수들이 `async`이기 때문입니다.

### 텍스처 로딩 (texture.rs)

```
PNG 파일
    │  image 크레이트로 디코딩
    ▼
RGBA 바이트 배열
    │  queue.write_texture()로 GPU 메모리에 업로드
    ▼
wgpu::Texture     ← GPU 메모리 위의 텍스처
    │
wgpu::TextureView ← 셰이더가 텍스처를 읽기 위한 뷰
wgpu::Sampler     ← 픽셀 보간 방식 (여기서는 Nearest = 선명)
    │
wgpu::BindGroup   ← 셰이더에 텍스처+샘플러를 묶어서 전달
```

### 스프라이트 렌더링 핵심 아이디어 (sprite.rs)

단순히 "스프라이트마다 draw 호출"하면 성능이 나쁩니다.
이 엔진은 **인스턴싱(instancing)** 을 사용합니다.

```
쿼드(정사각형) 1개만 GPU에 올려두고
스프라이트마다 "어디에, 얼마나 크게, 어떤 색으로" 데이터만 전달
→ GPU가 알아서 N개 복제해서 그림
```

```
Vertex Buffer (고정, 4정점)
┌─────────────────────┐
│ (-0.5, -0.5, 0, 0)  │  ← 좌하단
│ ( 0.5, -0.5, 1, 0)  │  ← 우하단
│ ( 0.5,  0.5, 1, 1)  │  ← 우상단
│ (-0.5,  0.5, 0, 1)  │  ← 좌상단
└─────────────────────┘

Instance Buffer (동적, 스프라이트 수만큼)
┌──────────────────────────────────────┐
│ [모델행렬(64B)] [색상(16B)] [UV(16B)] │ ← 스프라이트 0
│ [모델행렬(64B)] [색상(16B)] [UV(16B)] │ ← 스프라이트 1
│ ...                                  │
└──────────────────────────────────────┘
```

**텍스처 배칭**: 같은 텍스처를 쓰는 스프라이트끼리 묶어서 텍스처를 교체하는 횟수를 줄입니다.

```
[흰색A, 흰색B, 흰색C] → draw_indexed(인스턴스 3개) 1회
[player.png X, player.png Y] → draw_indexed(인스턴스 2개) 1회
```

**동적 버퍼 확장**: 스프라이트가 예상보다 많아지면 버퍼를 2배로 늘립니다.

```rust
if sprite_count > self.instance_capacity {
    self.instance_capacity = sprite_count * 2;  // 새 버퍼 생성
}
```

### 카메라 — 직교 투영

이 엔진은 2D이므로 원근감이 필요 없습니다.
직교 투영(orthographic projection)은 "픽셀 좌표 → GPU 클립 좌표"를 선형으로 변환합니다.

```
화면 좌상단 (0, 0)
              ────────────────────────→ x (width 픽셀)
             │
             │     스프라이트 (400, 300)
             │
             ▼
             y (height 픽셀)

GPU 클립 좌표로 변환:
  x: 0~width  → -1~+1
  y: 0~height → +1~-1  (y축 반전! GPU는 위가 +1)
```

---

## 7. WGSL 셰이더 — GPU 코드 읽기

> 읽을 파일: `assets/shaders/sprite.wgsl`

셰이더는 GPU에서 실행되는 코드입니다. CPU 코드와 달리 **모든 정점·픽셀에 대해 병렬로 실행**됩니다.

### 입력 데이터

```wgsl
// CPU(Rust)에서 바인딩한 유니폼
@group(0) @binding(0) var<uniform> camera: Camera;    // 카메라 행렬
@group(1) @binding(0) var t_sprite: texture_2d<f32>;  // 텍스처
@group(1) @binding(1) var s_sprite: sampler;          // 샘플러
```

### 버텍스 셰이더 (vs_main)

정점(vertex)마다 실행됩니다. 쿼드 4개 정점 × 스프라이트 수 = N번 실행.

```wgsl
@vertex
fn vs_main(v: Vertex, inst: Instance) -> VertexOutput {
    // 인스턴스 데이터에서 4×4 모델 행렬 재조립
    let model = mat4x4(inst.model_0, inst.model_1, inst.model_2, inst.model_3);

    // 클립 좌표 계산: 카메라 행렬 × 모델 행렬 × 정점 위치
    let clip_pos = camera.view_proj * model * vec4(v.position, 0.0, 1.0);

    // UV 매핑: 스프라이트시트의 올바른 영역 계산
    // v.uv는 [0,1] 범위의 기본 UV
    // inst.uv_offset, inst.uv_size로 스프라이트시트 내 위치 지정
    let uv = inst.uv_offset + v.uv * inst.uv_size;

    return VertexOutput(clip_pos, uv, inst.color);
}
```

### 프래그먼트 셰이더 (fs_main)

픽셀(fragment)마다 실행됩니다. 최종 색상을 결정합니다.

```wgsl
@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // 텍스처에서 색상 샘플링 × 스프라이트 색상(배율)
    return textureSample(t_sprite, s_sprite, in.uv) * in.color;
}
```

단색 스프라이트는 `white_texture` (1×1 흰 픽셀)를 사용합니다.
`textureSample` → `(1,1,1,1)` × `color` = `color` 그대로.

---

## 8. 물리 시스템 — rapier2d 연동

> 읽을 파일: `crates/engine/src/physics/system.rs`

### rapier2d란?

Rust용 2D 물리 엔진입니다. 중력, 충돌, 마찰을 시뮬레이션합니다.

### 핵심 개념

| 용어 | 설명 | 예시 |
|------|------|------|
| RigidBody | 물리 법칙을 따르는 물체 | 플레이어, 상자 |
| Collider | 충돌 감지용 형상 | 박스, 원 |
| Dynamic | 중력·힘에 반응 | 플레이어 |
| Static | 고정된 물체 | 바닥, 벽 |

### 좌표 스케일 (SCALE = 50.0)

rapier2d는 SI 단위계(미터, kg)를 사용합니다.
픽셀을 직접 넣으면 물리값이 비현실적으로 커져서 시뮬레이션이 불안정해집니다.

```rust
const SCALE: f32 = 50.0;  // 1 물리 단위 = 50 픽셀

fn p(px: f32) -> f32 { px / SCALE }  // 픽셀 → 물리단위

// 플레이어를 (200px, 100px) 위치에 50×50px 크기로 생성
physics.add_dynamic_box(
    Vec2::new(p(200.0), p(100.0)),  // 위치: (4.0, 2.0) 물리단위
    p(25.0), p(25.0),               // 반폭/반높이: (0.5, 0.5) 물리단위
    true,                           // 회전 고정
);

// 물리 결과를 픽셀로 변환해서 Transform에 반영
transform.position = Vec2::new(t.x * SCALE, t.y * SCALE);
```

### Borrow Checker 문제와 해결

`PhysicsWorld`를 ECS 리소스로 넣으면 문제가 생깁니다.

```rust
// 이렇게 하면 안 됨:
let physics = world.resource_mut::<PhysicsWorld>();  // &mut world
let transforms = world.query::<Transform>();          // &mut world → 오류!
```

해결책: `PhysicsSystem` 구조체가 `PhysicsWorld`를 **직접 소유**합니다.

```rust
pub struct PhysicsSystem {
    physics: PhysicsWorld,  // ECS 외부에서 소유
    // ...
}

impl System for PhysicsSystem {
    fn run(&mut self, world: &mut World, dt: f32) {
        self.physics.step(dt);    // self 소유이므로 &mut world와 충돌 없음
        // world 접근 가능
    }
}
```

### 착지 판정

`has_contact()`는 특정 콜라이더가 현재 다른 물체와 접촉 중인지 확인합니다.
이를 이용해 "바닥에 닿아 있을 때만 점프 가능"을 구현합니다.

```rust
let on_ground = self.physics.has_contact(self.player_col);
if on_ground && input.just_pressed(KeyCode::Space) {
    // 점프 임펄스 적용
    body.apply_impulse(vector![0.0, -jump_force], true);
}
```

---

## 9. 애니메이션 시스템 — 스프라이트시트 활용

> 읽을 파일: `crates/engine/src/animation.rs`

### 동작 원리

```rust
impl System for AnimationSystem {
    fn run(&mut self, world: &mut World, dt: f32) {
        // Borrow Checker 패턴: ID 먼저 수집
        let entities: Vec<Entity> = world
            .query::<AnimationPlayer>()
            .map(|(e, _)| e)
            .collect();

        for entity in entities {
            let player = world.get_mut::<AnimationPlayer>(entity).unwrap();
            let clip = &player.clips[player.current_clip];
            let frame_duration = 1.0 / clip.fps;

            player.timer += dt;
            if player.timer >= frame_duration {
                player.timer -= frame_duration;
                player.current_frame += 1;

                if player.current_frame >= clip.frames.len() {
                    if clip.looping {
                        player.current_frame = 0;  // 처음으로
                    } else {
                        player.current_frame = clip.frames.len() - 1;  // 마지막 유지
                    }
                }
            }
        }
    }
}
```

`SpriteRenderer`가 렌더링할 때 `AnimationPlayer`가 있으면 현재 프레임의 UV를 가져다 씁니다.

---

## 10. 오디오 시스템 — 사운드 재생

> 읽을 파일: `crates/engine/src/audio.rs`

### 채널 기반 관리

여러 소리를 동시에 재생하면서 개별 제어가 필요합니다.
채널(문자열 이름)로 각 음원을 독립적으로 관리합니다.

```rust
// 오디오 초기화 (실패해도 게임은 계속 실행)
let audio = AudioManager::new().ok();
world.insert_resource(audio);

// 재생
audio.play("bgm", "assets/audio/bgm.wav", true);      // 배경음악 (반복)
audio.play("jump", "assets/audio/jump.wav", false);    // 점프 효과음 (1회)

// 같은 채널에 새 소리 → 기존 소리 자동 정지
audio.play("bgm", "assets/audio/gameover.wav", false); // bgm 대체

// 볼륨 조절
audio.set_volume("bgm", 0.5);  // 50%
```

`_stream` 필드가 `OutputStream`을 보유합니다.
이 필드를 `drop`하면 모든 소리가 멈추기 때문에 구조체가 살아있는 동안 반드시 유지해야 합니다.

---

## 11. 게임 샘플 완독 — 모든 것을 합치기

> 읽을 파일: `crates/game/src/main.rs`

이 파일이 엔진의 모든 기능을 사용하는 실제 예시입니다.

### 초기화 순서

```rust
fn main() {
    let mut app = App::new();               // 1. 엔진 생성

    let mut physics = PhysicsWorld::new(9.8); // 2. 물리 세계 생성

    // 3. 정적 지형 생성 (물리 + 시각)
    let floor_left = world.spawn();
    world.add_component(floor_left, Transform { ... });
    world.add_component(floor_left, Sprite::colored(...));
    let (rb, col) = physics.add_static_box(...);
    world.add_component(floor_left, PhysicsBody { rb, col });

    // 4. 플레이어 생성
    let player = world.spawn();
    // ... Transform, Sprite, PhysicsBody 부착

    // 5. 리소스 등록
    world.insert_resource(GameState::Playing);
    world.insert_resource(AudioManager::new().ok());

    // 6. 시스템 등록 (실행 순서 = 등록 순서)
    app.add_system(AnimationSystem);
    app.add_system(PlatformerSystem::new(physics, ...));

    app.run();  // 7. 게임 루프 시작 (블로킹)
}
```

### 게임 로직 흐름 (PlatformerSystem::run)

```
1. GameState 확인
   ├─ GameOver → R키 대기, 재시작 처리
   └─ Playing  → 아래 로직 실행

2. 입력 읽기
   ├─ A/← → 왼쪽 속도 설정
   ├─ D/→ → 오른쪽 속도 설정
   └─ Space + 착지 상태 → 점프 임펄스

3. 물리 시뮬레이션 (step)

4. Transform 동기화 (물리 위치 → ECS 위치)

5. 낙하 감지
   └─ position.y > 700px → GameState::GameOver
```

---

## 12. 실습 과제

코드를 읽고 이해했다면 직접 수정해 보세요.

### 초급

**과제 1: 스프라이트 색상 바꾸기**
`crates/game/src/main.rs`에서 플레이어 스프라이트 색상을 파란색으로 변경하세요.

**과제 2: 이동 속도 조절**
`PlatformerSystem`의 `speed` 값을 2배로 늘려서 더 빠르게 움직이게 하세요.

**과제 3: 낙하 판정 높이 변경**
현재 `y > 700.0`에서 게임 오버가 됩니다. 이 값을 `600.0`으로 바꾸면 어떻게 되나요?

### 중급

**과제 4: 새 엔티티 추가**
`main.rs`에서 공중 플랫폼을 하나 더 추가하세요.
바닥과 같은 방식으로 `Transform + Sprite + PhysicsBody`를 붙이면 됩니다.

**과제 5: 점프 횟수 제한**
현재 플레이어가 공중에서도 점프가 가능합니다.
`PlatformerSystem`에 카운터 필드를 추가해서 최대 2회 점프로 제한해 보세요.

**힌트**:
```rust
struct PlatformerSystem {
    // ...
    jump_count: u32,      // 현재 점프 횟수 추가
}

// run() 내에서:
// - 착지 시 jump_count = 0 으로 리셋
// - 점프 시 jump_count += 1
// - jump_count < 2 일 때만 점프 허용
```

**과제 6: 게임 오버 시 색상 변경**
`GameState::GameOver`가 되면 플레이어 스프라이트를 빨간색으로 바꾸세요.
`PlatformerSystem::run` 내에서 `world.get_mut::<Sprite>(player_entity)`로 접근할 수 있습니다.

### 고급

**과제 7: 새 시스템 작성**
`System` trait을 구현하는 새 구조체를 만드세요.
예시: 모든 스프라이트를 매 프레임 천천히 회전시키는 `RotationSystem`.

```rust
struct RotationSystem {
    speed: f32,  // 라디안/초
}

impl System for RotationSystem {
    fn run(&mut self, world: &mut World, dt: f32) {
        // 여기에 구현
        // world.query::<Transform>()으로 순회
        // transform.rotation += self.speed * dt;
    }
}
```

**과제 8: 스코어 시스템**
정수 `Score` 타입을 ECS 리소스로 추가하고,
플레이어가 플랫폼에 착지할 때마다 점수가 오르게 만드세요.

**과제 9: AnimationSystem 확장**
`AnimationPlayer`에 현재 클립이 끝났을 때 자동으로 다른 클립으로 전환하는 기능을 추가하세요.
(예: 점프 애니메이션이 끝나면 idle 클립으로 자동 복귀)

---

## 참고: 주요 Rust 개념 정리

| 개념 | 이 프로젝트에서의 쓰임새 |
|------|--------------------------|
| `Box<T>` | 힙에 할당. `Box<dyn Any>`로 타입 소거 |
| `Arc<T>` | 여러 곳에서 공유. 텍스처 캐시에서 사용 |
| `Option<T>` | 없을 수 있는 값. GPU 초기화 전 필드들 |
| `dyn Trait` | 런타임 다형성. `Box<dyn System>`, `Box<dyn Any>` |
| `TypeId` | 런타임 타입 식별자. ECS 컴포넌트 구분 키 |
| `downcast_ref` | `Any` 타입 객체를 원래 타입으로 복원 |
| `HashMap` | O(1) 키-값 조회. 컴포넌트·리소스 저장 |
| `HashSet` | O(1) 원소 존재 확인. 입력 키 상태 |
| `impl Trait` | 정적 다형성. `add_system(impl System)` |

---

*이 엔진은 학습 목적으로 만들어졌습니다. 코드를 두려워하지 말고 직접 바꿔보세요. 컴파일 오류 메시지가 어디를 고쳐야 할지 알려줍니다.*
