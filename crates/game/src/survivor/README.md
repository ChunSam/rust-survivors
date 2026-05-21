# survivor — Vampire Survivors 클론

`crates/game/src/survivor/` — Vampire Survivors 풀 클론 모듈.
탑다운 자동공격 로그라이트. 상세 로드맵: `CLAUDE.md` "Vampire Survivors 클론 로드맵" 섹션.

**현재 단계**: Phase 2-E 완료 — Garlic(오라) + HolyWater(필드 풀) 추가. 스타터 로드아웃 8 무기 보유.

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
| `area.rs` | `HolyWaterPool { damage, radius, lifetime, tick_cooldown, tick_elapsed }`, `GarlicSystem` (오라 tick 데미지), `HolyWaterSystem` (풀 드롭), `HolyWaterPoolSystem` (풀 lifetime + tick 데미지), `spawn_holy_water_pool` |
| `inventory.rs` | `WeaponKind` enum (Whip/MagicWand/Knife/Axe/Cross/FireWand/Garlic/HolyWater), `WeaponSlot { kind, level, cooldown, elapsed, tick() }`, `WeaponInventory { slots: Vec<WeaponSlot> }` |
| `projectile.rs` | `Projectile { velocity, lifetime, pierce, damage, layer_mask, radius, behavior }`, `ProjectileBehavior` (Straight / Arc{gravity} / Boomerang{return_at, elapsed, returned}), `ProjectileSystem`, `spawn_projectile{_ex}` 헬퍼 |
| `player.rs` | `Player` (태그), `Velocity(Vec2)`, `PlayerStats { move_speed }`, `PlayerMovementSystem` |
| `camera_follow.rs` | `CameraFollowSystem { lerp, viewport }` — 플레이어 위치로 카메라 lerp 추적 |
| `enemy.rs` | `Zombie` (태그), `EnemyAi { move_speed }`, `EnemyAiSystem` — 플레이어 추격 |
| `spawn.rs` | `SpawnTimer { interval, elapsed }`, `EnemySpawnSystem`, `spawn_zombie(world, pos)` |
| `health.rs` | `Health { current, max }`, `take_damage(amount) -> bool` |
| `weapon.rs` | `WhipSystem` (좌/우 AABB), `MagicWandSystem`/`KnifeSystem`/`CrossSystem` (가장 가까운 적 방향), `AxeSystem` (위로 던지는 포물선), `FireWandSystem` (랜덤 적 타깃), `HitFlash`/`HitFlashSystem` |
| `xp.rs` | `XpGem { value }`, `XpAccumulator { current, level, next_threshold }`, `MagnetSystem`, `spawn_xp_gem(world, pos, value)` |
| `levelup.rs` | `CardKind` (WhipDamage/WhipArea/WhipCooldown), `PendingLevelUp { offered, consumed }`, `LevelUpSystem`, `apply_card` 헬퍼 |
| `world_setup.rs` | `spawn_player(world)`, `setup_survivor_world(world)` — 초기 엔티티 스폰 + `SpawnTimer` + `GameStats` 리소스 삽입 |
| `hud.rs` | `GameStats { elapsed, kills }` 리소스, `HudSystem` — 좌상단 HUD + LevelUp 카드 안내 + GameOver 화면 |
| `death.rs` | `EnemyContactDamageSystem` (1초 cooldown 접촉 데미지), `DeathSystem` (HP 0→GameOver), `RestartSystem` (R 키→재시작), `restart_world` 헬퍼 |

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
| `WeaponInventory` | slots[0..8] = Whip / MagicWand / Knife / Axe / Cross / FireWand / Garlic / HolyWater (with_starter_loadout) |

## 시스템 등록 순서 (Phase 2-E — 현재)

```
app.add_system(LevelUpSystem)                        // 첫 번째 — 상태 전환·키 입력 처리
app.add_system(PlayerMovementSystem)
app.add_system(EnemyAiSystem)
app.add_system(EnemySpawnSystem::default())
app.add_system(EnemyContactDamageSystem::default())  // 적 접촉 → 플레이어 데미지
app.add_system(DeathSystem)                          // HP 0 → GameOver
app.add_system(RestartSystem)                        // R 키 → 재시작
app.add_system(CameraFollowSystem::default())
app.add_system(WhipSystem::default())                // 좌/우 AABB
app.add_system(MagicWandSystem::default())           // 가까운 적 직선
app.add_system(KnifeSystem::default())               // 가까운 적 amount개 직선
app.add_system(AxeSystem::default())                 // 위로 던지는 포물선
app.add_system(CrossSystem::default())               // 가까운 적 방향 부메랑
app.add_system(FireWandSystem::default())            // 랜덤 적 큰 데미지
app.add_system(GarlicSystem::default())              // 플레이어 오라 tick 데미지 (2-E)
app.add_system(HolyWaterSystem)                      // 풀 드롭 (2-E)
app.add_system(HolyWaterPoolSystem::default())       // 풀 lifetime + tick 데미지 (2-E)
app.add_system(ProjectileSystem::default())          // 모든 투사체 이동·hit·despawn
app.add_system(MagnetSystem::default())              // XpGem 흡수
app.add_system(HitFlashSystem)
app.add_system(HudSystem)                            // 마지막 — TextQueue push
```

## 다음 단계: Phase 2-F — King Bible + Lightning Ring

