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

---

### Phase 1-D — XpGem 드롭 + 자석 흡수 (2026-05-20)

#### 신규 파일

| 파일 | 내용 |
|---|---|
| `crates/game/src/survivor/xp.rs` | `XpGem`, `XpAccumulator`, `MagnetSystem`, `spawn_xp_gem` |

#### 신규 컴포넌트

| 컴포넌트 | 설명 | 부착 대상 |
|---|---|---|
| `XpGem { value: u32 }` | 적 사망 시 드롭되는 경험치 보석 | 스폰된 gem 엔티티 |
| `XpAccumulator { current: u32 }` | XP 누적기. 1-D 는 누적만, 레벨업은 1-E | Player |

#### 신규 시스템

| 시스템 | 기본값 | 동작 |
|---|---|---|
| `MagnetSystem` | pickup=20 / magnet=80 / attract=400 px/s | 픽업 반경 내 gem → XP 누적 + despawn; 자석 반경 내 gem → 플레이어 방향 이동 |

#### 변경 파일

| 파일 | 변경 |
|---|---|
| `crates/game/src/survivor/weapon.rs` | `WhipSystem` hit 루프: 적 사망 시 `spawn_xp_gem(world, pos, 1)` 호출 (despawn 전 위치 캐시) |
| `crates/game/src/survivor/world_setup.rs` | `spawn_player` 에 `XpAccumulator::default()` 컴포넌트 추가 |
| `crates/game/src/survivor/mod.rs` | `pub mod xp` + `LAYER_XP = 1 << 2` + 재수출 추가 |
| `crates/game/src/bin/survivor.rs` | 주석·임포트 갱신 + `MagnetSystem::default()` 등록 (WhipSystem 다음, HitFlashSystem 앞) |
| `crates/game/src/lib.rs` | 임포트 확장 + 신규 테스트 3건 |

#### 레이어 상수

```rust
pub const LAYER_XP: u32 = 1 << 2;
```

#### 콘솔 로그

XP 픽업 시 `"XP: {current}"` 출력 (HUD 도입 전 임시 확인용)

#### 테스트

game lib 13 통과 (직전 10 + 신규 3):
- `xp_gem_spawned_when_zombie_dies` — zombie 즉사 시 XpGem 1개 스폰 확인
- `xp_gem_attracted_within_magnet_radius` — 자석 반경 안의 gem 이 플레이어 방향으로 이동 확인
- `xp_gem_picked_up_in_range` — 픽업 반경 안 gem despawn + XpAccumulator.current += 1 확인

#### 검증

```bash
cargo build --workspace                     # ok
cargo test --workspace                      # engine 26 / game lib 13 / doc 2 통과
cargo build --release --workspace           # ok
```

#### 핵심 결정

- `XpAccumulator` 는 Player 컴포넌트로만 사용. 리소스로는 insert 하지 않음.
- borrow checker: Player 위치 캐시 → gems collect → to_pickup/to_attract 분류 → 적용 순서로 단계 분리.
- XpGem z=0.3 (zombie 0.5 아래, player 1.0 아래) — 겹칠 때 gem 이 가장 뒤에 그려짐.
- `spawn_xp_gem` 은 despawn 이후 호출 — `get::<Transform>` 으로 위치 캐시 후 `despawn` → `spawn_xp_gem` 순서. borrow 충돌 없음.

## 다음 작업: Phase 1-F — HUD + 사망/GameOver/R 재시작

---

### Phase 1-E — 레벨업 + 카드 선택 (2026-05-21)

#### 신규 파일

| 파일 | 내용 |
|---|---|
| `crates/game/src/survivor/levelup.rs` | `CardKind`, `PendingLevelUp`, `LevelUpSystem`, `apply_card` 헬퍼 |

#### XpAccumulator 확장

| 필드 | 기본값 | 설명 |
|---|---|---|
| `current: u32` | 0 | 현재 누적 XP |
| `level: u32` | 1 | 현재 레벨 |
| `next_threshold: u32` | 5 | L1→L2 임계치. `next_threshold(level) = 5 + 5×level` |

#### 레벨업 임계치

```
next_threshold(level) = 5 + 5 × level
L1→L2: 5, L2→L3: 10, L3→L4: 15 …
```

#### 카드 3장 (Whip 강화)

| 카드 | 효과 | 표시 |
|---|---|---|
| `WhipDamage` | `damage += 5.0` | `1=DMG+5` |
| `WhipArea` | `area_width += 20, area_height += 10` | `2=AREA UP` |
| `WhipCooldown` | `cooldown *= 0.85` | `3=CD-15%` |

#### 일시정지 가드 추가 시스템 (5개)

`PlayerMovementSystem`, `EnemyAiSystem`, `EnemySpawnSystem`, `WhipSystem`, `MagnetSystem`의 `System::run` 시작 부분에 다음 가드 추가:
```rust
if !matches!(world.resource::<GameState>(), Some(GameState::Playing)) {
    return;
}
```

가드 제외: `CameraFollowSystem`, `HitFlashSystem`, `LevelUpSystem`

#### PendingLevelUp 제거 패턴

`World::remove_resource` 미존재 → 패턴 (b) 선택: `consumed: bool` 필드.  
`consumed = true` 를 sentinel 로 삼아 재처리 방지. `has_pending = !consumed`.

#### 시스템 등록 순서 (Phase 1-E)

```
LevelUpSystem            ← 첫 번째 (상태 전환·키 입력)
PlayerMovementSystem
EnemyAiSystem
EnemySpawnSystem
CameraFollowSystem
WhipSystem
MagnetSystem
HitFlashSystem
```

#### 콘솔 로그

- `"LEVEL UP! (Level N) — Press: 1=DMG+5  2=AREA UP  3=CD-15%"` — 임계치 도달 시
- `"Whip upgraded: damage=X area=Y×Z cooldown=W"` — 카드 적용 후
- `"Resumed (XP=A, next threshold=B)"` — Playing 복귀 시

#### 테스트

game lib 16 통과 (직전 13 + 신규 3):
- `levelup_triggered_when_xp_reaches_threshold` — XP=5 → Paused + PendingLevelUp 삽입 확인
- `levelup_applies_card_whip_damage` — apply_card 직접 호출로 Whip.damage+5, level=2, threshold=15 확인
- `paused_blocks_enemy_movement` — Paused 상태에서 좀비 이동 없음 확인

기존 1-D 테스트 13건 모두 유지 (`GameState::Playing` 수동 삽입 추가).

#### 변경 파일

| 파일 | 변경 |
|---|---|
| `crates/game/src/survivor/levelup.rs` | 신규 |
| `crates/game/src/survivor/xp.rs` | `XpAccumulator` 확장 (level/next_threshold), MagnetSystem 가드 |
| `crates/game/src/survivor/player.rs` | PlayerMovementSystem 가드 |
| `crates/game/src/survivor/enemy.rs` | EnemyAiSystem 가드 |
| `crates/game/src/survivor/spawn.rs` | EnemySpawnSystem 가드 |
| `crates/game/src/survivor/weapon.rs` | WhipSystem 가드 |
| `crates/game/src/survivor/mod.rs` | `pub mod levelup` + 재수출 |
| `crates/game/src/bin/survivor.rs` | LevelUpSystem 등록 (첫 번째) |
| `crates/game/src/lib.rs` | 신규 테스트 3건 + 기존 setup 에 GameState::Playing 추가 |

#### 검증

```bash
cargo build --workspace                     # ok
cargo test --workspace                      # engine 26 / game lib 16 / doc 2 통과
cargo build --release --workspace           # ok
```

#### 핵심 결정

- `apply_card` 를 `LevelUpSystem` 의 `pub` 연관 함수로 분리 → 키 입력 없이 카드 효과만 단위 테스트 가능 (InputState::press 가 pub(crate) 라 외부 시뮬레이션 불가)
- `PendingLevelUp.consumed` sentinel: `World::remove_resource` 미존재, `HitFlash`의 `f32::NEG_INFINITY` 전례와 동일한 패턴 적용

---

### Phase 1-F — HUD + 사망 + GameOver + R 재시작 (2026-05-21)

#### 신규 파일

| 파일 | 내용 |
|---|---|
| `crates/game/src/survivor/hud.rs` | `GameStats { elapsed, kills }` 리소스, `HudSystem` |
| `crates/game/src/survivor/death.rs` | `EnemyContactDamageSystem`, `DeathSystem`, `RestartSystem`, `restart_world` 헬퍼 |

#### HUD (`HudSystem`)

- **모든 GameState** 에서 동작 (Playing/Paused/GameOver 모두 표시)
- 좌상단 `(10, 10)`: `MM:SS  Lv N  XP cur/max  HP cur/max  Kills N` 한 줄
- Paused + PendingLevelUp: 화면 중앙 (`320, 220`) "LEVEL UP!" 48px + 카드 3장 22px
- GameOver: 화면 중앙 (`310, 250`) "YOU DIED" 56px + `(310, 325)` "Press R to restart" 22px
- 좌표 근거: 800×600 viewport 기준 좌상단 `(10,10)`, 중앙 x≈310~320 (글자 폭 고려 어림값)

#### 적 접촉 데미지 (`EnemyContactDamageSystem`)

- Playing 가드 적용
- 1초 cooldown, 접촉 반경 25px, 적 1마리 이상 시 Player -10 HP
- `SpatialGrid` 직접 소유 (WhipSystem 패턴 답습)
- 이벤트성 로그: `"Player hit by N enemies (HP=X)"`

#### 사망 (`DeathSystem`)

- Playing 중, Player Health <= 0 이면 `GameState::GameOver` 전환

#### 재시작 (`RestartSystem` + `restart_world` 헬퍼)

- GameOver 중, R 키 → `restart_world(world)` 호출
- `restart_world`: Transform 보유 모든 엔티티 despawn → GameStats/PendingLevelUp/SpawnTimer 리셋 → `setup_survivor_world` 재호출 → GameState::Playing 복귀
- `restart_world` 를 `pub fn` 헬퍼로 분리 → 테스트에서 InputState 시뮬레이션 없이 직접 호출 가능

#### kills 카운트

- `WhipSystem` 적 사망 처리 직후 `GameStats.kills += 1` 누적
- 기존 매 발화 `println!("Whip: {} hits, {} killed")` 제거 — HudSystem.Kills 로 대체

#### 콘솔 로그 정리

- **제거**: 매 발화 `println!` (Whip hit/killed 집계, XP 픽업)
- **유지**: 이벤트성만 (`"Player hit by N enemies"`, `"YOU DIED"`, `"Restarted."`, `"Whip upgraded"`, `"Resumed"`, `"LEVEL UP!"`)

#### setup_survivor_world 추가

- `world_setup.rs` 에 `pub fn setup_survivor_world(world)` 신규 — `spawn_player` + `GameStats` 미존재 시 default 삽입
- `bin/survivor.rs` 및 `restart_world` 모두 이 함수를 단일 진입점으로 호출

#### 시스템 등록 순서 (Phase 1-F)

```
LevelUpSystem
PlayerMovementSystem
EnemyAiSystem
EnemySpawnSystem::default()
EnemyContactDamageSystem::default()  ← 신규
DeathSystem                          ← 신규
RestartSystem                        ← 신규
CameraFollowSystem::default()
WhipSystem::default()
MagnetSystem::default()
HitFlashSystem
HudSystem                            ← 신규 (마지막 — TextQueue push)
```

#### 테스트

game lib 19 통과 (직전 16 + 신규 3):
- `player_dies_when_hp_reaches_zero` — HP 0 강제 → DeathSystem → GameOver 확인
- `restart_clears_world_and_resumes` — restart_world 직접 호출 → Player=1, Zombie=0, XpGem=0, Playing 확인
- `enemy_contact_damages_player_after_cooldown` — 반경 15px 좀비 + dt=1.0 → HP 90.0 확인

#### 검증

```bash
cargo build --workspace                     # ok
cargo test --workspace                      # engine 26 / game lib 19 / doc 2 통과
cargo build --release --workspace           # ok
```

#### 핵심 결정

- `restart_world` 를 `pub fn` 헬퍼로 분리: `RestartSystem` 이 시스템 안에서 R 키 확인 후 호출, 테스트는 헬퍼를 직접 호출 (InputState::just_pressed 외부 시뮬레이션 불가)
- `setup_survivor_world` 로 초기화 단일 진입점 통일: 최초 기동과 재시작 모두 같은 함수 사용
- HUD 좌표: 800×600 viewport 기준 — 좌상단 `(10, 10)`, 중앙부 x≈310~320 (렌더 검증은 사용자 게이트)

---

## **Phase 1 Vertical Slice MVP 완료**

한 루프 **(이동 → 자동공격 → XP → 레벨업 → 사망/재시작)** 가 닫힘.

| sub-phase | 내용 |
|---|---|
| 1-A | 진입점 + Player + Camera follow |
| 1-B | Zombie 적 + 스폰 |
| 1-C | Whip 무기 + DamageSystem + HitFlash |
| 1-D | XpGem 드롭 + 자석 흡수 |
| 1-E | 레벨업 + 카드 선택 |
| 1-F | HUD + 사망 + GameOver + R 재시작 |

다음: **Phase 2 — 무기 풀 확장** (Magic Wand, Knife, Axe, Cross, Garlic, Holy Water, King Bible, Fire Wand, Lightning Ring + 공용 ProjectileSystem)

---

### Phase 2-A — WeaponInventory + Whip 마이그레이션 (2026-05-21)

#### 신규 파일

| 파일 | 내용 |
|---|---|
| `crates/game/src/survivor/inventory.rs` | `WeaponKind` enum, `WeaponSlot { kind, level, cooldown, elapsed }`, `WeaponInventory { slots: [Option<WeaponSlot>; 6] }` |

#### 핵심 타입

| 타입 | 설명 |
|---|---|
| `WeaponKind::Whip { damage, area_width, area_height }` | 무기 종류 + 파라미터. 2-B 에서 MagicWand 변종 추가 예정 |
| `WeaponSlot::tick(dt) -> bool` | dt 누적 후 발화 가능 여부 반환 + elapsed 초기화 |
| `WeaponInventory::with_whip_default()` | 슬롯 0 에 Whip(damage=10, area=120×60, cd=1.0) 초기화 |
| `WeaponInventory::whip_slot() / whip_slot_mut()` | 첫 Whip 슬롯 참조 헬퍼 |

#### 마이그레이션 내용

| 항목 | 변경 전 | 변경 후 |
|---|---|---|
| 무기 컴포넌트 | `Whip { level, damage, area_width, area_height, cooldown }` | `WeaponInventory { slots: [Option<WeaponSlot>; 6] }` |
| cooldown 트래킹 | `WhipSystem.elapsed: f32` 필드 | `WeaponInventory.slots[0].elapsed` |
| 발화 로직 | `WhipSystem` 이 elapsed 직접 누적 | `slot.tick(dt)` 호출 — 슬롯이 자체 추적 |
| 레벨업 강화 | `world.get_mut::<Whip>(pe)` 직접 수정 | `world.get_mut::<WeaponInventory>(pe)?.whip_slot_mut()?` 경로 |
| 플레이어 스폰 | `Whip::default()` 부착 | `WeaponInventory::with_whip_default()` 부착 |

#### 동작 변화

**없음.** Whip 1초 cooldown, 데미지 10, 좌/우 AABB 범위(120×60) 완전히 동일.

#### 변경 파일

| 파일 | 변경 |
|---|---|
| `crates/game/src/survivor/inventory.rs` | **신규** |
| `crates/game/src/survivor/weapon.rs` | `Whip` 컴포넌트 제거. `WhipSystem.elapsed` 필드 제거. `WeaponInventory` slot tick 경로로 변경 |
| `crates/game/src/survivor/levelup.rs` | `Whip` → `WeaponInventory::whip_slot_mut()` 경로로 강화 적용 |
| `crates/game/src/survivor/world_setup.rs` | `Whip::default()` → `WeaponInventory::with_whip_default()` |
| `crates/game/src/survivor/mod.rs` | `pub mod inventory` 추가. `Whip` 재수출 제거. `WeaponInventory/WeaponKind/WeaponSlot` 재수출 추가 |
| `crates/game/src/lib.rs` | 임포트 수정 (`Whip` → `WeaponInventory/WeaponKind`). `levelup_applies_card_whip_damage` 테스트를 inventory 경로로 수정 |

#### 테스트

game lib 21 통과 (직전 19 + 신규 2):
- `inventory_starts_with_whip_slot_0` — 슬롯 0 에 Whip, 나머지 5 슬롯 None 확인
- `weapon_slot_tick_returns_true_on_cooldown` — 0.5+0.5=1.0 누적 시 발화 + elapsed 리셋 확인

기존 테스트 수정 (1개):
- `levelup_applies_card_whip_damage` — `Whip` 직접 접근 → `WeaponInventory::whip_slot()` 경로로 변경

#### 검증

```bash
cargo build --workspace                     # ok
cargo test --workspace                      # engine 26 / game lib 21 / doc 2 통과
cargo build --release --workspace           # ok
```

#### 핵심 결정

- **슬롯 6개 고정 배열**: Vampire Survivors 원작 무기 슬롯 수. `[Option<WeaponSlot>; 6]` — heap 할당 없이 스택 상주
- **WeaponKind 단일 변종 경고**: `irrefutable_let_patterns` 경고 → `#[allow(...)]` 로 명시적 억제. 2-B 에서 MagicWand 변종 추가 시 자동 해소
- **시스템 등록 변경 없음**: `WhipSystem` 이름 유지, `survivor.rs` 등록 순서 그대로

다음: **Phase 2-B — ProjectileSystem + Magic Wand**
