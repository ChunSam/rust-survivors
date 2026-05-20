# engine — Rust 2D 게임 엔진

학습 목적으로 Rust로 처음부터 만드는 2D 게임 엔진 라이브러리.
wgpu 22 렌더링 · 직접 구현한 ECS · rapier2d 물리 · spatial-hash 충돌을 단계적으로 구축.

## 의존 크레이트

| 크레이트 | 버전 | 역할 |
|---|---|---|
| `wgpu` | 22 | GPU 렌더링 |
| `glyphon` | 0.6 | GPU 텍스트 렌더링 (wgpu 22 네이티브) |
| `winit` | 0.30 | 창·이벤트 루프 |
| `rapier2d` | 0.22 | 2D 물리 (플랫포머 데모용) |
| `glam` | 0.28 | 수학 (`Vec2`, `Mat4`) |
| `bytemuck` | 1.x | GPU 버퍼 캐스팅 |
| `image` | 0.25 | PNG 텍스처 로딩 |
| `pollster` | 0.3 | wgpu async 초기화 |
| `rodio` | — | 오디오 재생 |
| `serde` + `ron` | — | 저장/로드 직렬화 |
| `dirs` | — | OS 데이터 디렉토리 경로 |
| `rand` | 0.8 | 난수 생성 |

## 모듈 요약

| 모듈 | 경로 | 핵심 내용 |
|---|---|---|
| `ecs` | `src/ecs/` | `World`, `Entity(u32)`, `System` trait |
| `renderer` | `src/renderer/` | `GpuContext`, `SpriteRenderer`, `Texture`, `TextRenderer` |
| `camera` | `src/camera.rs` | `Camera { position, zoom }`, `view_proj` |
| `collision` | `src/collision/` | `Collider`, `CollisionLayer`, `SpatialGrid`, `CollisionGridSystem` |
| `physics` | `src/physics/` | `PhysicsWorld` (rapier2d 래퍼), `PhysicsSystem`, `PhysicsBody` |
| `input` | `src/input.rs` | `InputState` — 키보드·마우스·스크롤 |
| `audio` | `src/audio.rs` | `AudioManager` — rodio 기반 wav 재생 |
| `animation` | `src/animation.rs` | `AnimationSystem`, `AnimationPlayer`, `AnimationClip` |
| `save` | `src/save.rs` | RON 직렬화 저장/로드 |
| `app` | `src/app.rs` | `App` — winit 이벤트 루프, 게임 루프 |
| `components` | `src/components.rs` | `Transform`, `Sprite`, `UvRect`, `GameState` |

## 주요 API

### App

| 함수 | 시그니처 | 설명 |
|---|---|---|
| `App::new` | `() -> Self` | 앱 초기화 (winit EventLoop 생성) |
| `App::add_system` | `<S: System + 'static>(&mut self, S)` | 시스템 등록 (등록 순으로 매 프레임 실행) |
| `App::run` | `(self)` | 이벤트 루프 진입. ESC 로 종료. |
| `App::load_texture` | `(&mut self, path: &str) -> TextureHandle` | PNG 텍스처 로드 |
| `App::reload_scene` | `(&mut self)` | World 초기화 후 setup 콜백 재실행 |

### World (ECS)

| 함수 | 시그니처 | 설명 |
|---|---|---|
| `spawn` | `(&mut self) -> Entity` | 빈 엔티티 생성 (`free_ids` 재사용) |
| `despawn` | `(&mut self, Entity)` | 엔티티 제거 + 컴포넌트 해제 |
| `add_component` | `<T: 'static>(&mut self, Entity, T)` | 컴포넌트 부착 |
| `get` | `<T: 'static>(&self, Entity) -> Option<&T>` | 컴포넌트 읽기 |
| `get_mut` | `<T: 'static>(&mut self, Entity) -> Option<&mut T>` | 컴포넌트 쓰기 |
| `query` | `<T: 'static>(&self) -> impl Iterator<(Entity, &T)>` | 단일 컴포넌트 순회 |
| `query2` | `<A, B: 'static>(&self) -> impl Iterator<(Entity, &A, &B)>` | 두 컴포넌트 동시 순회 |
| `query3` | `<A, B, C: 'static>(&self) -> impl Iterator<(Entity, &A, &B, &C)>` | 세 컴포넌트 동시 순회 |
| `insert_resource` | `<T: 'static>(&mut self, T)` | 전역 리소스 삽입 |
| `resource` | `<T: 'static>(&self) -> Option<&T>` | 리소스 읽기 |
| `resource_mut` | `<T: 'static>(&mut self) -> Option<&mut T>` | 리소스 쓰기 |

### Camera

| 항목 | 내용 |
|---|---|
| 타입 | `Camera { pub position: Vec2, pub zoom: f32 }` |
| 기본값 | `position = Vec2::ZERO`, `zoom = 1.0` |
| `Camera::new` | `(position: Vec2, zoom: f32) -> Self` |
| `Camera::view_proj` | `(&self, width: f32, height: f32) -> Mat4` |
| 좌표 규약 | `position` = 뷰포트 좌상단 월드 좌표. 기본값은 `Mat4::orthographic_rh(0, w, h, 0, -1, 1)` 과 동일 |

플레이어를 화면 중앙에 놓으려면:
```rust
camera.position = player_pos - Vec2::new(w, h) / (2.0 * camera.zoom);
```

### Collision

| 항목 | 시그니처 | 설명 |
|---|---|---|
| `Collider::Circle` | `{ radius: f32 }` | 원형 콜라이더 |
| `Collider::Aabb` | `{ half_extents: Vec2 }` | 축 정렬 박스 |
| `CollisionLayer` | `(u32)` newtype | 비트마스크 레이어. `AND = 0` 이면 충돌 무시 |
| `CollisionLayer::ALL` | `Self(u32::MAX)` | 모든 레이어 |
| `SpatialGrid::new` | `(cell_size: f32) -> Self` | 기본 128 px 셀 |
| `SpatialGrid::rebuild` | `(&mut self, world: &World)` | 매 프레임 호출 |
| `SpatialGrid::query_radius` | `(&self, center: Vec2, radius: f32, mask: CollisionLayer) -> Vec<Entity>` | 원형 범위 쿼리 |
| `SpatialGrid::query_aabb` | `(&self, min: Vec2, max: Vec2, mask: CollisionLayer) -> Vec<Entity>` | AABB 범위 쿼리 |
| `CollisionGridSystem` | `impl System` | 매 프레임 `SpatialGrid.rebuild` 실행. `grid` 필드 직접 소유. |

> **borrow checker 주의**: `SpatialGrid` 를 ECS 리소스로 넣으면 `&mut World` 와 충돌한다.
> `CollisionGridSystem` 이 직접 소유하는 것이 정석 — `PhysicsSystem` 패턴과 동일.

### 텍스트 렌더링

| 항목 | 내용 |
|---|---|
| `DrawText` | `{ text: String, position: Vec2, size: f32, color: [u8;4] }` |
| `TextQueue::push` | `(&mut self, DrawText)` — 매 프레임 항목 추가 |
| `TextQueue::clear` | `(&mut self)` — `TextRenderer::render` 내부에서 자동 호출 |
| 폰트 | `NotoSansKR-Regular.ttf` `include_bytes!` 임베딩 (SIL OFL 1.1) |

### 저장

| 함수 | 시그니처 | 설명 |
|---|---|---|
| `save::save_path` | `(app_name: &str, file: &str) -> PathBuf` | OS 데이터 디렉토리 하위 경로 |
| `save::save` | `<T: Serialize>(path: &Path, data: &T) -> Result<(), SaveError>` | RON 직렬화 후 파일 저장 |
| `save::load` | `<T: DeserializeOwned>(path: &Path) -> Result<T, SaveError>` | 파일 읽고 RON 역직렬화 |
| `SaveError` | `Io(io::Error) \| Ron(String)` | 에러 타입 |

## 좌표계

- 좌상단 = `(0, 0)`, 우하단 = `(width, height)`. Y 아래 방향이 양수.
- 픽셀 단위. 물리 시스템(rapier2d)은 `SCALE = 50.0` 으로 변환.
- 카메라는 좌상단 앵커 — `Camera::position` 이 뷰포트 좌상단 월드 좌표.

## 빌드

```bash
cargo test -p engine          # 단위 테스트 (26개)
cargo build -p engine         # 라이브러리 빌드
```

## 단위 테스트

| 모듈 | 테스트 수 | 주요 항목 |
|---|---|---|
| `camera` | 3 | 기본값 회귀, 이동, 줌 |
| `collision/grid` | 4 | 빈 쿼리, 단일 원 검출, 마스크 필터, despawn 후 rebuild |
| `save` | 2 | tempdir RON 라운드트립, 누락 파일 에러 |
| `renderer/text` | 3 | push/clear, 이터레이터 순서, DrawText 필드 |
| `ecs/world` | 기타 | spawn/despawn, query, 리소스 등 |

총 **26개** 통과 (`cargo test -p engine`).

## 이 엔진을 사용한 게임

`crates/game/` — 두 개의 독립 게임 바이너리:

| 바이너리 | 설명 |
|---|---|
| `game` | 플랫포머 데모 (`src/main.rs`). WASD/화살표 이동, Space 점프, rapier2d 물리. |
| `survivor` | Vampire Survivors 클론 (`src/bin/survivor.rs`). 탑다운 자동공격 로그라이트. |
| `text_demo` | glyphon 텍스트 렌더링 데모 (`src/bin/text_demo.rs`). |
