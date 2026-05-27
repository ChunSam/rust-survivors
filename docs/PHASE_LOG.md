# Phase 진행 로그

Vampire Survivors 클론 개발 진행 상황. 각 sub-phase 완료 시 항목 추가.

---

## Phase 11-A — 데미지 숫자 (floating combat text) (2026-05-22)

> Phase 11 폴리쉬의 첫 항목. 적이 피격될 때마다 데미지 값이 적 위치 위로 떠올랐다가 페이드아웃.

### 신규 파일

| 파일 | 핵심 타입·함수 |
|---|---|
| `crates/game/src/survivor/damage_number.rs` | `DamageNumber { value, age, lifetime }` 컴포넌트, `spawn_damage_number()`, `DamageNumberSystem` (age 증가 + 위로 이동 + 만료 despawn) |

### 변경 파일

| 파일 | 변경 내용 |
|---|---|
| `survivor/damage.rs` | `apply_damage_to_enemy()` 가 피격 위치에서 `spawn_damage_number(world, p, damage)` 호출 |
| `survivor/hud.rs` | InGame 모드 마지막 블록 (#8) 추가 — `query2::<DamageNumber, Transform>` 으로 모든 데미지 숫자를 카메라 좌표(`pos - camera.position`)로 변환 후 `TextQueue` 에 push. 페이드는 알파 채널 `(1 - fade) * 255` 적용. 뷰포트 밖이면 skip. |
| `survivor/mod.rs` | `pub mod damage_number` + `DamageNumber`, `DamageNumberSystem`, `spawn_damage_number` 재수출 |
| `bin/survivor.rs` | `DamageNumberSystem` 등록 (HitFlashSystem → HudSystem 사이) |

### 동작

1. 무기 시스템이 `apply_damage_to_enemy(world, enemy, dmg)` 호출
2. 적 위치에서 `±8px` jitter 한 좌표 + y -16px 에 `DamageNumber` 엔티티 스폰 (lifetime 0.6s)
3. 매 프레임 `DamageNumberSystem` 이 `age += dt` + `position.y -= 40 * dt`
4. `HudSystem` 이 카메라 변환 + 알파 페이드 적용 + TextQueue push
5. age ≥ lifetime → despawn

### 디자인 결정

| 결정 | 이유 |
|---|---|
| 별도 컴포넌트 (Sprite 없이) | 텍스트 전용 — `Transform` 만 있으면 위치 추적 충분. Sprite 안 붙이면 SpriteRenderer 가 그리지 않으므로 인스턴스 버퍼 낭비 0 |
| HUD 안에서 렌더 | 텍스트 큐는 `HudSystem` 만 사용 → 카메라 변환 한 곳에 모으는 게 자연스러움. 별도 `DamageNumberRenderSystem` 분리 안 함 |
| 800×600 상수 사용 | 현재 윈도우 크기 고정. 추후 리사이즈 대응 시 `CameraFollowSystem::viewport` 같은 리소스 도입 후 통일 |
| jitter `±8px` | 같은 프레임 멀티히트 시 숫자 겹침 방지 (시각 노이즈) |
| 알파 페이드만 (크기 변화 X) | glyphon `Buffer` 는 metrics 가 고정. 매 프레임 새 Buffer 만들면 비용 큼. 알파만 변경하는 게 안전 |

### borrow checker 주의

- `query2<DamageNumber, Transform>` 가 `&World` 빌림 중에는 `world.resource_mut::<TextQueue>()` 호출 불가
  → 먼저 `Vec<(Vec2, f32, u8)>` 로 항목을 수집한 뒤, 별도 스코프에서 큐에 push
- `DamageNumberSystem` 도 동일 패턴: `query2` 로 entity 만 수집 → 별도 패스에서 `get_mut` + `despawn`

### 테스트 (+3 → 80)

| 테스트 | 내용 |
|---|---|
| `fade_progresses_from_zero_to_one` | `age/lifetime` 비율이 0→1 로 진행 + `age > lifetime` 시 1.0 clamp |
| `spawn_creates_entity_with_components` | `spawn_damage_number` 가 `DamageNumber + Transform` 둘 다 부착, y 정확히 `pos.y - 16`, x jitter `±8` 안 |
| `system_moves_up_and_despawns_after_lifetime` | dt 0.1 → 위로 이동 / dt 1.0 → 수명 초과 시 despawn |

```bash
cargo build --workspace                     # ok
cargo test --workspace                      # engine 26 / game lib 80 / doc 2 통과
```

#### 다음

**Phase 11-B** — 히트 플래시 개선 / 파티클 / SFX 중 택1

---

## Phase 10 — 다중 스테이지 3종 + StageSelect (2026-05-21) — **Phase 10 완료 / 풀 클론 핵심 시스템 완성**

> **Phase 10 완료 = Vampire Survivors 풀 클론 핵심 시스템 100% 구현**
> 향후: Phase 11 폴리쉬 (데미지 숫자, 히트 플래시 개선, SFX, BGM, 업적, 로컬라이제이션, macOS .app)

### 신규 파일

| 파일 | 핵심 타입·함수 |
|---|---|
| `crates/game/src/survivor/stage.rs` | `StageKind` (3종 enum), `SelectedStage` (리소스), `StageCursor` (리소스), `StageSelectSystem` |
| `assets/data/waves_mad_forest.ron` | MadForest 6 wave (zombie/bat/skeleton/ghost/mage/mantis/slime/plant/mummy/knight) |
| `assets/data/waves_inlaid_library.ron` | InlaidLibrary 6 wave (skeleton/ghost/mage/mantis/plant/slime/mummy/knight 우세) |
| `assets/data/waves_dairy_plant.ron` | DairyPlant 6 wave (slime/mummy/knight/mantis 우세, 최고 난이도) |

### 변경 파일

| 파일 | 변경 내용 |
|---|---|
| `survivor/meta.rs` | `SurvivorMode::StageSelect` variant 추가. `ModeTransitionSystem` — Title `KeyT` → StageSelect 진입, `StageSelect` match arm. InGame StageClear 전환 시 `unlocked_stages` 에 다음 스테이지 해금 + 저장. |
| `survivor/hud.rs` | `SurvivorMode::StageSelect` HUD (커서·해금 상태 표시). Title HUD 힌트 "C = CHAR  T = STAGE  S = SHOP" 로 갱신. |
| `survivor/world_setup.rs` | `SelectedStage::default()` + `StageCursor::default()` 최초 삽입. MetaSave.unlocked_stages 초기화. SpawnDirector waves 를 SelectedStage 기반으로 교체. |
| `survivor/mod.rs` | `pub mod stage` + `SelectedStage`, `StageCursor`, `StageKind`, `StageSelectSystem` 재수출. |
| `bin/survivor.rs` | `StageSelectSystem` 시스템 등록 (CharacterSelectSystem 다음). |

### 스테이지별 적 풀 차별화

| 스테이지 | 테마 | 특징 적 | 난이도 |
|---|---|---|---|
| Mad Forest | 숲 — 기본 | Zombie/Bat 시작, 점진적 확장 | 표준 (spawn 1.2s) |
| Inlaid Library | 도서관 언데드 | Skeleton/Ghost/Mage 우세, 빠른 시작 | 중간 (spawn 1.0s) |
| Dairy Plant | 공장 슬라임 | Slime/Mummy/Knight 조기 등장, 2마리 버스트 시작 | 최고 (spawn 0.9s, burst×2~14) |

### 해금 체인

`MadForest` (기본 해금) → 클리어 → `InlaidLibrary` 해금 → 클리어 → `DairyPlant` 해금

### 테스트 (+3 → 77)

| 테스트 | 내용 |
|---|---|
| `stage_count_is_three` | `StageKind::ALL.len() == 3` |
| `mad_forest_waves_load` | `MadForest.load_waves()` RON 파싱 성공 (len > 0) |
| `stage_prerequisite_chain` | `MadForest→None`, `Library→MadForest`, `Dairy→Library` |

### Phase 1~10 누적 시스템

10 무기 + 16 패시브 + 10종 적 + 보스 3 + 진화 8 + 픽업 5 + Title/Shop/CharacterSelect/StageSelect + 6 캐릭터 해금 + 3 스테이지 해금 + 메타 진행(영구 저장) — Vertical Slice → 풀 클론 핵심 루프 완성.

---

## Phase 9 — 캐릭터 6종 + CharacterSelect + 해금 (2026-05-21)

### 신규 파일

| 파일 | 핵심 타입·함수 |
|---|---|
| `crates/game/src/survivor/character.rs` | `CharacterKind` (6종 enum), `SelectedCharacter` (리소스), `CharacterCursor` (리소스), `CharacterSelectSystem` |

### 변경 파일

| 파일 | 변경 내용 |
|---|---|
| `survivor/meta.rs` | `SurvivorMode::CharacterSelect` variant 추가. `ModeTransitionSystem` Title 분기에서 `KeyC` → CharacterSelect 진입. CharacterSelect match arm 추가. |
| `survivor/hud.rs` | `SurvivorMode::CharacterSelect` HUD (커서·해금 상태·골드 표시). Title HUD에 "C = CHAR  S = SHOP" 힌트. |
| `survivor/world_setup.rs` | `spawn_player`: `SelectedCharacter` 리소스 기반 시작 인벤토리 + 스탯 보정. `setup_survivor_world`: `SelectedCharacter::default()` 최초 삽입. |
| `survivor/stats.rs` | `StatRecalcSystem`: 패시브·파워업 합산 후 `selected.0.apply_stats(&mut s)` 추가 (매 프레임 캐릭터 보정 반영). |
| `survivor/mod.rs` | `pub mod character` + `CharacterKind`, `SelectedCharacter`, `CharacterCursor`, `CharacterSelectSystem` 재수출. |
| `bin/survivor.rs` | `CharacterSelectSystem` 시스템 등록 (`ShopInputSystem` 다음). |

### 캐릭터 스펙

| 캐릭터 | 시작 무기 | 스탯 보정 | 해금 골드 |
|---|---|---|---|
| Antonio | Whip (전체 10종 로드아웃) | 없음 | 0 (기본) |
| Imelda | MagicWand | growth ×1.10 | 100 |
| Pasqualina | Knife | amount +1 | 300 |
| Gennaro | Axe | might +1.0 | 600 |
| Arca | FireWand | might +2.0 | 1000 |
| Porta | LightningRing | luck ×1.20 | 2000 |

### 테스트

- engine 26 / game lib 74 / doc 2 — 전체 통과
- 신규 3종: `character_kind_count_is_six`, `antonio_starter_inventory_has_whip`, `apply_stats_arca_increases_might`

---

## Phase 0 — 엔진 보강 (2026-05-20 완료)

서바이버 작업 시작 전 채워야 했던 엔진 공백을 채움.

### 신규 모듈

| 모듈 | 파일 | 핵심 타입·함수 |
|---|---|---|
| 카메라 | `rust-2d-engine/src/camera.rs` | `Camera { position, zoom }`, `Camera::new(pos, zoom)`, `view_proj(w, h) -> Mat4` |
| 충돌 그리드 | `rust-2d-engine/src/collision/grid.rs` | `Collider`, `CollisionLayer(u32)`, `SpatialGrid`, `CollisionGridSystem` |
| 충돌 쿼리 | `rust-2d-engine/src/collision/query.rs` | `SpatialGrid::query_radius`, `SpatialGrid::query_aabb` |
| 저장 | `rust-2d-engine/src/save.rs` | `save_path`, `save<T: Serialize>`, `load<T: DeserializeOwned>`, `SaveError` |

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
| `rust-2d-engine/src/renderer/text.rs` | `TextRenderer`, `TextQueue`, `DrawText` |
| `assets/fonts/NotoSansKR-Regular.ttf` | OpenType 한글 폰트 (SIL OFL 1.1) |
| `assets/fonts/OFL.txt` | SIL OFL 1.1 라이선스 전문 |

### 설계 결정

- `Cache` 소유권 배치: `TextRenderer` 구조체가 `cache` 필드 직접 소유 — `TextAtlas::new` 에 `&Cache` 가 필요하므로 먼저 생성 후 공유
- TextRenderer 가 `Cache` 를 `#[allow(dead_code)]` 로 보존 (실제로는 atlas 내부에서 참조)
- 텍스트 렌더 패스: `LoadOp::Load` 로 스프라이트 패스 위에 합성

### 검증

- 한글·영문 텍스트 출력은 survivor HUD/메뉴에서 검증
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
- 바이너리 확인: `game` · `survivor`

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

---

### Phase 2-B — ProjectileSystem + Magic Wand (2026-05-21)

#### 신규 파일

| 파일 | 핵심 타입·함수 |
|---|---|
| `crates/game/src/survivor/projectile.rs` | `Projectile`, `ProjectileSystem`, `spawn_projectile` |

#### 수정 파일

| 파일 | 변경 내용 |
|---|---|
| `crates/game/src/survivor/inventory.rs` | `WeaponKind::MagicWand { damage, projectile_speed, lifetime, pierce }` variant 추가. `with_starter_loadout()` 신설 (Whip slot[0] + MagicWand slot[1]). `with_whip_default` deprecated 래퍼로 유지. `magic_wand_slot_mut/magic_wand_slot` 헬퍼 추가 |
| `crates/game/src/survivor/weapon.rs` | `MagicWandSystem` 추가 — 매 cooldown 마다 가장 가까운 적 1마리에게 직선 투사체 발사. `spawn_projectile` 헬퍼 임포트 |
| `crates/game/src/survivor/world_setup.rs` | `with_whip_default()` → `with_starter_loadout()` |
| `crates/game/src/survivor/mod.rs` | `pub mod projectile` 추가. `Projectile/ProjectileSystem/spawn_projectile/MagicWandSystem` 재수출 |
| `crates/game/src/bin/survivor.rs` | `MagicWandSystem::default()` + `ProjectileSystem::default()` 등록 (WhipSystem 다음) |
| `crates/game/src/lib.rs` | 신규 임포트 + 3개 테스트 추가 + `levelup_applies_card_whip_damage` irrefutable let 수정 |

#### 신규 시스템: MagicWandSystem

- 매 cooldown(1.2초)마다 SpatialGrid 로 반경 400px 안에서 가장 가까운 적 1마리 탐색.
- `spawn_projectile` 헬퍼를 통해 연보라색(0.9, 0.7, 1.0) 10×10 투사체 스폰.
- 이동·충돌·데미지·despawn 은 ProjectileSystem 이 전담.

#### ProjectileSystem hit 처리 방식

WhipSystem 과 별도 구현 (중복 허용). 이유:
- Whip 은 AABB 배치 판정(좌/우 부채꼴), 투사체는 이동 중인 점(Circle) 판정 — 알고리즘이 달라 공유 시 오히려 복잡.
- 각 시스템이 SpatialGrid 를 직접 소유(PhysicsSystem 패턴)하므로 별도 소유가 자연스러움.
- Trade-off: HitFlash·XpGem·kills 처리 코드가 중복(~30줄). 추후 hit_handler 헬퍼 함수로 추출 가능하나 현 단계에서는 가독성을 우선.

#### 테스트

game lib 26 통과 (직전 21 + 신규 3 + projectile.rs 내 2):
- `projectile_expires_after_lifetime` — lifetime 소진 후 despawn 확인
- `projectile_damages_zombie_on_collision` — 투사체 이동 후 zombie 충돌 데미지 확인
- `magic_wand_spawns_projectile_after_cooldown` — cooldown 경과 후 투사체 1개 스폰 확인

#### 검증

```bash
cargo build --workspace                     # ok
cargo test --workspace                      # engine 26 / game lib 26 / doc 2 통과
cargo build --release --workspace           # ok
```

#### 핵심 결정

- **with_starter_loadout**: 스타터 로드아웃을 단일 함수로 관리. `with_whip_default` 는 deprecated 래퍼로 하위 호환 유지
- **ProjectileSystem 독립 grid**: MagicWandSystem 도 grid 를 소유 — 같은 프레임에 grid 가 2회 rebuild 되지만 엔티티 수가 적을 때는 무시할 수 있는 비용
- **pierce=0 의미**: 1마리 타격 후 즉시 despawn (관통 없음). pierce=1 이면 2마리 관통.

다음: **Phase 2-C — Knife + Axe + ProjectileBehavior**

---

### Phase 2-C — Knife + Axe + ProjectileBehavior 일반화 (2026-05-21)

#### ProjectileBehavior enum 도입

```rust
pub enum ProjectileBehavior {
    Straight,               // velocity 그대로 직선 이동
    Arc { gravity: f32 },   // velocity.y += gravity * dt (포물선)
}
```

- `Projectile` 컴포넌트에 `pub behavior: ProjectileBehavior` 필드 추가
- `ProjectileSystem::run` 이 매 프레임 behavior 에 따라 velocity 갱신 후 이동
- `Projectile.velocity` 도 매 프레임 갱신 → Arc 투사체가 실제 포물선 궤적을 그림
- **Y 축**: `(0,0)=좌상단, y↑=아래`. 위로 던지기 = `velocity.y < 0`, 중력 = `gravity > 0`

#### 신규 헬퍼 함수

| 함수 | 설명 |
|---|---|
| `spawn_projectile(...)` | `Straight` 고정 (기존 호출자 변경 없음) |
| `spawn_projectile_ex(..., behavior)` | behavior 를 명시적으로 지정 |

#### WeaponKind 신규 variant

| variant | 파라미터 | 설명 |
|---|---|---|
| `Knife` | `damage, projectile_speed, lifetime, pierce, amount, spread_radians` | 가장 가까운 적 방향 `amount` 개 직선 투사체. `amount > 1` 이면 `spread_radians` 범위 부채꼴 분산 |
| `Axe` | `damage, initial_speed, gravity, lifetime, pierce, amount` | `velocity.y = -initial_speed` 로 위로 던져 `Arc` 중력으로 떨어지는 포물선 투사체 |

#### 스타터 로드아웃 (with_starter_loadout)

| 슬롯 | 무기 | cooldown | 주요 파라미터 |
|---|---|---|---|
| 0 | Whip | 1.0s | damage=10, area=120×60 |
| 1 | MagicWand | 1.2s | damage=8, speed=300, pierce=0 |
| 2 | Knife | 0.8s | damage=6, speed=400, amount=1, spread=0.3rad |
| 3 | Axe | 1.5s | damage=12, init_speed=250, gravity=600, pierce=2 |
| 4–5 | — | — | 비어 있음 |

#### 신규 시스템

| 시스템 | 동작 |
|---|---|
| `KnifeSystem` | cooldown → grid rebuild → 가장 가까운 적 탐색 → amount 개 `Straight` 투사체 발사 (흰색) |
| `AxeSystem` | cooldown → amount 개 `Arc { gravity }` 투사체 발사 (갈색). 적 방향 탐색 없음 — 항상 위로 던짐 |

#### 시스템 등록 순서 (Phase 2-C)

```
LevelUpSystem
PlayerMovementSystem
EnemyAiSystem
EnemySpawnSystem
EnemyContactDamageSystem
DeathSystem
RestartSystem
CameraFollowSystem
WhipSystem
MagicWandSystem
KnifeSystem::default()       ← 신규
AxeSystem::default()         ← 신규
ProjectileSystem
MagnetSystem
HitFlashSystem
HudSystem
```

#### 변경 파일

| 파일 | 변경 |
|---|---|
| `crates/game/src/survivor/projectile.rs` | `ProjectileBehavior` enum 추가. `Projectile.behavior` 필드. `ProjectileSystem` velocity 갱신 로직. `spawn_projectile_ex` 헬퍼 추가 |
| `crates/game/src/survivor/inventory.rs` | `WeaponKind::Knife`, `WeaponKind::Axe` variant 추가. `knife_slot_mut/axe_slot_mut` 헬퍼. `with_starter_loadout` slot[2]/[3] 추가 |
| `crates/game/src/survivor/weapon.rs` | `KnifeSystem`, `AxeSystem` 추가. `spawn_projectile_ex` 임포트 |
| `crates/game/src/survivor/mod.rs` | `KnifeSystem`, `AxeSystem`, `spawn_projectile_ex`, `ProjectileBehavior` 재수출 |
| `crates/game/src/bin/survivor.rs` | `KnifeSystem::default()`, `AxeSystem::default()` 등록 |
| `crates/game/src/lib.rs` | 임포트 확장 + 신규 테스트 3건 |

#### 테스트

game lib 29 통과 (직전 26 + 신규 3):
- `knife_spawns_projectile_after_cooldown` — cooldown 0.8 → dt=1.0 → 투사체 1개 스폰 확인
- `axe_projectile_arcs_downward` — Arc 투사체 dt=1.0 후 velocity.y ≈ 300 (중력 적용 확인)
- `inventory_starter_loadout_has_four_slots_filled` — slot[0..3] 모두 Some, slot[4..5] None. Whip/MagicWand/Knife/Axe 순서 확인

#### 검증

```bash
cargo build --workspace                     # ok
cargo test --workspace                      # engine 26 / game lib 29 / doc 2 통과
cargo build --release --workspace           # ok
```

#### 핵심 결정

- `ProjectileBehavior` enum 으로 이동 방식 분기: `Straight` 는 velocity 불변, `Arc` 는 velocity.y += gravity*dt. velocity 도 갱신해야 포물선이 점점 빨라지는 물리 효과 표현 가능.
- `spawn_projectile` 은 `Straight` 기본으로 고정 → 2-B MagicWand 호출 코드 변경 없음.
- `AxeSystem` 은 적 탐색 없이 항상 위로 발사 → Vampire Survivors Axe 원작 동작 재현 (자동으로 포물선이 넓게 커버).
- `KnifeSystem` spread: `amount == 1` 이면 정방향 1발, `> 1` 이면 `-spread/2 ~ +spread/2` 균등 분산.

다음: **Phase 2-D — Cross + Fire Wand**

---

### Phase 2-D — Cross + Fire Wand + Inventory Vec 변경 (2026-05-21)

#### 신규 컴포넌트 / 시스템

- `ProjectileBehavior::Boomerang { return_at, elapsed, returned }` — `elapsed` 가 `return_at` 에 도달하면 velocity 반전. `returned` 플래그로 1회만 반전. `ProjectileSystem` 이 매 프레임 `Projectile.behavior` 를 다시 set 해 상태 갱신.
- `WeaponKind::Cross { damage, projectile_speed, lifetime, pierce, amount, return_at }` — 황금색 부메랑 투사체. pierce 3 (관통하며 돌아옴).
- `WeaponKind::FireWand { damage, projectile_speed, lifetime, pierce }` — 주황색 단발 큰 데미지. 랜덤 zombie 타깃.
- `CrossSystem` (KnifeSystem 패턴 답습 + Boomerang behavior)
- `FireWandSystem` (target_radius 400 안 후보 적 모두 수집 → `rand` 로 1마리 랜덤 선택 → Straight 단발 발사)

#### 인프라 변경

- `WeaponInventory.slots`: `[Option<WeaponSlot>; 6]` → `Vec<WeaponSlot>` 로 변경. 동적 push, 6 무기 모두 보유.
- 모든 slot 헬퍼(`whip_slot_mut` / `magic_wand_slot_mut` / `knife_slot_mut` / `axe_slot_mut`)에 `cross_slot_mut` / `fire_wand_slot_mut` 추가.
- `with_starter_loadout`: 6 무기 모두 push (Whip / MagicWand / Knife / Axe / Cross / FireWand).
- 기존 `inventory_starter_loadout_has_four_slots_filled` 테스트는 `inventory_starter_loadout_has_six_weapons` 로 갱신 (None 검증 제거).

#### 시스템 등록 순서

```
LevelUpSystem
PlayerMovementSystem
EnemyAiSystem
EnemySpawnSystem
EnemyContactDamageSystem
DeathSystem
RestartSystem
CameraFollowSystem
WhipSystem
MagicWandSystem
KnifeSystem
AxeSystem
CrossSystem::default()       ← 신규
FireWandSystem::default()    ← 신규
ProjectileSystem
MagnetSystem
HitFlashSystem
HudSystem
```

#### 테스트

game lib 32 통과 (직전 29 + 신규 3):
- `inventory_starter_loadout_has_six_weapons`
- `cross_spawns_boomerang_projectile`
- `boomerang_velocity_reverses_after_return_at`
- (추가) `fire_wand_spawns_projectile_after_cooldown`

#### 검증

```bash
cargo build --workspace                     # ok
cargo test --workspace                      # engine 26 / game lib 32 / doc 2 통과
cargo build --release --workspace           # ok
```

#### 핵심 결정

- `Boomerang` 의 상태(`elapsed` / `returned`)는 `ProjectileBehavior` enum variant 안에 저장. `ProjectileSystem` 이 캐시 후 갱신된 behavior 를 `Projectile.behavior` 에 다시 write — Straight/Arc 에는 영향 없음.
- 무기 슬롯 6 한도(`[Option<_>; 6]`) 대신 `Vec` 채택 → 학습 단계에서 슬롯 관리 UX 없이도 모든 무기를 보유한 *시각 검증 가능*. 원작의 6 슬롯 게이트는 추후 Phase 8 메뉴 작업에서 도입 가능.
- `FireWand` 의 *랜덤 타깃* 은 `rand::seq::SliceRandom::choose` 사용. `MagicWand` / `Knife` / `Cross` 의 *가장 가까운 적* 패턴과 의도적 차별화.

다음: **Phase 2-E — Garlic + Holy Water (지속 area damage)**

---

### Phase 2-E — Garlic + Holy Water (2026-05-21)

지속 area-damage 패턴 두 종류 추가.

#### 신규 파일

| 파일 | 내용 |
|---|---|
| `crates/game/src/survivor/area.rs` | `HolyWaterPool` 컴포넌트, `GarlicSystem`, `HolyWaterSystem`, `HolyWaterPoolSystem`, `spawn_holy_water_pool` |

#### 수정 파일

| 파일 | 변경 내용 |
|---|---|
| `crates/game/src/survivor/inventory.rs` | `WeaponKind::Garlic`, `WeaponKind::HolyWater` 추가; `garlic_slot_mut/slot`, `holy_water_slot_mut/slot` 헬퍼; `with_starter_loadout` 에 Garlic + HolyWater 슬롯 추가 (8개 무기) |
| `crates/game/src/survivor/mod.rs` | `pub mod area;` + 재수출 추가 |
| `crates/game/src/bin/survivor.rs` | `GarlicSystem`, `HolyWaterSystem`, `HolyWaterPoolSystem` 시스템 등록 |
| `crates/game/src/lib.rs` | 3개 신규 테스트 + import 갱신 |

#### 시스템 등록 순서

```
LevelUpSystem
PlayerMovementSystem
EnemyAiSystem
EnemySpawnSystem
EnemyContactDamageSystem
DeathSystem
RestartSystem
CameraFollowSystem
WhipSystem
MagicWandSystem
KnifeSystem
AxeSystem
CrossSystem
FireWandSystem
GarlicSystem::default()        ← 신규 (오라)
HolyWaterSystem                ← 신규 (풀 스폰)
HolyWaterPoolSystem::default() ← 신규 (풀 tick + lifetime)
ProjectileSystem
MagnetSystem
HitFlashSystem
HudSystem
```

#### 테스트

game lib 35 통과 (직전 32 + 신규 3):
- `garlic_damages_zombies_in_radius`
- `holy_water_spawns_pool_after_cooldown`
- `pool_damages_zombie_on_tick`

#### 검증

```bash
cargo build --workspace                     # ok
cargo test --workspace                      # engine 26 / game lib 35 / doc 2 통과
cargo build --release --workspace           # ok
```

#### 핵심 결정

- `GarlicSystem` 은 `SpatialGrid::query_radius` 로 오라 반경 안 모든 적을 한 번에 타격. Whip 의 AABB 쿼리와 달리 완전한 원형 판정.
- `HolyWaterSystem` 은 풀 *스폰만* 담당. 풀의 lifetime 감소 + tick 데미지는 `HolyWaterPoolSystem` 이 분리 처리 — 역할 분리로 각 시스템이 단일 책임.
- `HolyWaterPool` 은 ECS 컴포넌트로 저장. grid rebuild 는 `HolyWaterPoolSystem` 이 직접 소유한 `SpatialGrid` 로.
- HitFlash + 사망 처리 코드가 WhipSystem / ProjectileSystem / GarlicSystem / HolyWaterPoolSystem 4곳에 중복. 향후 `apply_damage_to_zombie(world, entity, damage)` 헬퍼로 추출 권장.

다음: **Phase 2-F — King Bible + Lightning Ring**

---

### Phase 2-F — King Bible + Lightning Ring + damage 헬퍼 추출 (2026-05-21)

#### 신규 파일

| 파일 | 내용 |
|---|---|
| `crates/game/src/survivor/damage.rs` | `apply_damage_to_zombie(world, zombie, damage) -> bool` 헬퍼 — HitFlash + XpGem 드롭 + despawn 통합 |
| `crates/game/src/survivor/bible.rs` | `OrbitingBook` 컴포넌트, `KingBibleSystem`(책 스폰), `OrbitingBookSystem`(회전+tick damage), `spawn_book` 헬퍼 |
| `crates/game/src/survivor/lightning.rs` | `LightningFlash` 컴포넌트, `LightningRingSystem`(랜덤 zombie area damage), `LightningFlashSystem`(flash lifetime), `spawn_lightning_flash` 헬퍼 |

#### damage 헬퍼 추출

WhipSystem / ProjectileSystem / GarlicSystem / HolyWaterPoolSystem 4곳에 동일하게 복사되어 있던 ~15줄의 hit 처리 블록을 `apply_damage_to_zombie` 단일 함수로 추출.

```rust
// 변경 전 (4곳 반복)
let original = ...sprite 빨강...;
let died = ...Health::take_damage...;
if died { despawn + spawn_xp_gem } else { HitFlash 부착 }

// 변경 후 (4곳 모두)
if apply_damage_to_zombie(world, zombie, damage) { killed += 1; }
```

감소 줄 수: 약 60줄 (4 시스템 × ~15줄) → 4줄 (헬퍼 호출 1줄 × 4)

#### King Bible 동작

| 파라미터 | 기본값 | 설명 |
|---|---|---|
| damage | 6.0 | tick 당 데미지 |
| book_count | 2 | 스폰 권 수 |
| radius | 60.0 px | 플레이어 중심 회전 반경 |
| angular_speed | 3.0 rad/s | 회전 속도 |
| lifetime | 5.0 s | 책 활성 시간 |
| tick_cooldown | 0.3 s | 데미지 tick 간격 |
| hit_radius | 16.0 px | 충돌 판정 반경 |
| cooldown | 8.0 s | 새 책 세트 스폰 주기 |

- `KingBibleSystem`: cooldown 마다 `book_count` 권 각도 균등 분산하여 `OrbitingBook` 엔티티 스폰
- `OrbitingBookSystem`: 매 프레임 Player 위치 기준 회전 이동 + lifetime 감소. tick_cooldown 마다 SpatialGrid 로 hit_radius 안 zombie에게 area damage

#### Lightning Ring 동작

| 파라미터 | 기본값 | 설명 |
|---|---|---|
| damage | 30.0 | area 데미지 |
| strike_count | 1 | 타격 zombie 수 |
| hit_radius | 40.0 px | 각 strike 의 area 반경 |
| cooldown | 4.0 s | 발화 주기 |
| target_radius | 600.0 px | 후보 탐색 반경 |

- cooldown 마다 `target_radius` 안 zombie 중 `strike_count` 마리 랜덤 선택
- 각 타깃 위치에 `spawn_lightning_flash` (노란 40×40, 0.15초 lifetime) 스폰
- 같은 위치에서 `query_radius(hit_radius)` 로 추가 area damage

#### 시작 로드아웃 (10 무기)

| 슬롯 | 무기 | cooldown |
|---|---|---|
| 0 | Whip | 1.0s |
| 1 | MagicWand | 1.2s |
| 2 | Knife | 0.8s |
| 3 | Axe | 1.5s |
| 4 | Cross | 1.8s |
| 5 | FireWand | 2.5s |
| 6 | Garlic | 0.5s |
| 7 | HolyWater | 3.0s |
| 8 | KingBible | 8.0s |
| 9 | LightningRing | 4.0s |

#### 시스템 등록 순서 (Phase 2-F)

```
... (기존 동일)
GarlicSystem
HolyWaterSystem
HolyWaterPoolSystem
KingBibleSystem              ← 신규 (책 스폰)
OrbitingBookSystem::default()← 신규 (책 회전 + tick)
LightningRingSystem::default()← 신규 (랜덤 적 즉시 area damage)
LightningFlashSystem         ← 신규 (flash lifetime)
ProjectileSystem
...
```

#### 변경 파일

| 파일 | 변경 |
|---|---|
| `crates/game/src/survivor/damage.rs` | **신규** — `apply_damage_to_zombie` 헬퍼 |
| `crates/game/src/survivor/bible.rs` | **신규** — `OrbitingBook`, `KingBibleSystem`, `OrbitingBookSystem`, `spawn_book` |
| `crates/game/src/survivor/lightning.rs` | **신규** — `LightningFlash`, `LightningRingSystem`, `LightningFlashSystem`, `spawn_lightning_flash` |
| `crates/game/src/survivor/inventory.rs` | `WeaponKind::KingBible`, `WeaponKind::LightningRing` 추가; 헬퍼 4개; `with_starter_loadout` 에 슬롯 2개 추가 (10무기); 내부 테스트 갱신 |
| `crates/game/src/survivor/weapon.rs` | `WhipSystem` hit 루프 → `apply_damage_to_zombie` 1줄로 축소 |
| `crates/game/src/survivor/projectile.rs` | `ProjectileSystem` hit 루프 → `apply_damage_to_zombie` 1줄로 축소 |
| `crates/game/src/survivor/area.rs` | `GarlicSystem` + `HolyWaterPoolSystem` hit 루프 → `apply_damage_to_zombie` 1줄로 축소 |
| `crates/game/src/survivor/mod.rs` | `pub mod bible/damage/lightning` + 재수출 추가 |
| `crates/game/src/bin/survivor.rs` | 시스템 4개 등록. docstring 갱신 |
| `crates/game/src/lib.rs` | import 갱신 + 신규 테스트 3건 + `inventory_starter_loadout_has_six_weapons` 10 슬롯으로 갱신 |

#### 테스트

game lib 38 통과 (직전 35 + 신규 3):
- `king_bible_spawns_books_after_cooldown` — dt=9.0 → cooldown(8.0) 초과 → OrbitingBook 2권 스폰 확인
- `orbiting_book_rotates_and_damages_zombie` — tick_cooldown 초과 → zombie HP 감소 확인; book.angle 갱신 확인
- `lightning_ring_strikes_random_zombie` — dt=5.0 → cooldown(4.0) 초과 → LightningFlash 1개 이상 스폰 + zombie HP 감소 확인

#### 검증

```bash
cargo build --workspace                     # ok
cargo test --workspace                      # engine 26 / game lib 38 / doc 2 통과
cargo build --release --workspace           # ok
```

#### 핵심 결정

- `apply_damage_to_zombie` 는 `damage.rs` 별도 파일로 분리 — 순환 의존 회피. `weapon.rs` 의 `HitFlash` 를 참조하므로 `super::weapon::HitFlash` 로 접근.
- `OrbitingBookSystem` 의 borrow checker 처리: books 상태를 먼저 Vec 으로 collect → lifetime 갱신(get_mut) → book_updates 적용 → tick_hits 적용 순서로 borrow 를 단계적으로 분리.
- `LightningRingSystem` 의 flash 스폰: `spawn_lightning_flash` 가 새 엔티티를 생성하지만 grid 에 등록되지 않음 — 그 프레임의 area damage 계산은 이미 rebuild 된 grid 로 수행하므로 충돌 없음.
- `OrbitingBook` 의 `tick_hits` 는 회전 *이후* 위치 기반이지만, grid 는 회전 *이전* 위치 기준으로 rebuild — 매우 빠른 회전이 아닌 한 오차 무시 가능.

다음: **Phase 2-G — weapons.ron 외부 데이터 + 카드 풀 확장 (2026-05-21)**

---

### Phase 2-G — weapons.ron 외부 데이터 + 카드 풀 확장 (2026-05-21) — Phase 2 완료

#### 신규 파일

| 파일 | 내용 |
|---|---|
| `crates/game/src/survivor/data.rs` | `WeaponDef` (평탄 struct, 모든 무기별 필드 Option), `WeaponsDataFile`, `load_starter_weapons()` |
| `assets/data/weapons.ron` | 10 무기의 시작 스탯 RON 정의 (`include_str!` 로 컴파일 타임 내장) |

#### 수정 파일

| 파일 | 변경 내용 |
|---|---|
| `crates/game/Cargo.toml` | `serde = { workspace = true }`, `ron = { workspace = true }` 추가 (워크스페이스 이미 등록) |
| `crates/game/src/survivor/inventory.rs` | `with_starter_loadout`: 10 무기 하드코딩 제거 → `data::load_starter_weapons()` 위임 |
| `crates/game/src/survivor/levelup.rs` | `CardKind`: 3개 → 29개 (10 무기 × 2~3 카드). `ALL_CARDS` 상수 추가. `LevelUpSystem::run` 카드 선택을 `rand::choose_multiple` 랜덤으로 전환. `apply_card` 29 variant 전체 처리. `label()` 29개 정의 |
| `crates/game/src/survivor/mod.rs` | `pub mod data;`, `pub use data::load_starter_weapons;` 추가 |

#### WeaponDef 설계 선택

**평탄 struct (flat struct)** 채택. 모든 무기별 필드를 `Option<f32>` / `Option<u8>` 로 단일 구조체에 수평 나열.

- 장점: RON 직렬화 단순 (enum 직렬화 없이 `Deserialize` derive 만으로 파싱), 무기 추가 시 필드만 추가.
- 단점: 사용하지 않는 필드가 `None` 으로 남음 (RON 파일이 약간 장황). 무기당 21개 필드 중 종류에 따라 4~7개만 `Some`.
- 대안(enum 직렬화)과 비교: RON 은 `WeaponKind` enum 을 태그로 직렬화 가능하나, 각 variant 의 named field 를 RON 에서 파싱하려면 `#[serde(tag = "...")]` 등 설정이 복잡해짐. 학습 목적에는 평탄 struct 가 더 명확.

#### CardKind variant 총 29개

| 무기 | 카드 (개수) |
|---|---|
| Whip | WhipDamage / WhipArea / WhipCooldown (3) |
| MagicWand | MagicWandDamage / MagicWandSpeed / MagicWandCooldown (3) |
| Knife | KnifeDamage / KnifeAmount / KnifeCooldown (3) |
| Axe | AxeDamage / AxePierce / AxeCooldown (3) |
| Cross | CrossDamage / CrossReturnAt / CrossCooldown (3) |
| FireWand | FireWandDamage / FireWandCooldown (2) |
| Garlic | GarlicDamage / GarlicRadius / GarlicCooldown (3) |
| HolyWater | HolyWaterDamage / HolyWaterDropCount / HolyWaterCooldown (3) |
| KingBible | KingBibleDamage / KingBibleBookCount / KingBibleCooldown (3) |
| LightningRing | LightningDamage / LightningStrikeCount / LightningCooldown (3) |

#### apply_card 매치 절

29개 variant 대응. 각 카드 적용 후 `slot.level += 1`. cooldown 카드는 `slot.cooldown *= 0.85`.

강화 효과값:
- Damage: +4~+8 (무기 기본 데미지에 비례)
- Cooldown: *0.85
- Area/Radius: +15~+20 px
- Amount/Count/Pierce: +1
- ProjectileSpeed: +60 px/s
- ReturnAt: +0.1 초

#### 테스트

game lib 41 통과 (직전 38 + 신규 3):
- `weapons_ron_loads_ten_weapons` (`data.rs` 내) — `load_starter_weapons()` 호출 → 10개 슬롯, Whip/MagicWand/.../LightningRing 순서 확인, 대표 스탯 검증
- `apply_card_magic_wand_damage_increases` (`levelup.rs` 내) — MagicWandDamage 적용 → damage 8.0 → 12.0, level 1 → 2 확인
- `apply_card_lightning_strike_count_increases` (`levelup.rs` 내) — LightningStrikeCount 적용 → strike_count 1 → 2, level 1 → 2 확인

기존 38 테스트 모두 유지 (RON 로딩 후 동일 스탯 보장 — `inventory_starts_with_whip_slot_0` 포함).

#### 검증

```bash
cargo build --workspace                     # ok
cargo test --workspace                      # engine 26 / game lib 41 / doc 2 통과
cargo build --release --workspace           # ok
```

---

## **Phase 2 무기 풀 확장 완료**

| sub-phase | 내용 |
|---|---|
| 2-A | WeaponInventory + Whip 마이그레이션 |
| 2-B | ProjectileSystem + Magic Wand |
| 2-C | Knife + Axe + ProjectileBehavior |
| 2-D | Cross + Fire Wand + Inventory Vec 변경 |
| 2-E | Garlic + Holy Water |
| 2-F | King Bible + Lightning Ring + damage 헬퍼 추출 |
| 2-G | weapons.ron 외부 데이터 + 카드 풀 29개 확장 |

다음: **Phase 3 — PlayerStats + 패시브 16종**

---

## Phase 3-A — PlayerStats 16 필드 확장 + 시스템 통합 (2026-05-21)

### 개요

PlayerStats 를 1 필드(move_speed) → 16 필드로 확장하고, 모든 무기·시스템에 stats 를 반영.
Phase 3-A 의 기본값(항등원)이면 게임플레이 변화 0. Phase 3-B 패시브 도입 시 즉시 반영.

### PlayerStats 16 필드

| 필드 | 기본값 | 의미 |
|---|---|---|
| `might` | 0.0 | 데미지 가산 (damage += might) |
| `area` | 1.0 | hitbox/오라 반경 곱 |
| `projectile_speed` | 1.0 | 투사체 속도 곱 |
| `duration` | 1.0 | 투사체/필드 lifetime 곱 |
| `amount` | 0 | 발사 수 가산 (Knife/HolyWater/KingBible/Lightning) |
| `cooldown` | 1.0 | cooldown 곱 (< 1 이면 빨라짐) |
| `max_health` | 100.0 | 최대 체력 |
| `recovery` | 0.0 | 초당 자연 회복 |
| `armor` | 0.0 | 받는 데미지 감쇄 |
| `move_speed` | 200.0 | 이동 속도 px/s |
| `magnet` | 1.0 | 자석/픽업 반경 곱 |
| `luck` | 1.0 | 드롭률 placeholder |
| `growth` | 1.0 | XP 획득 배율 |
| `greed` | 1.0 | 골드 배율 placeholder |
| `curse` | 1.0 | 적 강화 placeholder |
| `revival` | 0 | 부활 횟수 placeholder |

### 신규 파일

| 파일 | 내용 |
|---|---|
| `crates/game/src/survivor/stats.rs` | `StatRecalcSystem` (stub no-op) + `read_player_stats` 헬퍼 |

### 기존 파일 변경

| 파일 | 변경 내용 |
|---|---|
| `player.rs` | `PlayerStats` 1 → 16 필드, `#[derive(Clone, PartialEq)]` 추가 |
| `inventory.rs` | `WeaponSlot::tick_with_cooldown_multiplier(dt, cd_multiplier)` 추가 |
| `health.rs` | `HealthRegenSystem` 신규 추가 (recovery stat 처리) |
| `mod.rs` | `stats` 모듈 + `HealthRegenSystem`, `StatRecalcSystem`, `read_player_stats` export |
| `weapon.rs` | WhipSystem/MagicWand/Knife/Axe/Cross/FireWand — 6종 stats 적용 |
| `area.rs` | GarlicSystem/HolyWaterSystem — 2종 stats 적용 |
| `bible.rs` | KingBibleSystem — 1종 stats 적용 |
| `lightning.rs` | LightningRingSystem — 1종 stats 적용 |
| `death.rs` | EnemyContactDamageSystem — armor 감쇄 적용 |
| `xp.rs` | MagnetSystem — magnet 반경 곱, growth XP 배율 적용 |
| `bin/survivor.rs` | StatRecalcSystem(첫) + HealthRegenSystem 등록 |

### 설계 결정

- **WeaponSlot::tick_with_cooldown_multiplier** 채택: `slot.cooldown` 필드는 원본 유지, 비교 시점에만 stats.cooldown 곱 → slot 자체를 변경하지 않아 레벨업 카드 강화와 독립적으로 동작
- **read_player_stats 헬퍼**: `&World` 불변 빌림 → 이후 `get_mut` 호출과 borrow 충돌 없음. 각 시스템이 매 프레임 시작에 stats 스냅샷 확보
- **HolyWaterPool, OrbitingBook**: 생성 시점의 stats 반영 → 이후 stats 변경에 영향 안 받음 (의도적)
- **StatRecalcSystem**: 첫 시스템으로 등록. Phase 3-B 패시브 합산 로직 추가 예정

### 검증

```bash
cargo build --workspace                     # ok
cargo test --workspace                      # engine 26 / game lib 44 / doc 2 통과
cargo build --release --workspace           # ok
```

### 다음

**Phase 3-B** — PassiveKind 16개 + PassiveInventory + StatRecalcSystem 합산 로직

---

## Phase 3-B — PassiveInventory + StatRecalcSystem (2026-05-21)

Phase 3-A 에서 stub no-op 이었던 StatRecalcSystem 에 PassiveInventory 합산 로직 구현.

### 신규 파일

| 파일 | 내용 |
|---|---|
| `crates/game/src/survivor/passive.rs` | `PassiveKind` 16 variant, `PassiveSlot { kind, level }`, `PassiveInventory`, `level_of()` |

### 수정 파일

| 파일 | 변경 |
|---|---|
| `crates/game/src/survivor/stats.rs` | `StatRecalcSystem::run` — PassiveInventory 로부터 PlayerStats 합산 |
| `crates/game/src/survivor/levelup.rs` | `CardKind` 에 `Passive*` 16개 variant 추가, `apply_card` 에 passive 분기 추가 |
| `crates/game/src/survivor/world_setup.rs` | `spawn_player` 에 `PassiveInventory::default()` 컴포넌트 추가 |
| `crates/game/src/survivor/mod.rs` | `pub mod passive` + 재수출 추가 |

### 검증

```bash
cargo build --workspace                     # ok
cargo test --workspace                      # engine 26 / game lib 47 / doc 2 통과
cargo build --release --workspace           # ok
```

---

## Phase 4-A — 적 10종 + AI 변종 (2026-05-21)

### 개요

Zombie 단일 종에서 10종 다양화. AI 변종 6개(Chase/Hover/Kite/Dash/Stay/Split) 도입.
EnemySpawnSystem 이 균등 랜덤으로 10종 중 하나를 선택해 스폰.
EnemyContactDamageSystem 이 각 적의 contact_damage 를 개별 합산.
apply_damage_to_zombie → apply_damage_to_enemy rename (호출처 6곳).
Slime split 무한 방지: split_remaining 필드로 자식 슬라임은 0, 재귀 split 없음.

### 신규 타입

#### EnemyKind (10 variant)

| variant | AI | HP | speed | scale | contact_dmg |
|---|---|---|---|---|---|
| Zombie   | Chase  | 30   | 80  | 40 | 10 |
| Bat      | Chase  | 15   | 140 | 30 | 6  |
| Ghost    | Hover  | 25   | 100 | 36 | 8  |
| Skeleton | Chase  | 35   | 110 | 38 | 9  |
| Mage     | Kite   | 40   | 90  | 38 | 7  |
| Mantis   | Dash   | 45   | 80  | 42 | 12 |
| Plant    | Stay   | 60   | 0   | 44 | 8  |
| Slime    | Split  | 30   | 70  | 36 | 7  |
| Mummy    | Chase  | 120  | 40  | 48 | 15 |
| Knight   | Chase  | 80   | 90  | 44 | 14 |

#### EnemyAiKind (6 variant)

| variant | 동작 |
|---|---|
| Chase | 단순 player 추격 (Zombie/Bat/Skeleton/Mummy/Knight) |
| Hover { stop_at } | stop_at 거리에서 멈춤 (Ghost, stop_at=80) |
| Kite { min_dist } | min_dist 보다 가까우면 달아남, 멀면 추격 (Mage, min_dist=200) |
| Dash { cooldown, dash_speed, dash_lifetime, ... } | cooldown 마다 dash_lifetime 초 동안 dash_speed 로 burst (Mantis) |
| Stay | 정지 (Plant) |
| Split | Chase 이동 + 사망 시 split (Slime) |

#### Enemy 컴포넌트

```rust
pub struct Enemy {
    pub kind:            EnemyKind,
    pub contact_damage:  f32,
    pub split_remaining: u8,  // Slime: 2(초기) → 자식 0 (무한 split 방지)
}
```

### 수정 파일

| 파일 | 변경 내용 |
|---|---|
| `crates/game/src/survivor/enemy.rs` | EnemyKind 10종, EnemyStats, EnemyAiKind 6종, Enemy 컴포넌트, EnemyAi 확장, EnemyAiSystem 분기 처리. Zombie 태그 유지(호환) |
| `crates/game/src/survivor/spawn.rs` | `spawn_enemy(world, pos, kind)`, `spawn_enemy_with_splits`, `pick_random_enemy_kind`. EnemySpawnSystem: 균등 random 10종. despawn 로직을 `Enemy` 쿼리로 일반화. `spawn_zombie` 는 `spawn_enemy(Zombie)` 호출로 단순화 (하위 호환 유지) |
| `crates/game/src/survivor/damage.rs` | `apply_damage_to_zombie` → `apply_damage_to_enemy` rename. Slime 사망 시 split_remaining > 0 이면 자식 2마리 스폰(split_remaining=0) |
| `crates/game/src/survivor/death.rs` | `EnemyContactDamageSystem`: 각 적의 `Enemy.contact_damage` 합산 (이전 고정 10 → 동적) |
| `crates/game/src/survivor/projectile.rs` | `Zombie` 이중 확인 → `Enemy` 컴포넌트 확인으로 변경 |
| `crates/game/src/survivor/weapon.rs` | `apply_damage_to_zombie` → `apply_damage_to_enemy` |
| `crates/game/src/survivor/area.rs` | `apply_damage_to_zombie` → `apply_damage_to_enemy` |
| `crates/game/src/survivor/bible.rs` | `apply_damage_to_zombie` → `apply_damage_to_enemy` |
| `crates/game/src/survivor/lightning.rs` | `apply_damage_to_zombie` → `apply_damage_to_enemy` |
| `crates/game/src/survivor/mod.rs` | `Enemy`, `EnemyAiKind`, `EnemyKind`, `EnemyStats`, `apply_damage_to_enemy`, `spawn_enemy` 재수출 추가 |
| `crates/game/src/lib.rs` | 임포트 갱신 + 신규 테스트 3건. `spawn_timer_triggers_after_interval` 를 `Zombie` → `Enemy` 쿼리로 수정 |

### apply_damage_to_zombie → apply_damage_to_enemy rename 영향

호출처 6곳 일괄 rename:
1. `weapon.rs` — WhipSystem
2. `projectile.rs` — ProjectileSystem
3. `area.rs` — GarlicSystem, HolyWaterPoolSystem
4. `bible.rs` — OrbitingBookSystem
5. `lightning.rs` — LightningRingSystem

### Slime split 무한 방지

`Enemy.split_remaining` 필드:
- 최초 spawn_enemy(Slime): `split_remaining = 2`
- 사망 시 split_remaining > 0 → 자식 2마리 스폰, 각 `split_remaining = 0`
- 자식이 사망해도 split_remaining == 0 → split 발생 안 함

### 테스트

game lib 50 통과 (직전 47 + 신규 3):
- `enemy_kind_stats_for_all_ten` — 10종 EnemyKind::stats() 호출, hp/scale/contact_damage > 0 검증
- `ghost_hovers_at_stop_distance` — Ghost (100, 0) 스폰 + dt=5.0 → pos.x < 100 (플레이어 방향 이동)
- `slime_splits_on_death` — Slime 즉사 → Enemy count == 2 (자식 2마리), split_remaining == 0 검증

기존 테스트 수정:
- `spawn_timer_triggers_after_interval`: `Zombie` → `Enemy` 쿼리 (EnemySpawnSystem 이 랜덤 종류 스폰)

### 검증

```bash
cargo build --workspace                     # ok
cargo test --workspace                      # engine 26 / game lib 50 / doc 2 통과
cargo build --release --workspace           # ok
```

### 핵심 결정

- `EnemyAiKind::Dash` 상태는 variant 필드(elapsed, dashing)에 인라인 저장 → 별도 컴포넌트 없이 EnemyAi.ai 에 함께 보관. EnemyAiSystem 이 collect → 계산 → get_mut 패턴으로 borrow checker 회피.
- 투사체 hit 체크를 `Zombie` 태그 이중 확인에서 `Enemy` 컴포넌트 확인으로 변경 → 10종 모두 피격 가능.
- `spawn_zombie` 는 `spawn_enemy(Zombie)` 래퍼로 단순화. 하위 호환 유지.
- EnemySpawnSystem despawn 로직: `Zombie` → `Enemy` 쿼리로 일반화.

### 다음

**Phase 4-B** — SpawnDirector + waves.ron (시간축 wave 정의)

---

### Phase 4-B — SpawnDirector + waves.ron (2026-05-21)

게임 진행 시간에 따라 wave 가 변하는 시간축 스폰 시스템. 기존 고정 cooldown 1.0초 균등 random 스폰(`EnemySpawnSystem`) 을 `SpawnDirectorSystem` 으로 교체.

#### 신규 파일

| 파일 | 내용 |
|---|---|
| `assets/data/waves.ron` | 6개 wave 정의 (0~30분 커버). 각 wave 에 `start_time`, `end_time`, `spawn_interval`, `enemies` 풀 + weight |
| `crates/game/src/survivor/director.rs` | `EnemyEntry`, `WaveDef`, `WavesFile` (Deserialize 구조체) + `SpawnDirector` (ECS 리소스) + `SpawnDirectorSystem` (System) |

#### 제거된 코드

| 항목 | 파일 |
|---|---|
| `SpawnTimer` 구조체 | `spawn.rs` (약 10줄) |
| `EnemySpawnSystem` (균등 random) | `spawn.rs` (약 44줄) |
| `pick_random_enemy_kind` 헬퍼 | `spawn.rs` (약 15줄) |
| `SpawnTimer::default()` 삽입 | `world_setup.rs` |
| `SpawnTimer` 리셋 | `death.rs` |

#### waves.ron 구조

| Wave | 시간 | interval | 등장 적 |
|---|---|---|---|
| 0 | 0:00~1:00 | 1.2s | Zombie(w6) + Bat(w2) |
| 1 | 1:00~3:00 | 1.0s | Zombie + Bat + Skeleton + Ghost |
| 2 | 3:00~6:00 | 0.8s | Zombie + Bat + Skeleton + Ghost + Mage + Mantis |
| 3 | 6:00~12:00 | 0.6s | Skeleton + Ghost + Mage + Mantis + Slime + Plant |
| 4 | 12:00~20:00 | 0.5s | Mage + Mantis + Slime + Plant + Mummy |
| 5 | 20:00~30:00 | 0.4s | Mantis + Slime + Mummy + Knight |

30분 이후엔 마지막 wave (Knight/Mummy) 반복.

#### 수정 파일

| 파일 | 변경 내용 |
|---|---|
| `crates/game/src/survivor/spawn.rs` | `SpawnTimer`, `EnemySpawnSystem`, `pick_random_enemy_kind` 제거. `spawn_enemy`, `spawn_zombie`, `spawn_enemy_with_splits` 유지 |
| `crates/game/src/survivor/world_setup.rs` | `SpawnTimer::default()` 삽입 → `SpawnDirector::default()` 삽입으로 교체 |
| `crates/game/src/survivor/death.rs` | `SpawnTimer` 리셋 → `SpawnDirector.spawn_elapsed = 0.0` 리셋 |
| `crates/game/src/survivor/mod.rs` | `director` 모듈 추가, `SpawnDirector/SpawnDirectorSystem/…` 재수출. `EnemySpawnSystem/SpawnTimer` 재수출 제거 |
| `crates/game/src/bin/survivor.rs` | `EnemySpawnSystem` → `SpawnDirectorSystem::default()` 교체 |
| `crates/game/src/lib.rs` | 임포트 갱신 + 신규 테스트 3건 + `spawn_timer_triggers_after_interval` 를 director 기반으로 전환 |

#### 테스트

game lib 52 통과 (직전 50 + 신규 3 - 교체 1):
- `waves_ron_loads_at_least_one_wave` — `SpawnDirector::default()` → `waves.len() > 0`
- `director_picks_zombie_in_first_wave` — wave 0 에서 50회 pick → Zombie 10회 이상 (weight 6/8)
- `spawn_director_spawns_enemy_after_interval` — `dt=2.0` → 적 1마리 이상 스폰
- `enemy_despawned_when_far` — 기존 테스트를 `SpawnDirectorSystem` 기반으로 갱신

#### 검증

```bash
cargo build --workspace                     # ok
cargo test --workspace                      # engine 26 / game lib 52 / doc 2 통과
cargo build --release --workspace           # ok
```

#### 다음

**Phase 4-C** — 패턴 스폰 (line / circle / surround / swarm)

---

### Phase 5 — 보스 3종 + 멀티 페이즈 + 화면 흔들기 + StageClear (2026-05-21)

#### 신규 파일

| 파일 | 핵심 타입·함수 |
|---|---|
| `crates/game/src/survivor/boss.rs` | `BossKind`, `Boss`, `BossSpawnQueue`, `CameraShake`, `StageProgress`, `spawn_boss`, `BossSpawnSystem`, `BossPhaseSystem`, `BossDeathSystem` |

#### 보스 3종 스펙

| 보스 | 등장 시간 | HP | 색상 | 스폰 후 단계 |
|---|---|---|---|---|
| GiantSlime | 10:00 (600s) | 1,000 | 청색(0.2,0.6,0.8) | HP<50%→phase1, HP<20%→phase2 |
| GhostKing  | 20:00 (1200s) | 2,500 | 연보라(0.7,0.7,1.0) | 동일 |
| Death      | 30:00 (1800s) | 6,000 | 암적(0.5,0.0,0.0) | 처치 시 StageClear |

#### 기존 파일 수정

| 파일 | 변경 내용 |
|---|---|
| `camera_follow.rs` | `CameraShake` 리소스 읽어 shake offset 을 카메라에 적용. borrow 분리 패턴 |
| `hud.rs` | 보스 HP 상단 표시, StageClear 오버레이 추가 |
| `world_setup.rs` | `BossSpawnQueue / CameraShake / StageProgress` 최초 init 시 삽입 |
| `death.rs` | `restart_world` 에서 보스 3 리소스 모두 default 로 리셋 |
| `mod.rs` | `pub mod boss` + 8종 재수출 |
| `bin/survivor.rs` | `BossSpawnSystem / BossPhaseSystem / BossDeathSystem` 등록 |

#### Borrow Checker 핵심 패턴 (CameraShake)

`CameraShake`(shake.elapsed 갱신 + offset 계산) 와 `Camera`(position 갱신) 를 같은 프레임에 mut 빌릴 수 없으므로:
1. `resource_mut::<CameraShake>()` 블록에서 offset 값을 `Option<Vec2>` 로 추출 후 borrow 종료.
2. 이후 `resource_mut::<Camera>()` 를 따로 빌려 position 에 offset 가산.

#### 신규 테스트 (+3 → 58)

- `boss_spawn_queue_triggers_at_10min` — elapsed=600.1 → GiantSlime 스폰, queue 에서 제거 확인
- `boss_phase_advances_on_low_hp` — HP 40% → phase 1 전환 확인
- `death_boss_defeat_triggers_stage_clear` — Death 보스 HP=0 → despawn + cleared=true

#### 검증

```bash
cargo build --workspace                     # ok
cargo test --workspace                      # engine 26 / game lib 58 / doc 2 통과
cargo build --release --workspace           # ok
```

#### 다음

**Phase 6** — 무기 진화 + 보물상자 (Evolution 레시피 8종)

---

## Phase 6 — 무기 진화 + 보물상자 (2026-05-20 완료)

### 신규 파일

| 파일 | 핵심 타입·함수 |
|---|---|
| `crates/game/src/survivor/chest.rs` | `Chest`, `ChestPickupSystem`, `try_evolve`, `spawn_chest`, `EVOLUTION_RULES` |

### 주요 기능

- 진화 레시피 8종: Whip/MagicWand/Knife/Axe/Cross/FireWand/Garlic/HolyWater — 각 무기 Lv8 + 페어 패시브 Lv5 + 보물상자 픽업 시 진화 발동
- `ChestPickupSystem` — 플레이어 반경 40px 안 Chest 자동 픽업 → `try_evolve()` 호출
- 진화 시 `slot.evolved = true` + cooldown×0.7 + 무기별 스탯 강화 (damage×2 등)
- 엘리트 사망 시 `spawn_chest()` 호출 (황금 28×24 사각형, z=0.4)

### 신규 테스트 (+3 → 61)

- `evolution_rules_count_is_eight` — EVOLUTION_RULES 길이 8 확인
- `try_evolve_with_full_setup_marks_whip_evolved` — Whip Lv8 + HollowHeart Lv5 → evolved=true + damage≥20
- `chest_pickup_despawns_chest` — ChestPickupSystem 실행 후 Chest 엔티티 0개 확인

### 검증

```bash
cargo build --workspace                     # ok
cargo test --workspace                      # engine 26 / game lib 61 / doc 2 통과
cargo build --release --workspace           # ok
```

#### 다음

**Phase 7** — 픽업 5종 (Coin/Chicken/Vacuum/Bomb/Rosary)

---

## Phase 7 — 픽업 5종 (2026-05-21 완료)

### 신규 파일

| 파일 | 핵심 타입·함수 |
|---|---|
| `crates/game/src/survivor/pickup.rs` | `PickupKind`, `Pickup`, `GoldWallet`, `PickupSystem`, `apply_pickup_effect`, `spawn_pickup`, `try_drop_normal_pickup`, `drop_boss_pickups` |

### 픽업 5종 사양

| 종류 | 드롭 조건 | 효과 |
|---|---|---|
| Coin | 적 사망 ~1.0% | `GoldWallet.current += 1` |
| Chicken | 적 사망 ~0.5% | Player HP +20 (max clamp) |
| Rosary | 적 사망 ~0.1% | 화면 내 모든 적 즉사 (9999 데미지) |
| Vacuum | 보스 사망 100% | 모든 XpGem → Player 위치로 즉시 이동 |
| Bomb | 보스 사망 100% | 화면 내 모든 적 100 데미지 |

### 드롭 확률 (일반 적)

`rand::random::<f32>()` roll 기반:
- `roll < 0.001` → Rosary (0.1%)
- `0.001 ≤ roll < 0.006` → Chicken (0.5%)
- `0.006 ≤ roll < 0.016` → Coin (1.0%)
- 그 외 → 픽업 없음 (일반 XpGem 만)

### 수정 파일

| 파일 | 변경 내용 |
|---|---|
| `damage.rs` | 일반 적 사망 분기에 `try_drop_normal_pickup()` 추가 |
| `boss.rs` | `BossDeathSystem` 에 `drop_boss_pickups()` 호출 추가 |
| `world_setup.rs` | `setup_survivor_world` 에 `GoldWallet::default()` 삽입 |
| `hud.rs` | HUD 라인에 Gold 표시 추가 |
| `mod.rs` | `pub mod pickup` + 8종 재수출 |
| `bin/survivor.rs` | `PickupSystem::default()` 등록 (ChestPickupSystem 직후) |

### GoldWallet 정책

사망 시 `restart_world` 에서 리셋하지 않음 — 메타 진행성 유지 (Phase 8 에서 영구 저장 예정).

### 신규 테스트 (+3 → 64)

- `chicken_heals_player` — HP 50 → Chicken → HP 70 확인
- `bomb_kills_all_enemies` — 좀비 3마리 → Bomb → 적 0마리 확인
- `coin_increments_gold_wallet` — Coin 2회 → gold == 2 확인

### 검증

```bash
cargo build --workspace                     # ok
cargo test --workspace                      # engine 26 / game lib 64 / doc 2 통과
cargo build --release --workspace           # ok
```

#### 다음

**Phase 8-A** — SurvivorMode + MetaSave + Title 화면

---

## Phase 8-A — SurvivorMode + MetaSave + Title 화면 (2026-05-21)

### 목표

최상위 게임 모드 시스템(`SurvivorMode`) + 영구 메타 데이터(`MetaSave`) + Title 화면 추가.
PowerUp 매장 18종은 Phase 8-B 에서 구현.

### 신규 파일

| 파일 | 내용 |
|---|---|
| `crates/game/src/survivor/meta.rs` | `MetaSave`, `SurvivorMode`, `ModeTransitionSystem` |

### 기존 파일 수정

| 파일 | 변경 내용 |
|---|---|
| `survivor/mod.rs` | `pub mod meta;` + `MetaSave`, `SurvivorMode`, `ModeTransitionSystem` 재수출 |
| `survivor/world_setup.rs` | `MetaSave::load_or_default()`, `SurvivorMode::default()` 최초 삽입 (if_none 보호) |
| `survivor/hud.rs` | `SurvivorMode` 분기: Title / StageClear / Shop 화면 early return, InGame 은 기존 HUD |
| `bin/survivor.rs` | `ModeTransitionSystem` 최상단 등록 |
| `lib.rs` | 테스트 import 에 `MetaSave`, `SurvivorMode`, `ModeTransitionSystem` 추가 |

### 핵심 설계

- `SurvivorMode` — Title(시작) / Shop(Phase 8-B) / InGame / StageClear 4종.
- `ModeTransitionSystem` — 최상단 등록. Title/Shop/StageClear 에서 `GameState::Paused` 강제 → Playing 가드 시스템들 모두 차단.
- Title + ENTER → `restart_world` + `SurvivorMode::InGame` + `GameState::Playing`.
- InGame + `StageProgress.cleared` → 메타 누적(gold/kills/best_time) → `save_to_disk` → `SurvivorMode::StageClear`.
- StageClear + ENTER → `restart_world` → `SurvivorMode::Title`.
- `MetaSave` / `SurvivorMode` 는 `restart_world` 에서 유지됨 (`if_none` 보호로 재삽입 방지).
- `StageProgress.cleared` 리셋은 `restart_world` 의 `*p = StageProgress::default()` 로 커버.

### 신규 테스트 (+4 → 68)

| 테스트 | 위치 | 검증 내용 |
|---|---|---|
| `meta_save_default_is_zero` | `meta.rs` | `MetaSave::default()` 의 gold/kills/best_time == 0 |
| `meta_save_serialization_roundtrip` | `meta.rs` | RON 직렬화 → 역직렬화 후 동일성 |
| `survivor_mode_default_is_title` | `meta.rs` | `SurvivorMode::default() == Title` |
| `enter_in_game_resets_world_and_sets_mode` | `meta.rs` | `restart_world` 후 `SurvivorMode::InGame` 전환 확인 |

### 검증

```bash
cargo build --workspace                     # ok
cargo test --workspace                      # engine 26 / game lib 68 / doc 2 통과
cargo build --release --workspace           # ok
```

#### 다음

**Phase 8-B** — PowerUp 매장 19종

---

## Phase 8-B — PowerUp 매장 19종 (2026-05-21) — Phase 8 완료

`SurvivorMode::Shop` 화면에서 영구 stat buff 를 gold 로 구매하는 시스템.

### 신규 파일

| 파일 | 내용 |
|---|---|
| `crates/game/src/survivor/powerup.rs` | `PowerUpKind`(19종) · `ShopCursor` · `ShopInputSystem` · `try_purchase` · `apply_powerups` |

### 수정 파일

| 파일 | 변경 내용 |
|---|---|
| `stats.rs` | `StatRecalcSystem` — 패시브 합산 후 `apply_powerups` 호출 (Phase 8-B) |
| `hud.rs` | Shop 화면 UI (19종 목록 + gold 표시 + 커서 강조). Title 화면에 "S = SHOP" 힌트 추가 |
| `meta.rs` | `ModeTransitionSystem` — Title 에서 `KeyS` 입력 시 Shop 진입 |
| `world_setup.rs` | `ShopCursor::default()` 리소스 최초 init 삽입 |
| `mod.rs` | `pub mod powerup` 추가 + `ShopInputSystem` 재수출 |
| `bin/survivor.rs` | `ShopInputSystem` 등록 (ModeTransitionSystem 직후) |

### PowerUp 19종

CLAUDE.md 로드맵 18종(Might/Armor/MaxHealth/Recovery/Cooldown/Area/Speed/Duration/Amount/MoveSpeed/Magnet/Growth/Greed/Luck/Curse/Revival/Reroll/Skip) + **Banish** = 19종.
Reroll/Skip/Banish 는 최대 Lv 3, 나머지는 Lv 5. 비용 = (current_lv + 1) × 100 gold.

### 테스트 (game lib: 68 → 71)

| 테스트 | 내용 |
|---|---|
| `powerup_count_is_nineteen` | `PowerUpKind::ALL.len() == 19` |
| `try_purchase_succeeds_with_enough_gold` | gold 200 → Might Lv1(100g) 구매 성공, gold 100 남음 |
| `apply_powerups_increases_might` | Might Lv3 → `stats.might == 3.0` |

#### 다음

**Phase 9** — 캐릭터 6+ + 해금 시스템
