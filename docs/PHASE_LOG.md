# Phase 진행 로그

Vampire Survivors 클론 개발 진행 상황. 각 sub-phase 완료 시 항목 추가.

---

## Phase 0 — 엔진 보강 (2026-05-20 완료)

서바이버 작업 시작 전 채워야 했던 엔진 공백을 채움.

### 신규 모듈

| 모듈 | 파일 | 핵심 타입·함수 |
|---|---|---|
| 카메라 | `crates/engine/src/camera.rs` | `Camera { position, zoom }`, `Camera::new(pos, zoom)`, `view_proj(w, h) -> Mat4` |
| 충돌 그리드 | `crates/engine/src/collision/grid.rs` | `Collider`, `CollisionLayer(u32)`, `SpatialGrid`, `CollisionGridSystem` |
| 충돌 쿼리 | `crates/engine/src/collision/query.rs` | `SpatialGrid::query_radius`, `SpatialGrid::query_aabb` |
| 저장 | `crates/engine/src/save.rs` | `save_path`, `save<T: Serialize>`, `load<T: DeserializeOwned>`, `SaveError` |

### 기존 모듈 확장

| 파일 | 변경 내용 |
|---|---|
| `ecs/world.rs` | `despawn(entity)` + `free_ids` 재사용, `query2::<A,B>()`, `query3::<A,B,C>()`, `get_mut::<T>()` |
| `components.rs` | `Transform.z: f32` 추가 (스프라이트 z 정렬), `GameState` 변종 확장 |
| `renderer/sprite.rs` | 하드코딩 ortho → `Camera` 리소스 기반 `view_proj` 사용, z 정렬 |
| `input.rs` | 마우스: `cursor: Vec2`, `mouse_pressed: [bool; 3]`, `scroll: Vec2` 추가 |
| `app.rs` | `Camera` 리소스 초기화, `TextRenderer` 렌더 패스 연결, 마우스 이벤트 핸들링 |
| `Cargo.toml` | `rand = "0.8"`, `serde`, `ron`, `dirs` 추가 |

### 검증

- 단위 테스트: engine 26개 통과 (`cargo test -p engine`)
- `cargo build --workspace` 성공
- 플랫포머 데모(`cargo run -p game --bin game`) 회귀 없음
  - `Camera` 기본값(position=ZERO, zoom=1.0)이 기존 `Mat4::orthographic_rh(0,w,h,0,-1,1)` 과 동일 — 단위 테스트로 강제

---

## Phase 0.5 — wgpu 22 + glyphon 0.6 + 한글 폰트 동봉 (2026-05-20 완료)

### 변경

| 항목 | 내용 |
|---|---|
| wgpu 0.20 → 22 | `DeviceDescriptor.memory_hints`, `RenderPipelineDescriptor.cache: None` 필드 추가 |
| glyphon 0.6 도입 | wgpu 22 네이티브 버전 (`vendor` 패치 불필요) |
| glyphon API | `Cache::new(device)` → `Viewport::new(device, &cache)` → `TextAtlas::new(device, queue, &cache, format)` 순서 |
| Viewport 갱신 | `Viewport::update(queue, Resolution { width, height })` 로 매 프레임 GPU 유니폼 갱신 |
| 한글 폰트 | `NotoSansKR-Regular.ttf` (~4.5 MB) `include_bytes!` 임베딩 — OS 시스템 폰트 의존 제거 |

### 신규 파일

| 파일 | 내용 |
|---|---|
| `crates/engine/src/renderer/text.rs` | `TextRenderer`, `TextQueue`, `DrawText` |
| `assets/fonts/NotoSansKR-Regular.ttf` | OpenType 한글 폰트 (SIL OFL 1.1) |
| `assets/fonts/OFL.txt` | SIL OFL 1.1 라이선스 전문 |

### 설계 결정

- `Cache` 소유권 배치: `TextRenderer` 구조체가 `cache` 필드 직접 소유 — `TextAtlas::new` 에 `&Cache` 가 필요하므로 먼저 생성 후 공유
- TextRenderer 가 `Cache` 를 `#[allow(dead_code)]` 로 보존 (실제로는 atlas 내부에서 참조)
- 텍스트 렌더 패스: `LoadOp::Load` 로 스프라이트 패스 위에 합성

### 검증

- `cargo run -p game --bin text_demo` — 한글·영문 동시 출력 (GUI 환경 필요)
- engine 단위 테스트 3건: `TextQueue::push/clear`, 이터레이터 순서 보존, `DrawText` 필드 보존

---

## Phase 1-A — 서바이버 진입점 + Player + Camera follow (2026-05-20 완료)

### 신규 파일

| 파일 | 내용 |
|---|---|
| `crates/game/src/lib.rs` | `pub mod survivor;` + game lib 단위 테스트 3건 |
| `crates/game/src/survivor/mod.rs` | 모듈 재수출 |
| `crates/game/src/survivor/player.rs` | `Player`, `Velocity`, `PlayerStats`, `PlayerMovementSystem` |
| `crates/game/src/survivor/camera_follow.rs` | `CameraFollowSystem { lerp: 0.15, viewport }` |
| `crates/game/src/survivor/world_setup.rs` | `spawn_player(world)` — 노란 48×48 스프라이트, z=1 |
| `crates/game/src/bin/survivor.rs` | Phase 1-A 진입점 |
| `crates/game/src/bin/text_demo.rs` | glyphon 텍스트 데모 |

### 시스템 등록 순서 (Phase 1-A)

```
PlayerMovementSystem → CameraFollowSystem
```

### 플레이어 스펙

| 항목 | 값 |
|---|---|
| 스프라이트 | 노란 사각형 48×48 px (Phase 1-B에서 텍스처 교체) |
| 이동 속도 | 200 px/s (`PlayerStats.move_speed`) |
| 카메라 lerp | 0.15 (기본) |
| 초기 위치 | 월드 (0, 0) |
| z 레이어 | 1.0 (적보다 위) |

### 검증

- `cargo build --workspace` 성공
- `cargo test --workspace` — engine 26 + game lib 3 = 29개 통과
- 바이너리 3개 확인: `game` · `survivor` · `text_demo`

---

## 다음 작업: Phase 1-B — Zombie 적 + 스폰

CLAUDE.md "Vertical Slice MVP" 섹션 참고.

### 예정 파일

| 파일 | 내용 |
|---|---|
| `crates/game/src/survivor/enemy.rs` | `Zombie`, `EnemyAi`, `EnemyAiSystem` (플레이어 추격) |
| `crates/game/src/survivor/spawn.rs` | `EnemySpawnSystem` — 카메라 외곽 원형 경계 주기 스폰, 거리 초과 시 despawn |

### 준비 사항

- `CollisionLayer` 부착 — Phase 1-C `DamageSystem` 의 spatial grid 쿼리 준비
- `SpatialGrid` / `CollisionGridSystem` 은 Phase 0 에서 이미 구현 완료
