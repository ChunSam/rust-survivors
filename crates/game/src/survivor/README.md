# survivor — Vampire Survivors 클론

`crates/game/src/survivor/` — Vampire Survivors 풀 클론 모듈.
탑다운 자동공격 로그라이트. 상세 로드맵: `CLAUDE.md` "Vampire Survivors 클론 로드맵" 섹션.

**현재 단계**: Phase 1-A (서바이버 진입점 · Player · Camera follow)

## 실행

```bash
cargo run -p game --bin survivor           # debug
cargo run -p game --bin survivor --release # 최적화
```

## 조작 (Phase 1-A)

| 키 | 동작 |
|---|---|
| `W` / `↑` | 위로 이동 |
| `S` / `↓` | 아래로 이동 |
| `A` / `←` | 왼쪽 이동 |
| `D` / `→` | 오른쪽 이동 |
| `ESC` | 종료 |

Phase 1-B 이후 자동공격·레벨업 등 추가 예정.

## 모듈 트리

| 파일 | 내용 |
|---|---|
| `mod.rs` | 하위 모듈 선언 + 최상위 재수출 |
| `player.rs` | `Player` (태그), `Velocity(Vec2)`, `PlayerStats { move_speed }`, `PlayerMovementSystem` |
| `camera_follow.rs` | `CameraFollowSystem { lerp, viewport }` — 플레이어 위치로 카메라 lerp 추적 |
| `world_setup.rs` | `spawn_player(world)` — 초기 엔티티 스폰 |

### player.rs 주요 타입

| 타입 | 설명 |
|---|---|
| `Player` | 태그 컴포넌트. 플레이어 엔티티 식별용. |
| `Velocity(Vec2)` | 현재 이동 벡터 (픽셀/초). `PlayerMovementSystem` 이 매 프레임 갱신. |
| `PlayerStats { move_speed: f32 }` | 기본 200 px/s. Phase 3 에서 스탯 풀로 확장. |
| `PlayerMovementSystem` | WASD/방향키 → 방향 정규화 → `Transform.position += dir * speed * dt` |

### camera_follow.rs

```
CameraFollowSystem { lerp: 0.15, viewport: Vec2(800, 600) }
  매 프레임:
    desired_top_left = player_pos - viewport * 0.5
    camera.position  = camera.position.lerp(desired_top_left, lerp)
```

`lerp = 0.0` → 카메라 고정, `1.0` → 즉시 추적. 기본 0.15 (부드러운 follow).

### world_setup.rs

`spawn_player` 가 생성하는 컴포넌트:

| 컴포넌트 | 값 |
|---|---|
| `Transform` | position=(0,0), scale=48×48 px, z=1.0 |
| `Sprite` | 노란 사각형 (Phase 1-B 에서 텍스처 교체) |
| `Player` | 태그 |
| `PlayerStats` | move_speed=200.0 |
| `Velocity` | (0, 0) |

## 시스템 등록 순서 (Phase 1-A)

```
app.add_system(PlayerMovementSystem)
app.add_system(CameraFollowSystem::default())
```

입력 처리 → 위치 갱신 → 카메라 follow 순서를 반드시 유지한다.

## 다음 단계: Phase 1-B — Zombie 적 + 스폰

| 예정 파일 | 내용 |
|---|---|
| `enemy.rs` | `Zombie`, `EnemyAi`, `EnemyAiSystem` (플레이어 방향 추격) |
| `spawn.rs` | `EnemySpawnSystem` — 카메라 외곽 원형 경계에서 주기 스폰, 거리 초과 시 `World::despawn` |

`CollisionLayer` 부착도 Phase 1-B 에서 추가 — Phase 1-C `DamageSystem` 의 `SpatialGrid::query_radius` 준비.
