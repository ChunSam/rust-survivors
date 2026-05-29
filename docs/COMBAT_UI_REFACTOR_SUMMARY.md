# Combat and UI Refactor Summary

Date: 2026-05-29
Commit: `984cba1 Refactor survivor combat and UI flow`

## Summary

이번 작업은 survivor 게임 코드의 안정성과 유지보수성을 높이는 리팩토링이다. 핵심은 보스 사망 처리가 일반 적 사망 경로에 의해 우회되지 않도록 막고, 전투 데미지/타깃 선택 중복을 공통 helper로 모은 것이다.

동시에 `HudSystem`과 `ModeTransitionSystem`의 큰 mode별 분기를 private helper 함수로 분리해 UI와 입력 전환 코드의 읽기 부담을 줄였다. 기존 dirty worktree에 있던 이미지 기반 UI 자산과 관련 문서도 함께 정리되어 커밋에 포함됐다.

## Key Changes

- `apply_damage_to_enemy`가 보스를 즉시 despawn하지 않도록 수정했다.
  - 보스 HP는 데미지로 0 이하가 될 수 있다.
  - 실제 보스 despawn, 드롭, StageClear, kill count 증가는 `BossDeathSystem`이 담당한다.
  - 보스 처치가 일반 적 kill count 경로에 중복 반영되지 않도록 했다.

- `crates/game/src/survivor/combat.rs`를 추가했다.
  - `WeaponFireContext`로 무기 발화 시 필요한 player entity, player position, player stats 읽기를 통합했다.
  - `apply_damage_events`, `apply_damage_to_targets`로 데미지 적용과 kill count 반영을 한 곳에 모았다.
  - `nearest_enemy_in_radius`, `random_enemy_in_radius`, `random_enemies_in_radius`, `direction_or_right`로 타깃 선택 중복을 줄였다.

- 전투 시스템 호출부를 공통 helper 기반으로 정리했다.
  - 대상: `weapon.rs`, `area.rs`, `bible.rs`, `lightning.rs`, `projectile.rs`, `pickup.rs`
  - 기존 동작은 유지했다. 특히 타깃이 없어도 이미 tick된 무기는 쿨다운이 소비되는 의미를 바꾸지 않았다.

- HUD 렌더링 분기를 함수화했다.
  - `HudSystem::run`에서 Title, CharacterSelect, StageSelect, StageClear, Shop, PauseMenu, Settings 렌더링을 mode별 private helper로 분리했다.
  - UI 수치, 문구, 색상, z-order는 유지했다.

- 메타/입력 전환 분기를 함수화했다.
  - `ModeTransitionSystem::run`에서 Title, PauseMenu, Settings 입력 처리를 helper로 분리했다.
  - 저장 구조, key mapping, `GameState`/`SurvivorMode` 전환 순서는 유지했다.

- 테스트 보강을 추가했다.
  - 보스가 무기/광역 데미지로 죽어도 일반 적 사망 경로에서 despawn되지 않는 회귀 테스트를 추가했다.
  - `BossDeathSystem` 실행 후 StageClear, 보스 드롭, kill count가 한 번만 처리되는지 확인한다.
  - `test_support.rs`를 추가해 보스 흐름 테스트 fixture를 일부 공통화했다.

## Assets and Docs Included

이번 커밋에는 기존 작업트리에 있던 이미지 기반 UI 자산도 포함됐다.

- `assets/textures/survivor/menu/*`
- `assets/textures/survivor/ui/*`
- 이미지 UI/엔진 마이그레이션/시각 QA 관련 문서
- 패키징 검증에서 새 menu/ui 이미지가 앱 번들에 포함되는 것을 확인했다.

## Validation

커밋 전 다음 검증을 통과했다.

```bash
cargo fmt --check
cargo test -p game --lib --locked -- --test-threads=1
cargo check -p game --all-targets --locked
cargo build -p game --bin survivor --locked
cargo build -p game --bin survivor --release --locked
bash scripts/package_macos.sh
bash scripts/verify_macos_package.sh
```

결과:

- lib tests: 148 passed
- survivor dev build: passed
- survivor release build: passed
- macOS package verification: passed

## Follow-ups

- `lib.rs`의 대형 테스트 묶음을 도메인별 테스트 모듈로 더 분리한다.
- `survivor/mod.rs`의 public module surface를 facade 중심으로 줄인다.
- HUD와 UI icon layout의 공통 계산을 더 강하게 단일화한다.
- `meta.rs`의 CharacterSelect/StageSelect/StageClear 전환 처리도 같은 방식으로 추가 분리할 수 있다.
