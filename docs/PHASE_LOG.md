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

---

### Phase 1-B — Zombie 적 + 스폰 시스템 (2026-05-20)

#### 신규 파일

| 파일 | 내용 |
|---|---|
| `crates/game/src/survivor/enemy.rs` | `Zombie` 태그, `EnemyAi { move_speed: 80.0 }`, `EnemyAiSystem` |
| `crates/game/src/survivor/spawn.rs` | `SpawnTimer { interval, elapsed }`, `EnemySpawnSystem`, `spawn_zombie(world, pos)` |

#### 신규 시스템

| 시스템 | 동작 |
|---|---|
| `EnemyAiSystem` | `query3::<Zombie, Transform, EnemyAi>` → collect → get_mut 두 단계로 플레이어 추격 |
| `EnemySpawnSystem` | 타이머 경과 시 카메라 중심 반경 500px 원형 경계에서 1마리 스폰. 반경 900px 초과 좀비 despawn |

#### 신규 컴포넌트 / 태그

- `Zombie` — 적 태그
- `EnemyAi { move_speed }` — 추격 속도 파라미터
- 스폰 시 `Collider::Circle { radius: 20.0 }` + `CollisionLayer(LAYER_ENEMY)` 부착 (Phase 1-C DamageSystem 준비)

#### 레이어 상수 (mod.rs)

```rust
pub const LAYER_PLAYER: u32 = 1 << 0;
pub const LAYER_ENEMY:  u32 = 1 << 1;
```

#### 테스트

game lib 6 통과 (직전 3 + 신규 3):
- `enemy_ai_moves_toward_player` — 좀비가 플레이어 방향으로 이동
- `spawn_timer_triggers_after_interval` — dt=interval 시 좀비 1마리 스폰
- `enemy_despawned_when_far` — despawn_radius 밖 좀비 자동 정리

#### 검증

```bash
cargo build --workspace        # ok
cargo test --workspace         # engine 26 / game lib 6 / doc 2 통과
cargo build --release --workspace  # ok
```

#### 핵심 결정

- `viewport` 는 `EnemySpawnSystem` 필드(800×600) — `Camera` 리소스에 viewport 가 없으므로 `CameraFollowSystem` 패턴 답습
- `SpawnTimer` 리소스는 `spawn_player()` 안에서 삽입 — 씬 재시작(`reload_scene`) 시 재삽입되지 않으므로, 추후 `reload_scene` 확장 시 주의
- `CollisionGridSystem` / `DamageSystem` 등록은 Phase 1-C 로 연기

---

### Phase 1-C — Whip 무기 + DamageSystem (2026-05-20)

#### 신규 파일

| 파일 | 내용 |
|---|---|
| `crates/game/src/survivor/health.rs` | `Health { current, max }`, `take_damage(amount) -> bool` |
| `crates/game/src/survivor/weapon.rs` | `Whip` 컴포넌트, `WhipSystem` (SpatialGrid 직접 소유) |

#### 신규 컴포넌트

| 컴포넌트 | 기본값 | 부착 대상 |
|---|---|---|
| `Health { current, max }` | `new(100)` / `new(30)` | Player / Zombie |
| `Whip { level, damage, area_width, area_height, cooldown }` | `1, 10.0, 120.0, 60.0, 1.0` | Player |

#### WhipSystem 동작

1. `elapsed += dt`
2. `query3::<Player, Transform, Whip>` 로 위치·수치 캐시
3. `elapsed < cooldown` 이면 조기 반환; 아니면 `elapsed -= cooldown`
4. 발화 시점에만 `grid.rebuild(world)` — 매 프레임 rebuild 비용 절감
5. 좌측 AABB `(player - (width, height/2), player + (0, height/2))` + 우측 AABB 양쪽 쿼리
6. `LAYER_ENEMY` 마스크로 hit 엔티티 수집 → sort + dedup
7. `Health::take_damage` → HP ≤ 0 이면 `world.despawn`

#### 테스트

game lib 9 통과 (직전 6 + 신규 3):
- `whip_does_not_trigger_before_cooldown` — cooldown 전 발화 없음
- `whip_damages_zombies_in_range` — 10 데미지 차감 (30→20)
- `zombie_dies_when_hp_reaches_zero` — HP ≤ 0 → despawn → `Zombie` count = 0

#### 검증

```bash
cargo build --workspace             # ok
cargo test --workspace              # engine 26 / game lib 9 / doc 2 통과
cargo build --release --workspace   # ok
```

#### 핵심 결정

- `SpatialGrid` 를 `WhipSystem` 이 직접 소유 — `PhysicsSystem` 이 `PhysicsWorld` 를 소유하는 패턴 답습. borrow checker 충돌 회피.
- `CollisionGridSystem` 미등록 — `WhipSystem` 이 자체 grid 를 발화 시점에만 rebuild 하므로 불필요.
- 부채꼴 대신 좌/우 AABB 사각형 hitbox — 학습 단계 단순화. Phase 2 무기 풀 확장 시 부채꼴·원·광선 hitbox 로 교체 가능.

### Phase 1-C 보강 (2026-05-20)

#### 히트플래시

- `HitFlash { remaining: f32, original: [f32; 4] }` 컴포넌트 신규 추가
- `HitFlashSystem` — 매 프레임 `remaining -= dt`; 만료 시 `Sprite.color = original` 복원
- `World::remove_component` 미존재 → 복원 완료 후 `remaining = f32::NEG_INFINITY` sentinel 처리로 재처리 방지
- `WhipSystem` hit 루프: sprite 색을 `[1.0, 0.3, 0.3, 1.0]` (빨강)으로 즉시 변경 후 `HitFlash` 부착 (0.1초); 사망 시 despawn (플래시 불필요)

#### 콘솔 로그

- `WhipSystem` 발화 시 `hit_actual > 0` 이면 `println!("Whip: {} hits, {} killed", hit_actual, killed)` 출력
- 디버깅 보조용 — 다음 폴리쉬 단계(Phase 1-D 이후)에서 HUD 처치 카운터로 대체 예정

#### 테스트

- game lib 10 통과 (직전 9 + 신규 1)
- `whip_hit_attaches_hit_flash` — hit 된 생존 좀비에 `HitFlash` 부착 확인 + sprite 색 빨강 확인

#### 변경 파일

| 파일 | 변경 |
|---|---|
| `crates/game/src/survivor/weapon.rs` | `HitFlash`, `HitFlashSystem` 추가; `WhipSystem` hit 루프에 색 변경 + 로그 추가 |
| `crates/game/src/survivor/mod.rs` | `HitFlash`, `HitFlashSystem` 재수출 추가 |
| `crates/game/src/bin/survivor.rs` | `HitFlashSystem` 임포트 + `WhipSystem` 다음 등록 |
| `crates/game/src/lib.rs` | `HitFlash` 임포트 + `whip_hit_attaches_hit_flash` 테스트 추가 |

---

## 다음 작업: Phase 1-D — XpGem 드롭 + 자석 + 레벨업 + 플레이어 사망
