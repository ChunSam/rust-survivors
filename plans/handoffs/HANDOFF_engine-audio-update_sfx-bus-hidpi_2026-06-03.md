# Apply skeleton-engine audio/input update to survivor SFX and title input

**Date:** 2026-06-03
**Status:** COMPLETED
**Bead(s):** none
**Epic:** Rust Survivors release hardening
**Chain:** `engine-audio-update` seq `1`
**Parent:** `none — first in chain`
**Prior chain:** `none — first in chain`

---

## Related Handoffs

- `plans/handoffs/HANDOFF_bgm-path-hardening_executable-audio-paths_2026-05-31.md` — separate audio chain that hardened executable-relative BGM asset lookup.
- `plans/handoffs/HANDOFF_bgm-path-hardening_repeat-flag-fix_2026-05-31.md` — separate audio chain follow-up that fixed StageClear/GameOver one-shot repeat behavior.
- `plans/handoffs/HANDOFF_bgm-path-hardening_variant-rollover-regression_2026-06-01.md` — separate audio chain follow-up that fixed repeating BGM variant rollover after natural track finish.
- `plans/handoffs/HANDOFF_survivor-ui-hud-layout_visual-qa-fixes_2026-06-02.md` — recent UI/visual QA chain; related only because this session also touched title-menu click handling.

## Reference Documents

- `AGENTS.md` — active repo instructions, engine boundary, validation commands, and next priorities.
- `CLAUDE.md` — older agent notes; useful but stale on dependency location and test count. Prefer `AGENTS.md` and current command output when they conflict.
- `docs/ENGINE_DEPENDENCY_WORKFLOW.md` — explains the current local path dependency on `/Users/jkl/Projects/skeleton-engine`.
- `docs/ENGINE_CHANGE_REQUESTS.md` — historical engine requests; no new engine request was needed in this session.
- `docs/audio_assets.md` — audio asset naming and BGM behavior reference.

## The Goal

The session goal was to check the current `skeleton-engine` update, apply it safely to `rust-survivors`, and then identify game-side improvements unlocked by the new engine behavior. The concrete implemented improvement was SFX volume architecture: use the engine's bus volume support so user SFX volume is applied once at the bus level, while each event keeps its own base gain. A second compatibility fix was kept from the earlier engine-update pass: title-menu mouse hit testing now uses the engine's logical cursor and press-time cursor snapshot instead of the old physical-pixel workaround. The work stayed inside the game repo and used only public engine APIs.

## Where We Are

- Branch at handoff creation: `main`.
- Worktree before this handoff file had three tracked modified files: `Cargo.lock`, `crates/game/src/survivor/meta.rs`, and `crates/game/src/survivor/sfx.rs`.
- After this handoff file is written, the handoff itself is also a new tracked-intended artifact under `plans/handoffs/`.
- `Cargo.lock` now records `skeleton-engine` package version `1.3.0` instead of `1.1.0`.
- The engine dependency remains a local path dependency from `crates/game/Cargo.toml`: `engine = { package = "skeleton-engine", path = "../../../skeleton-engine" }`.
- Local sibling engine checkout used by the build is `/Users/jkl/Projects/skeleton-engine`.
- Local sibling engine HEAD observed during this session: `02ec251 feat(ui): TextInput single-line horizontal scroll + IME max_len honesty (v1.3.0)`.
- Remote `https://github.com/ChunSam/skeleton-engine` `main` observed earlier in the session was still `e9a666fb351fa30954e3cb75b994af6d91e9d5e5`; local engine was ahead at `02ec251` when `Cargo.lock` moved to `1.3.0`.
- No engine checkout files were edited from this repo.
- `crates/game/src/survivor/meta.rs` no longer imports `DisplayScaleFactor`.
- `meta.rs` no longer has `logical_cursor_position(cursor, display_scale)`.
- `handle_title_input` now uses `i.mouse_press_cursor(MouseButton::Left)` when detecting title image button clicks.
- The old test `physical_cursor_is_converted_to_logical_for_retina_clicks` was replaced with `logical_cursor_hits_title_button_on_retina_windows`.
- The title-click fix matches `skeleton-engine` 1.2.0+ behavior: `InputState::cursor()` is already logical-pixel based, and press/release cursor snapshots exist for click hit testing.
- `crates/game/src/survivor/sfx.rs` now defines `const SFX_BUS: &str = "sfx";`.
- `SfxSystem::run` reads `MetaSave.sfx_volume` as before, but now calls `audio.set_bus_volume(SFX_BUS, volume)` once before draining playback events.
- `prepare_sfx_channel(audio, channel, gain)` assigns each SFX channel to the `"sfx"` bus and sets its channel base gain.
- `play_file(audio, channel, key, gain)` now uses event base gain, not user volume.
- `play_tone(audio, channel, freq, duration_secs, gain)` assigns the tone channel to the `"sfx"` bus and calls `AudioManager::play_tone` with event base gain.
- Tone fallbacks no longer multiply gain by `MetaSave.sfx_volume`.
- File-backed SFX no longer use `MetaSave.sfx_volume` as per-channel gain.
- All user SFX volume scaling now flows through engine bus volume.
- `base_gains(event)` is the central table for event-relative loudness.
- `base_gain(event, index)` is a small accessor used for single-gain events.
- `SfxEvent` and `SfxQueue` public surfaces were not changed.
- Current event base gains are:
  - `EnemyHit`: `0.22`
  - `EnemyDie`: `0.35`
  - `PlayerHit`: `0.50`
  - `LevelUp`: `0.55`, `0.55`
  - `XpGem`: `0.12`
  - `Pickup`: `0.40`
  - `Bomb`: `0.70`, `0.45`
  - `ChestOpen`: `0.45`, `0.35`
  - `BossAppear`: `0.70`, `0.45`
- Frame cap behavior is unchanged: enemy hit max 2, enemy die max 2, XP gem max 3 per drained frame.
- Tone frequencies and durations are unchanged from the pre-bus implementation.
- New SFX test `sfx_base_gains_are_valid` verifies every event has at least one base gain and each gain is `0.0 < gain <= 1.0`.
- Existing SFX tests remain: unique file keys and placeholder file existence.
- Final lib test count is now `189` passing tests.
- Final release survivor build passed against `skeleton-engine v1.3.0`.
- Manual audio listening QA was not run; it remains a recommended follow-up if the user wants perceptual confirmation.
- The user requested `$handoff` plus commit and push after implementation.

## What We Tried (Chronological)

1. **Engine update check.**
   - Searched `Cargo.toml`, `Cargo.lock`, `crates`, and docs for `skeleton-engine`.
   - Found the game depends on sibling path `../../../skeleton-engine`, not a pinned git dependency.
   - Ran `git ls-remote https://github.com/ChunSam/skeleton-engine HEAD refs/heads/main` and initially saw remote `main` at `e9a666fb351fa30954e3cb75b994af6d91e9d5e5`.
   - Checked local engine status and HEAD; local engine matched `e9a666f` at that moment but had dirty engine-side work.
   - `Cargo.lock` already had a version-only diff from `1.1.0` to `1.2.1`.
   - Ran validation and confirmed `skeleton-engine v1.2.1` worked with the game.

2. **Initial validation after engine lock update.**
   - `cargo fmt --check` passed.
   - `cargo test -p game --lib --locked -- --test-threads=1` passed with `188` tests.
   - `cargo build -p game --bin survivor --release --locked` passed.
   - At this point, only `Cargo.lock` was modified in the game repo.

3. **Search for game improvements unlocked by engine update.**
   - Read `skeleton-engine/docs/CHANGELOG.md`.
   - Important engine changes identified:
     - `InputState` stores cursor in logical pixels for HiDPI hit testing.
     - `InputState` exposes press/release cursor positions.
     - `LocalizedText` and `LocalizationSystem` were added for engine UI widgets.
     - `TextInput` improved IME/caret behavior and later single-line horizontal scroll in local `v1.3.0`.
     - `AudioManager::play_tone` now applies bus volume/effects like file playback.
     - macOS live resize redraw freeze was fixed engine-side.
   - Searched game code for mouse/cursor handling, localization, TextInput, and audio calls.
   - Found one immediate game bug: `meta.rs` still divided cursor by `DisplayScaleFactor`, which is wrong after engine logical cursor conversion.

4. **HiDPI title-click compatibility fix.**
   - Removed `DisplayScaleFactor` import from `meta.rs`.
   - Removed `logical_cursor_position`.
   - Changed title-click action detection to use `InputState::mouse_press_cursor(MouseButton::Left)`.
   - Updated the Retina click test to assert that logical coordinates hit the Settings title button.
   - This is a game-side compatibility adjustment to the new engine input contract.

5. **Unexpected local engine version drift.**
   - Running a test without `--locked` but with `--offline` updated `Cargo.lock` from `skeleton-engine v1.2.1` to `v1.3.0`.
   - Investigation showed local sibling engine now had HEAD `02ec251 feat(ui): TextInput single-line horizontal scroll + IME max_len honesty (v1.3.0)`.
   - Local sibling engine status was clean after that point.
   - Remote `main` remained behind at `e9a666f` when checked, so the lockfile reflects local path dependency state rather than remote availability.

6. **Project improvement planning correction.**
   - User invoked `$grill-me` asking for an improvement plan.
   - The first interpretation was wrong: it planned improvements to the `grill-me` skill itself.
   - User corrected: intent was project improvement planning based on the additional engine-update candidates.
   - Re-ran code inspection around `sfx.rs`, `bgm.rs`, `meta.rs`, settings, localization, and engine audio APIs.
   - User chose recommended options: focus on audio improvement and keep it a small safe change.

7. **SFX bus improvement implementation.**
   - Added `SFX_BUS`, `prepare_sfx_channel`, `play_tone`, `base_gains`, and `base_gain` to `sfx.rs`.
   - Moved user volume from per-event gain multiplication into one `audio.set_bus_volume(SFX_BUS, volume)` call.
   - Assigned every SFX channel to `SFX_BUS` before file or tone playback.
   - Kept current relative loudness, frame caps, file keys, and fallback tone frequencies/durations.
   - Added `sfx_base_gains_are_valid` test.

8. **Targeted and full validation.**
   - `cargo fmt --check` passed.
   - `cargo test -p game --lib --locked sfx -- --test-threads=1` passed with `3` SFX tests.
   - Full `cargo test -p game --lib --locked -- --test-threads=1` passed with `189` tests.
   - `cargo build -p game --bin survivor --release --locked` passed.
   - No manual app run or audio-listening QA was performed.

9. **Handoff, commit, and push request.**
   - User explicitly requested `[$handoff] ... 하고 커밋 푸쉬`.
   - This file is the handoff.
   - Next steps after writing and validating this file: stage changed files, commit, push `main`.

## Key Decisions

- **Keep this as a game-side update.**
  - Reason: AGENTS.md forbids direct engine edits from this repo, and the needed engine APIs already exist.
  - Rejected alternative: edit `/Users/jkl/Projects/skeleton-engine` or create a new engine request.

- **Treat local path dependency as the active engine source.**
  - Reason: `crates/game/Cargo.toml` uses `path = "../../../skeleton-engine"` and `docs/ENGINE_DEPENDENCY_WORKFLOW.md` documents that workflow.
  - Consequence: `Cargo.lock` package version follows local sibling engine package metadata.

- **Preserve the platformer demo and all public game data surfaces.**
  - Reason: AGENTS.md explicitly says to preserve platformer demo unless requested otherwise.
  - This session did not touch `crates/game/src/main.rs`, package scripts, assets, save schema, or menu option count.

- **Use `mouse_press_cursor` for title hit testing.**
  - Reason: engine 1.2.0 records cursor at press/release time to avoid wrong click hit tests when movement occurs in the same frame.
  - Rejected alternative: keep using live `cursor()` for click activation.

- **Remove the old Retina scale workaround.**
  - Reason: engine now stores `InputState::cursor()` in logical pixels; dividing again by display scale breaks HiDPI title clicks.
  - Rejected alternative: leave `DisplayScaleFactor` correction in game code.

- **Use an SFX bus for user volume.**
  - Reason: engine `play_tone` now respects bus volume and effects, so SFX can match file playback semantics.
  - Rejected alternative: continue multiplying every tone/file gain by `MetaSave.sfx_volume`.

- **Keep base gains separate from user volume.**
  - Reason: event loudness and user settings are different concepts; separating them makes later effects or bus changes safer.
  - Rejected alternative: store only final effective volumes.

- **Do not add `AudioManager` device tests.**
  - Reason: tests should not require a real audio device; this repo already avoids audio device init in tests where possible.
  - Rejected alternative: instantiate native audio sinks in lib tests.

- **Do not migrate settings UI to engine `TextInput`/`LocalizedText` now.**
  - Reason: current survivor UI is largely custom `TextQueue` plus image assets; migration is larger than the requested small safe audio task.
  - Rejected alternative: restructure settings screen around engine UI widgets.

- **Do not update manual QA docs for this audio-only behavior change.**
  - Reason: AGENTS.md says manual QA notes are relevant for visual changes; this work is code/test backed and no asset packaging changed.

## Evidence & Data

### 1. Git state before handoff

| Command | Result |
|---|---|
| `git branch --show-current` | `main` |
| `git status -s` | `M Cargo.lock`; `M crates/game/src/survivor/meta.rs`; `M crates/game/src/survivor/sfx.rs` |
| `git diff --stat` before handoff file | `3 files changed, 82 insertions(+), 39 deletions(-)` |

### 2. Relevant recent commits

| Hash | Summary |
|---|---|
| `0d6f4fa` | Fix survivor UI visual QA regressions |
| `fc8869b` | Fix BGM variant rollover regression |
| `f8f5b16` | Stabilize survivor UI layout and assets |
| `5d7c224` | session: repeat-flag-fix [bgm-path-hardening] |
| `cec4478` | Fix stageclear/gameover BGM looping forever instead of playing once |
| `cfb087a` | Harden survivor BGM asset resolution |
| `4944312` | Polish survivor text UI assets |
| `9e9223a` | Preserve survivor UI and sprite image aspects |

### 3. Engine dependency state

| Item | Value |
|---|---|
| Game dependency form | local path dependency |
| Path from `crates/game` | `../../../skeleton-engine` |
| Resolved sibling checkout | `/Users/jkl/Projects/skeleton-engine` |
| Current lock package version | `skeleton-engine 1.3.0` |
| Earlier remote `main` check | `e9a666fb351fa30954e3cb75b994af6d91e9d5e5` |
| Later local engine HEAD | `02ec251 feat(ui): TextInput single-line horizontal scroll + IME max_len honesty (v1.3.0)` |

### 4. Validation commands and results

| Command | Result |
|---|---|
| `cargo fmt --check` | passed |
| `cargo test -p game --lib --locked sfx -- --test-threads=1` | `3 passed; 0 failed; 186 filtered out` |
| `cargo test -p game --lib --locked -- --test-threads=1` | `189 passed; 0 failed` |
| `cargo build -p game --bin survivor --release --locked` | passed, compiled `skeleton-engine v1.3.0` and `game v0.1.0` |

### 5. Test count progression in this session

| Point | Lib test count |
|---|---:|
| After initial engine `v1.2.1` lock validation | 188 |
| After HiDPI input compatibility fix | 188 |
| After SFX base gain test addition | 189 |

### 6. SFX behavior before vs after

| Behavior | Before | After |
|---|---|---|
| User SFX volume | multiplied into every file/tone gain | applied once via `set_bus_volume("sfx", volume)` |
| File SFX channel volume | `MetaSave.sfx_volume` or final event volume | event base gain only |
| Tone SFX volume | `base_gain * MetaSave.sfx_volume` | base gain only |
| Engine bus assignment | none | every SFX channel assigned to `"sfx"` |
| Event frame caps | hit 2, die 2, XP 3 | unchanged |
| Public `SfxEvent`/`SfxQueue` | unchanged | unchanged |

### 7. Current base gain table

| Event | Gains |
|---|---|
| `EnemyHit` | `0.22` |
| `EnemyDie` | `0.35` |
| `PlayerHit` | `0.50` |
| `LevelUp` | `0.55`, `0.55` |
| `XpGem` | `0.12` |
| `Pickup` | `0.40` |
| `Bomb` | `0.70`, `0.45` |
| `ChestOpen` | `0.45`, `0.35` |
| `BossAppear` | `0.70`, `0.45` |

### 8. Engine API evidence used

| Engine API | Why it mattered |
|---|---|
| `InputState::mouse_press_cursor(MouseButton)` | Use press-time coordinate for title button hit testing |
| `InputState::cursor()` logical-pixel behavior | Removed old display-scale division |
| `AudioManager::assign_bus(channel, bus)` | Attach SFX channels to `"sfx"` bus |
| `AudioManager::set_bus_volume(bus, volume)` | Apply user SFX volume once |
| `AudioManager::set_volume(channel, gain)` | Store event base gain for file playback |
| `AudioManager::play_tone(channel, freq, duration, gain)` | Tone gain now combines with bus volume in engine |

### 9. Raw validation snippets

```text
running 3 tests
test survivor::sfx::tests::sfx_base_gains_are_valid ... ok
test survivor::sfx::tests::sfx_events_have_unique_file_keys ... ok
test survivor::sfx::tests::sfx_placeholder_files_exist_for_release_package ... ok
test result: ok. 3 passed; 0 failed; 186 filtered out
```

```text
running 189 tests
...
test survivor::sfx::tests::sfx_base_gains_are_valid ... ok
...
test result: ok. 189 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

```text
Compiling skeleton-engine v1.3.0 (/Users/jkl/Projects/skeleton-engine)
Compiling game v0.1.0 (/Users/jkl/Projects/rust-survivors/crates/game)
Finished `release` profile [optimized] target(s) in 10.13s
```

## Code Analysis

- `SfxSystem` drains `SfxQueue` into `self.events` before borrowing `AudioManager`, preserving the existing borrow-safe structure.
- `audio.set_bus_volume(SFX_BUS, volume)` is called only after confirming there are events and `AudioManager` exists. This avoids touching audio state when no SFX are emitted.
- `prepare_sfx_channel` calls `assign_bus` before `set_volume`. `assign_bus` immediately reapplies effective volume if a sink exists; `set_volume` then sets the base channel gain and also updates the sink.
- `play_file` calls `audio.play(...)` first and then `prepare_sfx_channel(...)`. This ensures newly-created file sinks receive the assigned bus and base gain immediately after playback starts.
- `play_tone` assigns the bus before `AudioManager::play_tone(...)`, so the engine can apply the bus volume to the new tone sink at creation time.
- `base_gains` deliberately contains more than one gain for layered events such as `Bomb`, `ChestOpen`, `BossAppear`, and `LevelUp`.
- `base_gain(event, 0)` is used for one-layer events and keeps magic numbers out of playback branches.
- The SFX file fallback behavior is unchanged: if a matching `assets/audio/{key}.ogg` or `.wav` exists, file playback wins; otherwise tone fallback is used.
- `meta.rs` title-click logic now trusts engine logical cursor coordinates. This matters for Retina and any scaled display where old code divided by the scale factor twice.
- `logical_cursor_hits_title_button_on_retina_windows` is not a full integration test of winit scaling; it is a regression guard for the game-side title-layout coordinate contract.

## Files Changed

### Source code

- `Cargo.lock` — updated `skeleton-engine` package version from `1.1.0` to `1.3.0` due to local path dependency resolving the current sibling engine checkout.
- `crates/game/src/survivor/meta.rs` — removed stale display-scale cursor workaround and switched title clicks to `mouse_press_cursor(MouseButton::Left)`.
- `crates/game/src/survivor/sfx.rs` — added SFX bus support, separated user volume from base event gain, assigned all SFX channels to `"sfx"`, and centralized SFX base gains.
- `plans/handoffs/HANDOFF_engine-audio-update_sfx-bus-hidpi_2026-06-03.md` — this handoff.

### Tests

- `crates/game/src/survivor/meta.rs` — updated Retina/title-click test to logical cursor contract.
- `crates/game/src/survivor/sfx.rs` — added `sfx_base_gains_are_valid`.

### Data & results

- No new data/result files were created.

### Config

- No config files were changed.

## User Feedback & Preferences (REQUIRED — never omit)

- User supplied the active `AGENTS.md` instructions for `rust-survivors`.
- User asked in Korean: `엔진 업데이트 확인하고 적용`.
- User then asked to find game updates or improvements aligned with the engine update.
- User invoked `$grill-me`; initial assistant interpretation incorrectly planned improvements to the `grill-me` skill itself.
- User corrected the intent: `사용자 의도는 스킬 개선 게획이 아니라. 프로젝트 개선 게획이였음. 추가 개선 후보 보고 개선 계획 세워줘`.
- User accepted recommended planning choices for project improvement: focus on audio improvement and keep it a small safe change.
- User provided a concrete implementation plan and said `PLEASE IMPLEMENT THIS PLAN`.
- User expects direct implementation after a plan is supplied; avoid stopping at another proposal when the plan is decision-complete.
- User asked for `$handoff` and commit/push in Korean: `[$handoff] ... 하고 커밋 푸쉬`.
- User works in Korean for this repo; final status reports should be concise Korean unless they ask otherwise.
- User is a Rust beginner per AGENTS.md; explain non-obvious borrow/lifetime choices briefly when they arise.
- User wants unrelated dirty worktree changes preserved; do not revert changes not made in the active task.

## Where We're Going

1. Stage `Cargo.lock`, `crates/game/src/survivor/meta.rs`, `crates/game/src/survivor/sfx.rs`, and this handoff.
2. Commit on `main` with a message such as `Integrate engine audio/input updates`.
3. Push `main` to the configured remote.
4. If a future session continues audio work, run manual listening QA for SFX volume at 100%, 50%, and 0%.
5. If the engine remote still does not contain `02ec251`, decide whether release/CI should wait for upstream push or pin a reachable engine commit.

## Risks & Blockers

- Local `Cargo.lock` now reflects `skeleton-engine v1.3.0` from the sibling checkout. Earlier remote check showed GitHub `main` still at `e9a666f`, so another machine may not reproduce this exact state unless the engine repo is updated or present locally.
- Manual audio QA was not run, so perceptual loudness balance is validated structurally but not by listening.
- `CLAUDE.md` still mentions older `/Volumes/SSD/...` paths and git dependency form. Prefer `AGENTS.md` plus `docs/ENGINE_DEPENDENCY_WORKFLOW.md`.
- `AGENTS.md` latest verified test count is stale at 118; current validated lib count is 189.

## Open Questions

- Should release/CI continue using local path dependency, or switch back to a pinned git revision once `skeleton-engine v1.3.0` is reachable upstream?
- Should a future pass add SFX bus effects such as low-pass/pitch for Bomb or BossAppear, now that engine tones honor effects?
- Should manual QA notes be updated after a listening pass, even though no visual/package asset change happened?

## Quick Start for Next Session

```bash
# Restore context
sed -n '1,360p' plans/handoffs/HANDOFF_engine-audio-update_sfx-bus-hidpi_2026-06-03.md

# Related audio history
sed -n '1,220p' plans/handoffs/HANDOFF_bgm-path-hardening_variant-rollover-regression_2026-06-01.md

# Reference docs
sed -n '1,180p' AGENTS.md
sed -n '1,220p' docs/ENGINE_DEPENDENCY_WORKFLOW.md

# Key files to read first
sed -n '1,260p' crates/game/src/survivor/sfx.rs
sed -n '360,410p' crates/game/src/survivor/meta.rs
sed -n '1010,1032p' crates/game/src/survivor/meta.rs

# Verify current state
cargo fmt --check
cargo test -p game --lib --locked -- --test-threads=1
cargo build -p game --bin survivor --release --locked

# Next action
# If continuing audio QA, run survivor and verify Settings SFX volume at 100%, 50%, and 0%
# using XP pickup, enemy hit, bomb, and level-up sounds; confirm BGM volume is independent.
```

## Handoff Self-Check

- Chain is valid: `engine-audio-update` seq `1`.
- Parent status is valid: `none — first in chain`.
- Related handoffs are listed but not treated as parents.
- "Where We Are" names real files and functions touched this session.
- "What We Tried" includes the wrong initial skill-plan interpretation and the corrected project-plan path.
- "Evidence & Data" includes exact commands, counts, hashes, version numbers, and behavior tables.
- "User Feedback & Preferences" is present and includes the user's correction and commit/push request.
- "Quick Start for Next Session" has concrete commands and one concrete next action.
