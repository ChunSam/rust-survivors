# Complete engine v2 pinning and gameplay action debt pass

**Date:** 2026-06-04
**Status:** COMPLETED
**Bead(s):** none
**Epic:** Rust Survivors release hardening
**Chain:** `engine-audio-update` seq `2`
**Parent:** `HANDOFF_engine-audio-update_sfx-bus-hidpi_2026-06-03.md`
**Prior chain:** `HANDOFF_engine-audio-update_sfx-bus-hidpi_2026-06-03.md` > this

---

## Related Handoffs

- `plans/handoffs/HANDOFF_bgm-path-hardening_executable-audio-paths_2026-05-31.md` — separate BGM executable-relative asset lookup chain.
- `plans/handoffs/HANDOFF_bgm-path-hardening_repeat-flag-fix_2026-05-31.md` — separate BGM chain follow-up for StageClear/GameOver one-shot repeat behavior.
- `plans/handoffs/HANDOFF_bgm-path-hardening_variant-rollover-regression_2026-06-01.md` — separate BGM chain follow-up for natural rollover between repeating BGM variants.
- `plans/handoffs/HANDOFF_survivor-ui-hud-layout_visual-qa-fixes_2026-06-02.md` — separate UI/visual QA chain; still relevant for the remaining 800x600/HiDPI manual checks.

## Since Last Handoff

- Parent seq 1 ended after SFX bus integration and title HiDPI input fixes, with engine still described as a local path dependency in the handoff.
- The next user request shifted from audio-only follow-up into a six-item integrated plan: engine v2 pinning, release/CI dependency posture, manual QA targets, visual QA targets, gameplay stat debt, and localization/release hardening.
- The release dependency question was answered: use a pinned git rev, not the sibling local path dependency.
- The gameplay feature policy was answered: implement all inert/placeholder stats and Reroll/Skip/Banish behavior rather than removing or hiding those upgrades.
- Parent risks around reproducibility materialized and were resolved: `crates/game/Cargo.toml` now pins `skeleton-engine` to rev `61c09f14434910a94afc4c9ea167b3f47e0b203b`, and `Cargo.lock` records `skeleton-engine v2.0.0` from that git source.
- Parent manual audio QA risk remains open: structural tests/build/package passed, but actual speaker listening was not performed.
- Test count moved from parent's `189` to `200` lib tests after adding gameplay debt tests.

## Reference Documents

- `AGENTS.md` — active repo instructions and current source of truth for commands, engine boundary, controls, validation count, and next priorities.
- `CLAUDE.md` — older compact agent notes; currently stale on paths, dependency form, and test count. Prefer `AGENTS.md` when conflicting.
- `docs/ENGINE_DEPENDENCY_WORKFLOW.md` — updated this session to document pinned git dependency workflow.
- `docs/manual_qa_checklist.md` — updated this session with 2026-06-03/04 engine v2 and gameplay action manual QA targets.
- `docs/ENGINE_CHANGE_REQUESTS.md` — historical engine requests; no new engine request was needed in this session.

## The Goal

The user asked to implement the previously proposed "1-6 integrated execution plan." The concrete objective was to finish the engine v2 update safely, make the game releasable without a local sibling engine checkout, and implement the lingering gameplay systems tied to already-sold meta upgrades. The work also needed to keep the prior SFX bus change, preserve the existing public save/menu surfaces, add tests, update relevant docs, run release/package validation, then create this handoff and commit/push.

## Where We Are

- Branch before handoff/commit: `main`.
- Worktree before this handoff file had 12 modified tracked files:
  - `AGENTS.md`
  - `Cargo.lock`
  - `crates/game/Cargo.toml`
  - `crates/game/src/survivor/death.rs`
  - `crates/game/src/survivor/hud.rs`
  - `crates/game/src/survivor/levelup.rs`
  - `crates/game/src/survivor/locale.rs`
  - `crates/game/src/survivor/pickup.rs`
  - `crates/game/src/survivor/spawn.rs`
  - `crates/game/src/survivor/weapon.rs`
  - `docs/ENGINE_DEPENDENCY_WORKFLOW.md`
  - `docs/manual_qa_checklist.md`
- This handoff file is also intended to be staged and committed.
- `crates/game/Cargo.toml` changed the engine dependency from `path = "../../../skeleton-engine"` to pinned git rev:
  - `git = "https://github.com/ChunSam/skeleton-engine"`
  - `rev = "61c09f14434910a94afc4c9ea167b3f47e0b203b"`
- `Cargo.lock` now records:
  - `name = "skeleton-engine"`
  - `version = "2.0.0"`
  - `source = "git+https://github.com/ChunSam/skeleton-engine?rev=61c09f14434910a94afc4c9ea167b3f47e0b203b#61c09f14434910a94afc4c9ea167b3f47e0b203b"`
- `crates/game/src/survivor/weapon.rs` preserves the engine v2 ECS compatibility fix: sort hit entities using `Entity::index()` rather than direct tuple field access.
- `LevelUpActions` was added in `crates/game/src/survivor/levelup.rs` as a run-local resource.
- `LevelUpActions::from_meta(meta)` reads existing `MetaSave.powerup_levels` entries for `Reroll`, `Skip`, and `Banish`.
- `CardKind` now derives `Eq` and `Hash` in addition to previous traits, allowing stable comparisons for banished-card tracking.
- `LevelUpSystem::ensure_actions` lazily initializes level-up action state at the first pending level-up.
- `LevelUpSystem::available_cards_for` now filters out both maxed/unavailable cards and cards in `LevelUpActions.banished`.
- `LevelUpSystem::random_offer` centralizes 3-card random offer generation.
- `LevelUpSystem::replace_offer` regenerates pending level-up offers after reroll/banish.
- `LevelUpSystem::reroll_offer` consumes one reroll and keeps the level-up pending state.
- `LevelUpSystem::skip_levelup` consumes one skip, increments XP level/threshold, removes pending level-up, and resumes `GameState::Playing`.
- `LevelUpSystem::banish_offer` binds `4/5/6` to banish offered card slots 1/2/3, consumes one banish, appends the card to `banished`, and regenerates the offer.
- `LevelUpInputAction` was added internally with variants `Choose(usize)`, `Reroll`, `Skip`, and `Banish(usize)`.
- Level-up input now handles:
  - `1/2/3` choose card
  - `R` reroll
  - `S` skip
  - `4/5/6` banish corresponding card
- `hud.rs` reads `LevelUpActions` during a pending level-up and displays localized counts below the three cards.
- `locale.rs` adds `UiText::LevelUpActions` with Korean and English strings:
  - KO: `R 재추첨 {reroll}  S 건너뛰기 {skip}  4/5/6 추방 {banish}`
  - EN: `R Reroll {reroll}  S Skip {skip}  4/5/6 Banish {banish}`
- `pickup.rs` now uses `PlayerStats.luck` for normal pickup drop thresholds.
- `normal_pickup_thresholds(luck)` clamps luck to `0.0..=5.0` and multiplies cumulative base thresholds:
  - Rosary base `0.001`
  - Chicken base `0.006`
  - Coin base `0.016`
- `pickup.rs` now uses `PlayerStats.greed` for coin value.
- `coin_gold_value(greed)` returns `greed.max(0.0).round().max(1.0) as u32`.
- `spawn.rs` now applies `PlayerStats.curse` to enemy HP, move speed, and contact damage at spawn time.
- `enemy_curse_multiplier(world)` reads active `PlayerStats.curse`, clamps it to `0.5..=3.0`, and defaults to `1.0` when no player stats exist.
- Curse does not scale enemy collision radius or visual size. Existing collider tests still guard that visual scale and collider scale remain decoupled.
- `death.rs` now has `RevivalState { used: u32 }` as a run-local resource.
- `DeathSystem` now checks `PlayerStats.revival` before setting `GameState::GameOver`.
- If revival remains, `DeathSystem` increments `RevivalState.used`, restores player HP to `max_hp * 0.5` with minimum `1.0`, pushes `SfxEvent::LevelUp`, logs `Revived`, and keeps `GameState::Playing`.
- If no revival remains, `DeathSystem` keeps the previous behavior and sets `GameState::GameOver`.
- `reset_run_state` now clears both `LevelUpActions` and `RevivalState` in addition to `PendingLevelUp`.
- `AGENTS.md` was updated to say the engine is a pinned git dependency, record the v2.0.0 rev, add level-up action controls, update current state date, and update the verified test count to `200`.
- `docs/ENGINE_DEPENDENCY_WORKFLOW.md` now documents pinned git dependency workflow instead of local path workflow.
- `docs/manual_qa_checklist.md` has a new 2026-06-03/04 section with completed automated validation and remaining manual QA targets.
- No save schema was changed.
- No `SfxEvent` or `SfxQueue` public surface was changed.
- No new art/audio assets were added.
- No external `skeleton-engine` checkout files were edited.
- Final automated validation passed:
  - `cargo fmt --check`
  - `cargo check -p game --bin survivor --locked`
  - `cargo test -p game --lib --locked -- --test-threads=1`
  - `cargo build -p game --bin survivor --release --locked`
  - `bash scripts/package_macos.sh`
  - `bash scripts/verify_macos_package.sh`
- Final lib test count: `200 passed; 0 failed`.
- `git diff --check` passed.
- Manual speaker QA and actual 800x600/HiDPI window visual QA were not performed; they are documented as remaining manual checks.

## What We Tried (Chronological)

1. **Started from user-provided plan in implementation mode.**
   - User supplied the full "1-6 integrated execution plan" and said `PLEASE IMPLEMENT THIS PLAN`.
   - The plan explicitly chose pinned engine git rev, gameplay debt implementation, localization labels, docs, test/build/package validation, and no new assets.
   - Default mode was active, so work proceeded directly instead of creating another plan.

2. **Baseline repo and dirty state check.**
   - Ran `git status --short`.
   - Initial dirty files were already expected from previous engine v2 check:
     - `M Cargo.lock`
     - `M crates/game/src/survivor/weapon.rs`
   - Those were preserved rather than reverted.

3. **Targeted code search.**
   - Searched for `PendingLevelUp`, `LevelUpSystem`, `choose_card`, `PowerUpKind::Reroll/Skip/Banish`, `PlayerStats`, `DeathSystem`, `try_drop_normal_pickup`, `GoldWallet`, and `spawn_enemy`.
   - Confirmed the relevant files:
     - `crates/game/src/survivor/levelup.rs`
     - `pickup.rs`
     - `spawn.rs`
     - `death.rs`
     - `hud.rs`
     - `locale.rs`
     - `powerup.rs`
     - `stats.rs`
   - Confirmed `powerup.rs` already calculated `Luck`, `Greed`, `Curse`, `Revival` stats but Reroll/Skip/Banish were no-op placeholders.

4. **Read implementation surfaces.**
   - Read `levelup.rs` around `CardKind`, `PendingLevelUp`, `LevelUpSystem::apply_card`, `choose_card`, and system `run`.
   - Read `pickup.rs` around `GoldWallet`, `apply_pickup_effect`, `try_drop_normal_pickup`, and existing pickup tests.
   - Read `spawn.rs` around `spawn_enemy_full` and collider tests.
   - Read `death.rs` around `DeathSystem`, `reset_run_state`, and restart/title reset tests.
   - Read `locale.rs` and `hud.rs` around `UiText` and pending level-up text drawing.

5. **Patched engine dependency and gameplay logic.**
   - Changed `crates/game/Cargo.toml` engine dependency to pinned git rev.
   - Added run-local `LevelUpActions` and level-up action helpers.
   - Added luck/greed helper functions and runtime usage.
   - Added curse multiplier and runtime enemy-stat usage.
   - Added revival state and DeathSystem revive behavior.
   - Added HUD count display and localization key.

6. **First lockfile attempt failed due package ID mismatch.**
   - Ran `cargo update -p skeleton-engine`.
   - It failed with `error: package ID specification 'skeleton-engine' did not match any packages`.
   - Reason: existing lockfile still represented the old path dependency source state.
   - Moved to full lockfile regeneration.

7. **First lockfile regeneration hit network sandbox.**
   - Ran `cargo generate-lockfile`.
   - It failed with DNS/network errors resolving GitHub under restricted network.
   - Re-ran with escalation because fetching the pinned git dependency was required.
   - Escalated `cargo generate-lockfile` succeeded.

8. **Formatted and type checked.**
   - Ran `cargo fmt`.
   - Ran `cargo check -p game --bin survivor --locked`.
   - First check failed because static crates downloads were blocked by sandbox DNS.
   - Re-ran `cargo check` with network escalation.
   - Check passed, compiling `skeleton-engine v2.0.0` from the pinned git rev.

9. **Initial full test failure exposed one test precision bug.**
   - Ran `cargo test -p game --lib --locked -- --test-threads=1`.
   - Compilation succeeded and 199 of 200 tests passed.
   - Failure was `survivor::pickup::tests::normal_pickup_thresholds_scale_and_clamp_luck`.
   - Actual value was `0.080000006`, expected exact `0.08`.
   - Fixed the assertion to use epsilon-based float comparison.

10. **Full automated tests passed after float fix.**
    - Ran `cargo fmt --check` and full lib tests in parallel.
    - `cargo fmt --check` passed.
    - `cargo test -p game --lib --locked -- --test-threads=1` passed with `200` tests.

11. **Docs updated to match new reality.**
    - Read `docs/manual_qa_checklist.md`, `AGENTS.md`, and `docs/ENGINE_DEPENDENCY_WORKFLOW.md`.
    - Updated `AGENTS.md` engine dependency wording, controls, current state, engine rev, and test count.
    - Rewrote `docs/ENGINE_DEPENDENCY_WORKFLOW.md` from local path workflow to pinned git workflow.
    - Added automated validation and remaining manual QA section to `docs/manual_qa_checklist.md`.

12. **Release and package validation.**
    - Ran `cargo build -p game --bin survivor --release --locked`; it passed.
    - Ran `bash scripts/package_macos.sh`; it passed and printed manifest verification for folder and `.app`.
    - Ran `bash scripts/verify_macos_package.sh`; it passed.
    - Updated manual QA doc from "build needed" to "build/package verified".

13. **Final diff/quality checks.**
    - Ran `git status --short`: saw the expected modified files.
    - Ran `git diff --stat`: `12 files changed, 796 insertions(+), 200 deletions(-)` before this handoff file.
    - Ran targeted `git diff` inspections for source and docs.
    - Ran `git diff --check`; it passed with no whitespace errors.

14. **Handoff request.**
    - User explicitly invoked `$handoff` and requested commit/push: `[$handoff] ... 하고 커밋 푸쉬`.
    - This file is the structured handoff for future continuation.
    - After writing and validating it, stage all intended files, commit, and push `main`.

## Key Decisions

- **Use pinned git dependency for engine v2.**
  - Reason: user chose recommended "Pinned git rev" posture in the plan phase, and release/CI should not require a sibling checkout.
  - Rejected alternative: keep `path = "../../../skeleton-engine"` for local co-development convenience.

- **Do not edit external engine checkout.**
  - Reason: AGENTS.md explicitly forbids direct external `skeleton-engine` edits from this repo.
  - No new engine API request was needed because current public APIs were enough.

- **Keep save schema stable.**
  - Reason: plan explicitly said no save data/menu/user-control schema changes.
  - Existing `MetaSave.powerup_levels` keys are now read for Reroll/Skip/Banish counts.

- **Implement all inert gameplay stats instead of hiding them.**
  - Reason: user selected recommended "전부 구현" policy.
  - Rejected alternative: remove or stop selling/rewarding inert upgrades.

- **Use run-local resources for consumed state.**
  - `LevelUpActions` tracks remaining rerolls/skips/banishes and banished cards.
  - `RevivalState` tracks consumed revivals.
  - Reason: per-run state should reset on restart/title reset and should not alter persistent save schema.

- **Bind banish to `4/5/6`, not a new cursor/highlight system.**
  - Reason: existing level-up UX uses direct numeric card keys; `4/5/6` maps clearly to card slots without adding highlight/cursor state.
  - Rejected alternative: add a selection cursor and bind `B` to highlighted card.

- **Make skip consume the pending level-up without a card.**
  - Reason: it mirrors the expected "skip reward" behavior while keeping XP level/threshold progression consistent with card selection.

- **Clamp luck and curse.**
  - Luck is clamped to `0.0..=5.0`.
  - Curse is clamped to `0.5..=3.0`.
  - Reason: prevent runaway values from making drop thresholds or enemy stats extreme.

- **Do not scale enemy collider or visuals with curse.**
  - Reason: plan said avoid playfield coverage changes, and existing visual/collider tests protect this separation.

- **Use 50% max HP for revival.**
  - Reason: concrete, deterministic, and easy to test; strong enough to matter without needing additional balance state.

- **Document manual QA rather than claiming it.**
  - Reason: actual speaker output and live 800x600/HiDPI window checks were not performed.
  - Automated tests/build/package validation were recorded as completed.

## Evidence & Data

### 1. Git state before this handoff file

| Command | Result |
|---|---|
| `git branch --show-current` | `main` |
| `git status -s` | 12 modified files before this handoff |
| `git diff --stat` | `12 files changed, 796 insertions(+), 200 deletions(-)` |

### 2. Modified files before this handoff

| File | Status |
|---|---|
| `AGENTS.md` | modified |
| `Cargo.lock` | modified |
| `crates/game/Cargo.toml` | modified |
| `crates/game/src/survivor/death.rs` | modified |
| `crates/game/src/survivor/hud.rs` | modified |
| `crates/game/src/survivor/levelup.rs` | modified |
| `crates/game/src/survivor/locale.rs` | modified |
| `crates/game/src/survivor/pickup.rs` | modified |
| `crates/game/src/survivor/spawn.rs` | modified |
| `crates/game/src/survivor/weapon.rs` | modified |
| `docs/ENGINE_DEPENDENCY_WORKFLOW.md` | modified |
| `docs/manual_qa_checklist.md` | modified |

### 3. Recent commit context

| Hash | Summary |
|---|---|
| `9b01b34` | Integrate engine audio/input updates |
| `0d6f4fa` | Fix survivor UI visual QA regressions |
| `fc8869b` | Fix BGM variant rollover regression |
| `f8f5b16` | Stabilize survivor UI layout and assets |
| `5d7c224` | session: repeat-flag-fix [bgm-path-hardening] |
| `cec4478` | Fix stageclear/gameover BGM looping forever instead of playing once |
| `cfb087a` | Harden survivor BGM asset resolution |
| `4944312` | Polish survivor text UI assets |

### 4. Engine dependency before vs after

| Location | Before | After |
|---|---|---|
| `crates/game/Cargo.toml` | `engine = { package = "skeleton-engine", path = "../../../skeleton-engine" }` | `engine = { package = "skeleton-engine", git = "https://github.com/ChunSam/skeleton-engine", rev = "61c09f14434910a94afc4c9ea167b3f47e0b203b" }` |
| `Cargo.lock` source | local path package source behavior | git source with exact rev |
| Engine package version | local sibling v2 after prior update | `skeleton-engine v2.0.0` from pinned git |

### 5. `Cargo.lock` engine entry evidence

```text
name = "skeleton-engine"
version = "2.0.0"
source = "git+https://github.com/ChunSam/skeleton-engine?rev=61c09f14434910a94afc4c9ea167b3f47e0b203b#61c09f14434910a94afc4c9ea167b3f47e0b203b"
```

### 6. Validation results

| Command | Result |
|---|---|
| `cargo generate-lockfile` | first sandbox attempt failed on network; escalated rerun passed |
| `cargo check -p game --bin survivor --locked` | first sandbox attempt failed on crates.io DNS; escalated rerun passed |
| `cargo fmt --check` | passed |
| `cargo test -p game --lib --locked -- --test-threads=1` | `200 passed; 0 failed` |
| `cargo build -p game --bin survivor --release --locked` | passed |
| `bash scripts/package_macos.sh` | passed, generated/verified folder and `.app` package |
| `bash scripts/verify_macos_package.sh` | passed |
| `git diff --check` | passed |

### 7. Test count progression

| Point | Count |
|---|---:|
| Parent handoff final lib tests | 189 |
| After this gameplay debt pass | 200 |
| Net new tests | 11 |

### 8. New/updated test coverage

| Area | Test behavior |
|---|---|
| Level-up meta counts | `levelup_actions_read_counts_from_meta` |
| Skip | `skip_levelup_consumes_count_and_resumes` |
| Reroll | `reroll_offer_consumes_count_and_keeps_pending` |
| Banish | `banish_offer_consumes_count_and_excludes_card` |
| Greed | `coin_gold_value_rounds_greed_with_minimum_one` |
| Luck | `normal_pickup_thresholds_scale_and_clamp_luck` |
| Curse default | `enemy_curse_multiplier_defaults_without_player` |
| Curse spawn effect | `enemy_spawn_applies_curse_to_stats_not_collider` |
| Revival first death | `death_system_consumes_revival_before_game_over` |
| Revival spent | `death_system_game_over_after_revival_is_spent` |
| Run reset | `reset_to_title_clears_run_only_action_state` |

### 9. One failed test and fix

| Failed test | Failure | Fix |
|---|---|---|
| `survivor::pickup::tests::normal_pickup_thresholds_scale_and_clamp_luck` | exact float compare expected `0.08`, got `0.080000006` | changed assertion to epsilon-based approximate comparison |

### 10. Runtime input mapping

| Key | Behavior |
|---|---|
| `1` | choose offered card 1 |
| `2` | choose offered card 2 |
| `3` | choose offered card 3 |
| `R` | reroll current offer if `reroll_remaining > 0` |
| `S` | skip pending level-up if `skip_remaining > 0` |
| `4` | banish offered card 1 if `banish_remaining > 0` |
| `5` | banish offered card 2 if `banish_remaining > 0` |
| `6` | banish offered card 3 if `banish_remaining > 0` |

### 11. Balance constants added or made active

| System | Constant/rule |
|---|---|
| Luck | clamp `0.0..=5.0` |
| Rosary threshold | `0.001 * luck` |
| Chicken cumulative threshold | `0.006 * luck` |
| Coin cumulative threshold | `0.016 * luck` |
| Greed coin value | `round(greed).max(1)` after `greed.max(0.0)` |
| Curse | clamp `0.5..=3.0` |
| Curse affected enemy stats | HP, move speed, contact damage |
| Revival HP restore | `max_hp * 0.5`, minimum `1.0` |

### 12. Package verification evidence

Package scripts verified the macOS folder and app:

```text
Verified macOS packages:
  /Users/jkl/Projects/rust-survivors/dist/macos/RustSurvivors
  /Users/jkl/Projects/rust-survivors/dist/macos/RustSurvivors.app
```

## Code Analysis

- `LevelUpSystem` uses helper functions to keep borrow scopes short:
  - input is read into `LevelUpInputAction` first,
  - then the player entity is located,
  - then the action mutates world state.
- This is important because `World` resources/components cannot be mutably borrowed while `InputState` or `PendingLevelUp` immutable borrows are still live.
- `available_cards_for` gracefully returns an empty vector if the player lacks expected inventories rather than panicking.
- `reroll_offer` checks available cards before decrementing count, so impossible rerolls do not consume a reroll.
- `banish_offer` simulates the banished list before mutating it and refuses to consume a banish if fewer than three cards would remain available.
- `skip_levelup` routes through `consume_level_without_card`, mirroring the XP-level progression part of `apply_card` without touching inventories.
- `RevivalState` is not inserted until needed, so normal runs without revival avoid extra resource state.
- `reset_run_state` clears `LevelUpActions`, `RevivalState`, and `PendingLevelUp`; this avoids carryover between gameover restart/title return and the next run.
- `enemy_curse_multiplier` reads `PlayerStats` from the world at spawn time. Already-spawned enemies do not update if curse changes later.
- `try_drop_normal_pickup` reads current player luck at death/drop time, so luck changes during a run affect future drops.
- `apply_pickup_effect` reads current player greed at coin pickup time, so greed changes during a run affect future coins.
- HUD hint rendering uses localized template strings and simple `.replace(...)` calls. It does not add a new formatter abstraction.
- `Cargo.lock` changed more registry transitive versions because `cargo generate-lockfile` recalculated the dependency graph after switching source form.

## Files Changed

### Source code

- `crates/game/Cargo.toml` — switched `skeleton-engine` from local path dependency to pinned git rev `61c09f14434910a94afc4c9ea167b3f47e0b203b`.
- `Cargo.lock` — regenerated lockfile for pinned engine git source and current compatible dependency graph.
- `crates/game/src/survivor/weapon.rs` — kept engine v2 compatibility by sorting entities with `Entity::index()`.
- `crates/game/src/survivor/levelup.rs` — added `LevelUpActions`, reroll/skip/banish helpers, key handling, offer regeneration, and tests.
- `crates/game/src/survivor/hud.rs` — displays remaining reroll/skip/banish counts on the pending level-up overlay.
- `crates/game/src/survivor/locale.rs` — added `UiText::LevelUpActions` with KO/EN templates and included it in `UiText::ALL`.
- `crates/game/src/survivor/pickup.rs` — connects luck to normal pickup drop thresholds and greed to coin value; adds helper tests.
- `crates/game/src/survivor/spawn.rs` — connects curse to enemy HP/speed/contact damage at spawn; adds helper/tests.
- `crates/game/src/survivor/death.rs` — adds `RevivalState`, revival-before-game-over behavior, reset cleanup, and tests.

### Tests

- `levelup.rs` — Reroll/Skip/Banish counts and behavior tests.
- `pickup.rs` — luck threshold and greed coin-value tests.
- `spawn.rs` — curse multiplier and collider-preservation tests.
- `death.rs` — revival consumption, spent-revival gameover, and run-state reset tests.
- Existing SFX tests from parent remain passing.

### Documentation

- `AGENTS.md` — updated dependency posture, controls, current state date, engine rev, test count, and next priorities.
- `docs/ENGINE_DEPENDENCY_WORKFLOW.md` — rewrote from local path workflow to pinned git workflow.
- `docs/manual_qa_checklist.md` — added automated validation results and remaining manual QA checks for SFX, level-up actions, stats, and visual responsiveness.
- `plans/handoffs/HANDOFF_engine-audio-update_gameplay-actions_2026-06-04.md` — this handoff.

### Generated artifacts

- `dist/macos/RustSurvivors`
- `dist/macos/RustSurvivors.app`
- These are outputs from `scripts/package_macos.sh`; they were not shown in `git status -s` and are not intended to be staged unless future release process says otherwise.

## User Feedback & Preferences (REQUIRED — never omit)

- User works primarily in Korean for this repo.
- User explicitly supplied the active AGENTS.md instructions, including "do not edit external skeleton-engine checkouts" and "keep changes scoped to crates/game unless docs/assets/package scripts are asked."
- User previously corrected that the requested improvement plan was for the project, not for the `$grill-me` skill.
- User requested "답변 모두 추천으로" earlier, which led to recommending pinned git rev and implementing all gameplay debt.
- User accepted/used the recommendation to pin engine dependency for release/CI reproducibility.
- User accepted/used the recommendation to implement all inert stats and Reroll/Skip/Banish rather than hide them.
- User provided a complete plan and said `PLEASE IMPLEMENT THIS PLAN`; this means implement, do not stop at another proposal when plan is decision-complete.
- User then invoked `[$handoff] ... 하고 커밋 푸쉬`, so handoff, commit, and push are all requested now.
- User expects concise final reporting with concrete command results.
- AGENTS.md says the user is a Rust beginner; if explaining later, mention borrow-checker/lifetime reasons briefly and concretely.
- User wants unrelated dirty worktree changes preserved; no unrelated reverts should be performed.

## Where We're Going

1. Finish this handoff self-validation, then stage all intended files including this handoff.
2. Commit on `main`; suitable message: `Implement engine v2 gameplay action pass`.
3. Push `main` to the configured remote.
4. Next session should run manual in-app QA for:
   - SFX volume 100/50/0 and BGM/SFX independence,
   - Level-up `R/S/4/5/6` display/input at 800x600,
   - Luck/Greed/Curse/Revival runtime feel,
   - basic 800x600/default/HiDPI visual overlap checks.
5. If manual QA finds visual overlap in the level-up action hint, adjust `LevelUpLayout` or the hint baseline/font size rather than changing the controls.

## Risks & Blockers

- Manual speaker output was not tested; SFX bus behavior is structurally tested/build-verified but not perceptually verified.
- Manual live-window visual QA was not run; 800x600/HiDPI hint placement could still need tuning.
- `Cargo.lock` was regenerated after switching dependency source form and updated some registry transitive versions; tests/build/package passed, but reviewers may want to inspect the lockfile diff.
- `CLAUDE.md` is stale and still mentions old `/Volumes/SSD/...` paths and older dependency form. Prefer `AGENTS.md` until `CLAUDE.md` is refreshed.
- New Reroll/Skip/Banish behavior is deterministic in counts but random in offers, so manual QA should focus on invariants rather than exact card identities.

## Open Questions

- Should `CLAUDE.md` be refreshed to mirror the new pinned engine dependency and `200` test count? It was not changed in this session.
- Should revival get a unique SFX later instead of reusing `SfxEvent::LevelUp`?
- Should greed eventually use fractional carry rather than `round(greed)`? Current implementation matches the plan and tests.
- Should banish controls remain `4/5/6`, or should a later UI pass add highlight/cursor support? Current implementation intentionally avoided new cursor state.

## Quick Start for Next Session

```bash
# Prior context
sed -n '1,360p' plans/handoffs/HANDOFF_engine-audio-update_sfx-bus-hidpi_2026-06-03.md
sed -n '1,420p' plans/handoffs/HANDOFF_engine-audio-update_gameplay-actions_2026-06-04.md

# Current source of truth
sed -n '1,180p' AGENTS.md
sed -n '1,120p' docs/ENGINE_DEPENDENCY_WORKFLOW.md
sed -n '1,60p' docs/manual_qa_checklist.md

# Key files to read first
sed -n '270,1040p' crates/game/src/survivor/levelup.rs
sed -n '1,260p' crates/game/src/survivor/pickup.rs
sed -n '1,180p' crates/game/src/survivor/spawn.rs
sed -n '100,230p' crates/game/src/survivor/death.rs

# Verify current state
cargo test -p game --lib --locked -- --test-threads=1

# Next action
# Run manual app QA for SFX volume and level-up action controls, then document results in docs/manual_qa_checklist.md.
```

## Handoff Self-Check

- Chain is valid: `engine-audio-update` seq `2`.
- Parent exists: `plans/handoffs/HANDOFF_engine-audio-update_sfx-bus-hidpi_2026-06-03.md`.
- Prior chain breadcrumb lists the parent and this file.
- "Where We Are" names the actual modified files and functions/resources added.
- "What We Tried" includes failed lockfile/check attempts and the failed exact-float test.
- "Evidence & Data" includes exact commands, counts, lockfile evidence, and validation outcomes.
- "User Feedback & Preferences" is present and includes direct user process preferences.
- "Quick Start for Next Session" has a concrete first action: manual app QA for SFX and level-up action controls.
