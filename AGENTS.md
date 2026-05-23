# AGENTS.md — Rust 2D Game Engine

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
| `wgpu` | 22 | GPU 렌더링 |
| `glyphon` | 0.6 | GPU 텍스트 렌더링 (wgpu 22 네이티브) |
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
| `has_any_active_contact` 필드 | rapier2d 0.22에서 메서드가 아닌 필드 |
| `cargo clean` 대신 `rm -rf target` | macOS `._CACHEDIR.TAG` 메타파일로 인해 `cargo clean` 실패 |
| wgpu 22 `RenderPipelineDescriptor.cache` 필드 | wgpu 22에서 추가된 파이프라인 캐시 — `None` 으로 비활성화 |
| glyphon 0.6 `Cache` + `Viewport` 신설 | 0.5의 `Resolution` 인자 대신 `Viewport::update(queue, resolution)` 로 GPU 유니폼 갱신 |
| glyphon 0.6 `TextAtlas::new` 에 `&Cache` 필요 | `Cache` 를 먼저 만들고 `atlas`/`viewport` 에 공유. `TextRenderer` 가 `Cache` 소유권을 보존해야 함 |
| 한글 글꼴 동봉 | `NotoSansKR-Regular.ttf` 를 `include_bytes!` 로 바이너리에 임베딩 → OS 시스템 폰트 의존 제거, Linux/Windows 포함 모든 환경에서 동일 결과. 바이너리 +~6MB 비용 수용. 라이선스: SIL OFL 1.1 (`assets/fonts/OFL.txt`) |

## 알려진 이슈 / 확장 포인트

- [x] 텍스처 로딩 연결: `app.load_texture(path)` → `Sprite::textured(path)` 로 사용
- [x] 씬 전환 / 게임 오버: `GameState` 리소스 (`Playing`/`Paused`/`GameOver`), 낙하 감지, R키 재시작
- [x] 오디오 시스템: `AudioManager` 리소스 (`rodio` wav), `audio.play(channel, path, repeat)`
- [x] 스프라이트 애니메이션: `AnimationPlayer` 컴포넌트, `UvRect::from_grid()`, `AnimationSystem`
- [ ] `cargo clean` macOS 버그 → `rm -rf target` 사용

## Git / GitHub

- 저장소: https://github.com/ChunSam/rust-game-engine
- 최신 릴리즈: [v0.1.0](https://github.com/ChunSam/rust-game-engine/releases/tag/v0.1.0)
- 브랜치: `main`

## 사용자 정보

- Rust 초급자 — borrow checker, lifetime 설명 시 이유를 한 줄 덧붙일 것
- 학습 목적 우선 — 성능보다 코드 가독성·원리 이해를 중시
- **Vampire Survivors 클론 한정**으로는 학습용을 넘어 **상용 빌드를 목표**로 진행 (아래 로드맵 참고)

## 진행 상황 (2026-05-22)

- **완료**: Phase 0(엔진 보강) + 0.5(wgpu 22 + NotoSansKR 동봉) + **Phase 1 전체 (Vertical Slice MVP)** + **Phase 2 전체 (무기 풀 확장)** + **Phase 3 전체 (PlayerStats + 패시브 16종)** + **Phase 4 전체 (적 다양화)** + **Phase 5 (보스 3종 + StageClear)** + **Phase 6 (보물상자 + 8 무기 진화 레시피)** + **Phase 7 (픽업 5종)** + **Phase 8 완료 (메인 메뉴 + 메타 진행 + 저장)** + **Phase 9 완료 (캐릭터 6종 + CharacterSelect + 해금)** + **Phase 10 완료 (다중 스테이지 3종 + StageSelect + 클리어 해금)** + **Phase 11-A~F 완료 (폴리쉬 6종)**
- **상태**: engine 26 tests · game lib 82 tests · doc 2 tests · binary 3개(`game`/`text_demo`/`survivor`) 모두 빌드 통과
- **달성 (Phase 11-A~F)**:
  - 11-A: `DamageNumber` 컴포넌트 + `DamageNumberSystem` (수명 0.6s, 위로 40px/s, 알파 페이드)
  - 11-B: 히트 플래시 — 흰색 펄스 + `Transform` 스케일 1.18× 버프 (duration/original_scale 추가)
  - 11-C: 화면 흔들기 확장 — Bomb(0.4s/14px), Rosary(0.35s/10px), 플레이어 피격(0.15s/4px)
  - 11-D: 절차적 SFX — `AudioManager::play_tone` + `SfxQueue` 리소스 + `SfxSystem`. 피격/사망/레벨업/픽업/XP 이벤트 연결
  - 11-E: 스프라이트 파티클 — `Particle` + `ParticleSystem` + `spawn_death_burst` / `spawn_collect_burst`
  - 11-F: 동적 카메라 줌 — 보스 활성 시 zoom 0.65, 평상 시 zoom 1.0 (lerp 0.05)
- **시각 검증 완료 (2026-05-22)**: Phase 11-A~F 전 항목 통과. 수정된 버그 3건:
  - SFX 무음 → `AudioManager` world_setup.rs 미삽입 수정
  - 파티클 미표시 → `Particle.size` 필드 누락으로 스케일 0~1px 렌더링되던 것 수정
  - 보스 줌 미작동 → ZOOM_BOSS 0.80→0.65, ZOOM_LERP 0.015→0.05 로 조정
- **다음**: BGM · SFX 음원 파일 추가 → 업적 시스템 → 로컬라이제이션 → macOS/Windows 빌드 → 출시 준비

상세 — 진행 로그: [`docs/PHASE_LOG.md`](docs/PHASE_LOG.md) · 엔진 API: [`crates/engine/README.md`](crates/engine/README.md) · 서바이버 모듈: [`crates/game/src/survivor/README.md`](crates/game/src/survivor/README.md)

---

## Vampire Survivors 클론 로드맵

이 엔진을 기반으로 **상용 수준의 Vampire Survivors 풀 클론**을 단계적으로 개발한다.
탑다운 자동공격 로그라이트 — 코어 루프(이동 → 자동공격 → XP → 레벨업 → 진화) → 메타 진행(골드 → 영구 강화 → 캐릭터/스테이지 해금) → 30분 1런.

### 확정 결정

| 항목 | 결정 |
|---|---|
| 범위 | 풀 클론 (무기 20+ · 패시브 16 · 진화 · 캐릭터 6+ · 스테이지 3~5 · 메타 진행 · 저장) |
| 전략 | **Vertical Slice 우선** — 4~6주 안에 한 루프 도는 플레이 가능 빌드 → 점진 확장 |
| 충돌 | rapier 의존 제거(서바이버 한정). spatial-hash grid + 원/AABB 거리 판정. 기존 `PhysicsSystem`은 플랫포머 데모용으로 보존 |
| 텍스트 | `glyphon` 0.6 도입 (wgpu 22 네이티브 버전, vendor 패치 불필요) |
| 아트 | Kenney.nl / itch.io CC0 팩 사용 (placeholder 부터 시작 가능) |
| 엔트리 | 기존 `crates/game/src/main.rs` (플랫포머) 보존, 서바이버는 `crates/game/src/bin/survivor.rs` 신규 binary |
| 코드 작성 방식 | 본 문서를 사양서 삼아 `/task` (sonnet 모델) 세션으로 작은 단위씩 구현 의뢰 |

### Phase 0 — 엔진 보강 (~2주)
서바이버 작업 시작 전 채워야 할 엔진 공백.

- `World::despawn(entity)` + `free_ids` 재사용 (`crates/engine/src/ecs/world.rs`)
- 다중 컴포넌트 query 헬퍼: `query2::<A,B>`, `query3::<A,B,C>`
- 카메라 follow: `Camera { position, zoom }` 리소스. `SpriteRenderer::render`(`renderer/sprite.rs:230`)의 하드코딩 ortho 를 카메라 기반 view-proj 로 교체
- 충돌 모듈 신규: `crates/engine/src/collision/{mod,grid,query}.rs` — `Collider(Circle|Aabb)`, `CollisionLayer(bitmask)`, `SpatialGrid(cell=128px)` + `rebuild` / `query_radius` / `query_aabb`
- 마우스 입력: `InputState` 에 `cursor`, `mouse_pressed`, `scroll` 추가. `app.rs` window_event 핸들링
- 텍스트: `glyphon` 통합. `crates/engine/src/renderer/text.rs` 신규. App 렌더에 text pass 추가. `TextQueue` 리소스. `NotoSansKR-Regular.ttf` 를 `include_bytes!` 로 임베딩 — 시스템 폰트 fallback 아닌 동봉 글꼴 사용으로 OS 간 한글 렌더 일관성 확보
- z-order: `Transform.z` 추가, sprite 그룹 내 z 정렬
- 저장: `serde` + `ron` + `dirs`. `crates/engine/src/save.rs` 신규
- `rand = "0.8"` 추가
- **기존 플랫포머 데모는 회귀 없이 그대로 동작해야 함**

### Phase 1 — Vertical Slice MVP (~4주)
완결된 한 루프가 도는 빌드.

- `crates/game/src/bin/survivor.rs` + `crates/game/src/survivor/` 모듈 트리
- Player: `Player + Transform + Velocity + Health + XpAccumulator + PlayerStats + Sprite + Collider`. WASD/방향키 이동
- `CameraFollowSystem`: 플레이어 lerp 따라감
- Zombie 단일 적: 플레이어 추격. 카메라 밖 반경에서 주기적 스폰. 카메라에서 너무 멀면 despawn
- 무기 1개: Whip (좌/우 부채꼴 hitbox, cooldown 1초). `DamageSystem` 이 spatial grid 로 hit 판정
- 죽음/드롭: 적 사망 시 `XpGem` 스폰. 플레이어 사망 시 `GameState::GameOver`
- 자석: 플레이어 반경 내 젬 가속 흡수. 픽업 시 XP 누적
- 레벨업: 임계치 도달 시 `GameState::LevelUp` + 3장 카드(현재는 Whip 강화만). 마우스 또는 1·2·3 키 선택
- HUD (glyphon): 타이머·레벨·HP·XP·처치 수
- GameOver: "YOU DIED" + R 재시작 (`App::reload_scene`)

### Phase 2 — 무기 풀 확장 (~3주)
- `WeaponKind` enum + `WeaponInventory { slots: [Option<WeaponSlot>; 6] }`
- 공용 `ProjectileSystem` (lifetime / pierce / damage)
- 추가 무기 9개: Magic Wand, Knife, Axe, Cross, Garlic, Holy Water, King Bible, Fire Wand, Lightning Ring
- 레벨 1~8 스탯 테이블: `assets/data/weapons.ron`

### Phase 3 — 패시브 + 스탯 시스템 (~2주)
- `PlayerStats` (might, area, projectile_speed, duration, amount, cooldown, max_health, recovery, armor, move_speed, magnet, luck, growth, greed, curse, revival)
- `StatRecalcSystem` — 보유 패시브 + 메타 파워업 + 기본값 합산
- 패시브 16종 (Spinach / Armor / Hollow Heart / Pummarola / Empty Tome / Candelabrador / Bracer / Spellbinder / Duplicator / Wings / Attractorb / Clover / Crown / Stone Mask / Skull O'Maniac / Tiragisu)

### Phase 4 — 적 다양화 + 스폰 감독관 (~3주)
- Enemy 10+종 (zombie / bat / ghost / skeleton / mage / mantis / plant / slime / mummy / knight). AI 변종: chase / dash / kite / projectile / split / summon
- `SpawnDirector` 리소스 — 시간축 wave 정의 (RON)
- 패턴: line / circle / surround / swarm
- 엘리트 마커 (보물상자 드롭)

### Phase 5 — 보스 + 30분 길이 (~2주)
- Boss 컴포넌트 (큰 HP, 멀티-페이즈, 화면 흔들기)
- 등장: 10분 / 20분 / 30분 (Death)
- 30분 도달 → StageClear

### Phase 6 — 무기 진화 + 보물상자 (~2주)
- Evolution 레시피 8종 (예: Whip+Hollow Heart→Bloody Tear, Magic Wand+Empty Tome→Holy Wand, Axe+Candelabrador→Death Spiral …)
- 조건: 무기 Lv8 + 페어 패시브 보유 + 엘리트가 떨군 보물상자 획득
- 보물상자 1/3/5 카드 연출

### Phase 7 — 픽업 (~1주)
- Coin (메타 골드), Chicken (체력 회복), Vacuum (전체 젬 흡수), Bomb (광역 데미지), Rosary (전체 적 즉사)

### Phase 8 — 메인 메뉴 + 메타 진행 + 저장 (~3주)
- 메뉴: Title / Character / PowerUp / Stage / Settings / Quit
- 영구 `GoldWallet`, `Unlocks { characters, stages, achievements }`, `PowerUps`
- PowerUp 매장 18종 (Might / Armor / Max Health / Recovery / Cooldown / Area / Speed / Duration / Amount / Move Speed / Magnet / Growth / Greed / Luck / Curse / Revival / Reroll / Skip / Banish)
- 저장: `engine::save` (RON). 위치 `dirs::data_dir() / "rust-vampire-survivors" / "save.ron"`

### Phase 9 — 캐릭터 + 해금 (~2주)
- 6~8 캐릭터, 고유 시작 무기 + 스탯 보정
- 해금 조건 (특정 무기 진화, N분 생존, M마리 처치 등)

### Phase 10 — 다중 스테이지 (~3주)
- 3~5 스테이지: Mad Forest / Inlaid Library / Dairy Plant / Gallo Tower / Cappella Magna
- 각 스테이지 RON 분리 (적 풀, 보스, wave, 타일)

### Phase 11 — 폴리쉬 & 출시 (~ongoing)
- 데미지 숫자, 히트 플래시, 화면 흔들기, 카메라 줌, 파티클
- SFX 20+, BGM 3~5곡
- 업적, 로컬라이제이션 (한/영)
- macOS .app / Windows .exe 빌드. Steam 출시 검토 (`steamworks-rs`)

### 일정 요약

| Phase | 기간 | 산출물 |
|---|---|---|
| 0 | 2주 | 엔진 보강, 플랫포머 회귀 없음 |
| 1 | 4주 | **Vertical Slice 플레이 가능** |
| 2~3 | 5주 | 무기 10 + 패시브 16 + 스탯 |
| 4~5 | 5주 | 적 10+, 스폰 감독관, 보스 3 |
| 6~7 | 3주 | 진화 + 픽업 |
| 8 | 3주 | 메인 메뉴 + 메타 + 저장 |
| 9~10 | 5주 | 캐릭터 + 스테이지 |
| 11 | ongoing | 폴리쉬 → 출시 |

---

## 서바이버 — 변경 예정 파일

구현 시점에 신규/수정될 파일 목록. 모든 작업은 `/task` (sonnet) 세션에서 위 로드맵을 참조하여 점진적으로 진행.

### 엔진 (`crates/engine/src/`)

| 파일 | 종류 | 내용 |
|---|---|---|
| `ecs/world.rs` | 수정 | `despawn`, `query2/3`, `free_ids` |
| `camera.rs` | **신규** | `Camera { position, zoom }` 리소스 |
| `renderer/sprite.rs` | 수정 | 라인 230 부근 ortho 를 카메라 view-proj 로 교체 + z 정렬 |
| `renderer/text.rs` | **신규** | `glyphon` 통합, `TextQueue` 리소스 |
| `collision/{mod,grid,query}.rs` | **신규** | `Collider`, `CollisionLayer`, `SpatialGrid` |
| `input.rs`, `app.rs` | 수정 | 마우스 위치·버튼·스크롤 추가 |
| `components.rs` | 수정 | `Transform.z` 필드, `GameState` 변종 추가(`LevelUp`, `Title`, `CharacterSelect` 등) |
| `save.rs` | **신규** | serde + ron 저장/로드 |
| `Cargo.toml` | 수정 | `rand`, `glyphon`, `serde`, `ron`, `dirs` 추가 |

### 게임 (`crates/game/`)

| 파일 | 종류 |
|---|---|
| `src/main.rs` | 보존 (플랫포머 데모) |
| `src/bin/survivor.rs` | **신규** 진입점 |
| `src/survivor/{player,enemy,weapon,projectile,xp,levelup,hud,spawn,stats,death,pickup,boss,chest,evolution}.rs` | **신규** |
| `src/survivor/data/{weapons,passives,waves,characters,stages,evolutions}.ron` | **신규** |
| `Cargo.toml` | 수정 — `[[bin]] name = "survivor"` 추가 |

### 자산 (`assets/`)

| 폴더 | 추가 |
|---|---|
| `textures/` | Kenney/itch.io CC0 팩 (player, enemy 10+, projectile, pickup, tileset) |
| `audio/` | SFX 20+, BGM 3~5 |
| `fonts/` | NotoSansKR-Regular.ttf 동봉 완료 (SIL OFL 1.1). Pretendard 등 추가 weight 는 필요 시 |

### 재사용할 기존 자산

- `engine::audio::AudioManager` (`crates/engine/src/audio.rs:12`) — 그대로
- `engine::animation::AnimationSystem` + `AnimationPlayer::current_uv` (`animation.rs`, `components.rs:111`) — 그대로
- `engine::ecs::System` trait (`ecs/system.rs`) — 그대로
- `engine::components::{Transform, Sprite, UvRect}` — `Transform.z` 만 추가
- `engine::renderer::SpriteRenderer` — 카메라/z-order 패치만, 인스턴싱 파이프라인 재사용
- `engine::components::GameState` — 변종만 확장

