# survivor — Vampire Survivors 클론

`crates/game/src/survivor/` — Vampire Survivors 풀 클론 모듈.
탑다운 자동공격 로그라이트. 상세 로드맵: `CLAUDE.md` "Vampire Survivors 클론 로드맵" 섹션.

**현재 단계**: Phase 1-D (XpGem 드롭 + 자석 흡수)

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
| `mod.rs` | 하위 모듈 선언 + 최상위 재수출 + `LAYER_PLAYER`/`LAYER_ENEMY`/`LAYER_XP` 상수 |
| `player.rs` | `Player` (태그), `Velocity(Vec2)`, `PlayerStats { move_speed }`, `PlayerMovementSystem` |
| `camera_follow.rs` | `CameraFollowSystem { lerp, viewport }` — 플레이어 위치로 카메라 lerp 추적 |
| `enemy.rs` | `Zombie` (태그), `EnemyAi { move_speed }`, `EnemyAiSystem` — 플레이어 추격 |
| `spawn.rs` | `SpawnTimer { interval, elapsed }`, `EnemySpawnSystem`, `spawn_zombie(world, pos)` |
| `health.rs` | `Health { current, max }`, `take_damage(amount) -> bool` |
| `weapon.rs` | `Whip` 컴포넌트, `WhipSystem` — 1초 cooldown, 좌/우 AABB, LAYER_ENEMY 쿼리, 사망 시 XpGem 드롭 + despawn |
| `xp.rs` | `XpGem { value }`, `XpAccumulator { current }`, `MagnetSystem`, `spawn_xp_gem(world, pos, value)` |
| `world_setup.rs` | `spawn_player(world)` — 초기 엔티티 스폰 + `SpawnTimer` 리소스 삽입 + `XpAccumulator` 부착 |

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
| `Sprite` | 노란 사각형 |
| `Player` | 태그 |
| `PlayerStats` | move_speed=200.0 |
| `Velocity` | (0, 0) |
| `Health` | current=max=100.0 |
| `Whip` | 기본값 (damage=10, cooldown=1.0) |

## 시스템 등록 순서 (Phase 1-D)

```
app.add_system(PlayerMovementSystem)
app.add_system(EnemyAiSystem)
app.add_system(EnemySpawnSystem::default())
app.add_system(CameraFollowSystem::default())
app.add_system(WhipSystem::default())       // 적 사망 시 XpGem 스폰
app.add_system(MagnetSystem::default())     // XpGem 흡수
app.add_system(HitFlashSystem)
```

입력 → 이동 → 추격 → 스폰/정리 → 카메라 → 무기(gem 드롭) → 자석(픽업) → 히트플래시 순서.

## 다음 단계: Phase 1-E — 레벨업 + 카드 선택
