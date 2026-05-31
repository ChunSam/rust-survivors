# Implement repeating BGM variant rollover fix and runtime regression coverage

**Date:** 2026-06-01
**Status:** COMPLETED
**Bead(s):** none
**Epic:** Rust Survivors release hardening
**Chain:** `bgm-path-hardening` seq `3`
**Parent:** `HANDOFF_bgm-path-hardening_repeat-flag-fix_2026-05-31.md`
**Prior chain:** `HANDOFF_bgm-path-hardening_executable-audio-paths_2026-05-31.md` > `HANDOFF_bgm-path-hardening_repeat-flag-fix_2026-05-31.md` > this

---

## Related Handoffs

- `plans/handoffs/HANDOFF_survivor-ui-hud-layout_2026-06-01.md` — separate UI/layout stabilization stream; it mentioned `bgm.rs` as unrelated dirty worktree state at the time.
- `plans/handoffs/HANDOFF_boss-clear-trigger_2026-05-31.md` — separate gameplay bugfix for StageClear trigger conditions, not audio playback.

## Since Last Handoff

- Parent seq 2 fixed a one-shot repeat regression by restoring `stageclear` and `gameover` to `repeat=false`, but it kept looping keys on rodio repeat and therefore did not exercise actual variant rollover.
- A later security/code-review pass found the new mismatch: repeating keys were started with `repeat=true`, so `audio.is_finished("bgm")` could never become `Some(true)` and the manual rollover branch was dead.
- The user first requested a concise Korean code-review report only; no code edits were made during that pass.
- Then the user invoked `$grill-me`, forced clarification on scope, and locked a narrow plan: BGM-only, game-side only, preserve one-shot behavior and path resolution, add runtime-policy regression coverage.
- The final implementation followed that plan exactly and touched only `crates/game/src/survivor/bgm.rs`.
- Parent seq 2 marked the audio chain “COMPLETED”; that was premature. This seq 3 closes the remaining functional gap in the repeating multi-variant behavior promised by `docs/audio_assets.md`.

## Reference Documents

- `AGENTS.md` — repo constraints, engine boundary, validation commands, and current priority notes
- `CLAUDE.md` — repo conventions and next-work framing
- `docs/audio_assets.md` — authoritative BGM naming rules and expected rollover behavior
- `docs/ENGINE_CHANGE_REQUESTS.md` — useful historical note that channel-finished visibility was once an engine gap; current engine already exposes `AudioManager::is_finished`
- `plans/handoffs/HANDOFF_bgm-path-hardening_executable-audio-paths_2026-05-31.md` — seq 1 path-hardening background
- `plans/handoffs/HANDOFF_bgm-path-hardening_repeat-flag-fix_2026-05-31.md` — seq 2 one-shot repeat fix

## The Goal

The goal of this session was to repair the still-broken part of the BGM playlist design: repeating survivor BGM (`title`, `ingame`, `boss`) was supposed to advance to the next MP3 variant after natural track end, but current runtime behavior never reached that branch. The user explicitly wanted implementation, not another plan, after a clarification round locked the scope. The work had to stay on the game side, preserve the recent executable-relative asset resolution and one-shot StageClear/GameOver behavior, and add regression coverage strong enough to catch this exact class of bug in the future. The end state is that `BgmSystem` now treats all BGM as non-looping at the rodio layer and performs controlled re-queue itself when the channel reports finished.

## Where We Are

- Branch is still `main`.
- Git worktree at handoff time has exactly one modified tracked file: `crates/game/src/survivor/bgm.rs`.
- `git diff --stat` reports `163` changed lines in `bgm.rs`, with `134` insertions and `29` deletions.
- No other tracked files were edited in this session.
- No engine checkout files were modified.
- No docs were changed in-repo during this implementation session.
- `crates/game/src/survivor/bgm.rs` now contains a new helper `bgm_restarts_after_finish(key)` at lines 71-73.
- `bgm_repeat_flag(_key)` at lines 80-82 now always returns `false`, which is intentional: all BGM playback is manually re-queued so `is_finished("bgm")` remains observable.
- New enum `BgmAction` at lines 84-88 represents the runtime policy decision surface: `KeepCurrent` or `PlayPath(String)`.
- New pure helper `choose_bgm_action(...)` at lines 90-113 centralizes the rollover policy independently from `AudioManager`.
- `choose_bgm_action` returns `None` only when playlist input is empty; otherwise it returns a precise decision for current-key steady state or key-switch entry state.
- When `current == Some(target)` and `finished == Some(true)` for repeating keys, `choose_bgm_action` increments `next_variant` and returns `PlayPath(next_path)`.
- When `current == Some(target)` and `finished == Some(false)`, `choose_bgm_action` returns `KeepCurrent` and does not mutate `next_variant`.
- When key changes, `choose_bgm_action` always picks the next scheduled path and increments `next_variant`, regardless of repeat-vs-one-shot classification.
- `play_bgm_file` at lines 115-117 no longer accepts `variant_count`; it only receives `audio`, `path`, and `key`.
- `BgmSystem::choose_action` at lines 230-240 is the game-side seam that bridges runtime `AudioManager` state to the pure decision helper.
- `BgmSystem::choose_action` clones the per-slot playlist before calling the helper. This was necessary to avoid Rust borrow conflicts between `self.playlists`, `self.current`, and `self.next_variants`.
- The steady-state branch in `BgmSystem::run` now reads `audio.is_finished("bgm")` and calls `self.choose_action(target, finished)` instead of open-coding variant logic.
- The key-switch branch now calls `self.choose_action(target, Some(false))` to schedule the first track of the new mode through the same policy seam.
- Path resolution logic is unchanged: macOS bundle `Contents/Resources/assets/audio`, executable-neighbor lookup, parent-neighbor lookup, and debug-only workspace fallback still behave as seq 1 implemented them.
- One-shot policy is unchanged semantically: `bgm_stageclear` and `bgm_gameover` do not auto-restart after finishing.
- The old helper `bgm_playlist_advances` still exists but is now `#[cfg(test)]` only. It remains useful as a compact assertion surface for the documented policy “only repeating multi-track BGM can advance across variants.”
- Existing tests for asset discovery, path priority, missing-asset handling, and key-slot stability are all still present.
- New test `repeating_bgm_requeues_next_variant_after_finish` proves the exact regression fix with a two-item playlist and `finished = Some(true)`.
- New test `repeating_bgm_does_not_restart_while_still_playing` proves that no spurious restart occurs while the track is active.
- New test `one_shot_bgm_does_not_auto_advance_after_finish` proves that one-shot keys stay one-shot even if two files exist in the playlist.
- New test `single_variant_repeating_bgm_restarts_same_track_after_finish` covers the fallback case where a repeating key has only one surviving file.
- `bgm_repeat_flag_is_false_for_one_shot_keys` was updated to assert `false` for all five survivor BGM keys, because looping is now a game-policy concept rather than a rodio repeat flag.
- `bgm_playlist_advances_only_for_repeating_multi_track_bgm` was updated to check both `bgm_playlist_advances(...)` and `bgm_restarts_after_finish(...)`.
- The targeted BGM test subset grew from `11` tests in seq 2 to `15` tests in this session.
- Full lib test count grew from `182` to `186`.
- `cargo test -p game --lib --locked survivor::bgm::tests:: -- --test-threads=1` passed with `15 passed; 0 failed; 171 filtered out`.
- `cargo test -p game --lib --locked -- --test-threads=1` passed with `186 passed; 0 failed`.
- `cargo build -p game --bin survivor --release --locked` passed.
- A transient compile failure occurred mid-session due to borrow-checker conflicts inside the first `BgmSystem::choose_action` draft; the final code resolves it by cloning the playlist slice before mixing immutable and mutable access to `self`.
- A transient release-build warning (`dead_code` on `bgm_playlist_advances`) occurred after functional success; it was resolved by annotating the helper with `#[cfg(test)]`.
- Optional manual QA requested in the implementation plan was not run in this session.
- The user did not ask for a commit, so no commit was created.
- The local automation memory file outside the repo was updated with the implementation result and final validation counts.

## What We Tried (Chronological)

1. **Review-only pass before code changes.**
   - User’s automation asked for a security/safety/regression review of recent code.
   - Read git history, recent diffs, `bgm.rs`, dependency changes, and test output.
   - Found one concrete issue: repeating BGM started with `repeat=true`, so `audio.is_finished("bgm")` would never become `Some(true)` and the multi-variant rollover logic for `title`/`ingame`/`boss` was dead.
   - No code edits were made during this pass.

2. **Clarification pass via `$grill-me`.**
   - User explicitly asked for an “improvement plan” using the `grill-me` skill.
   - Clarified and locked: scope is BGM-only, intended final behavior is “advance to next variant after natural finish”, engine boundary is game-side only, path resolution and StageClear/GameOver one-shot policy must remain unchanged, proof bar is lib tests + release build, manual QA is optional.
   - This prevented scope creep into broader audio redesign or engine changes.

3. **First implementation draft: add a pure decision helper.**
   - Introduced `BgmAction`, `choose_bgm_action`, `bgm_restarts_after_finish`, and the new manual re-queue policy.
   - Changed `bgm_repeat_flag` to always return `false`.
   - Rewired both steady-state and key-switch playback paths through the new helper.
   - Added four new runtime-policy regression tests.

4. **First verification attempt failed on Rust borrows.**
   - Running lib tests produced `E0503` and `E0499` in `BgmSystem::choose_action`.
   - Root cause: `self.ensure_playlists()?` held a mutable borrow of `self`, while the function also needed `self.current` immutably and `self.next_variants[slot]` mutably in the same call.
   - This is exactly the sort of borrow-checker trap AGENTS.md said to explain briefly for the user.

5. **Borrow-checker fix applied.**
   - Adjusted `BgmSystem::choose_action` to clone the per-slot playlist into a local `Vec<String>` before calling `choose_bgm_action`.
   - That ends the borrow from `ensure_playlists()` before reading `self.current` or mutating `self.next_variants[slot]`.
   - Re-ran tests successfully after the patch.

6. **Second verification attempt exposed a release-only cleanup issue.**
   - Full lib tests and release build passed functionally.
   - Release build emitted `warning: function bgm_playlist_advances is never used`.
   - This warning existed because the helper had become assertion-only after the policy refactor.

7. **Final cleanup.**
   - Added `#[cfg(test)]` to `bgm_playlist_advances`.
   - Re-ran full lib tests and release build.
   - Final state is warning-free for the affected code path, and all requested validation commands pass.

8. **Automation memory update.**
   - Wrote a concise implementation summary to `/Users/jkl/.codex/automations/rust-survivors-security-code-monitor/memory.md`.
   - Included the new behavior, test counts, and the fact that manual QA remains optional/not run.

## Key Decisions

- **Keep all work in `crates/game/src/survivor/bgm.rs`.**
  - Reason: the user locked a game-side-only fix during the clarification pass, and the current engine already exposes `AudioManager::is_finished`.
  - Rejected alternative: write a new engine request or modify `skeleton-engine` directly.

- **Set rodio repeat to `false` for every survivor BGM key.**
  - Reason: variant rollover depends on a visible finished state; rodio repeat hides that state for looping keys.
  - Rejected alternative: keep `repeat=true` for repeating keys and try to infer rollover some other way. That would leave the original dead branch intact.

- **Introduce a pure `choose_bgm_action(...)` helper.**
  - Reason: runtime rollover behavior needed direct regression tests without depending on real audio sinks.
  - Rejected alternative: test only `bgm_repeats`/`bgm_repeat_flag` helper values again. That was too weak and is why the regression escaped in the first place.

- **Preserve StageClear/GameOver one-shot semantics exactly.**
  - Reason: previous seq 2 fixed that user-visible regression already, and the user explicitly classified it as preserve-not-redesign.
  - Rejected alternative: re-evaluate one-shot behavior while refactoring the repeating policy.

- **Preserve executable-relative path resolution untouched.**
  - Reason: seq 1 already hardened this area, and the current task was behavioral, not path-related.
  - Rejected alternative: combine rollover fix with path refactor or doc changes.

- **Use the same decision seam for both steady-state and mode-switch playback.**
  - Reason: one policy surface reduces divergence and keeps test logic aligned with production logic.
  - Rejected alternative: leave mode-switch logic open-coded and only abstract the steady-state branch.

- **Solve the borrow conflict by cloning the per-slot playlist.**
  - Reason: simplest safe Rust fix with the smallest scope and no lifetime gymnastics.
  - Rejected alternative: redesign struct ownership or split `self` across more locals/refcells just to avoid one `Vec<String>` clone on transition checks.

- **Keep `bgm_playlist_advances` as test-only instead of deleting it.**
  - Reason: it remains a useful documentation-level assertion of the multi-track advance rule.
  - Rejected alternative: remove it entirely and encode that policy only indirectly through other assertions.

- **Do not run manual QA automatically.**
  - Reason: the locked proof bar only required lib tests and release build; optional QA was not promoted to mandatory.
  - Rejected alternative: spend extra time reaching title/ingame/boss transitions manually before handoff.

- **Do not commit.**
  - Reason: user asked for implementation but not a commit in this session.
  - Rejected alternative: create an unsolicited commit and collapse the modified state prematurely.

## Evidence & Data

### 1. Git state snapshot

| Command | Result |
|---|---|
| `git branch --show-current` | `main` |
| `git status --short` | `M crates/game/src/survivor/bgm.rs` |
| `git diff --stat` | `crates/game/src/survivor/bgm.rs | 163 +++++++++++++++++++++++++++++++++-------` |

### 2. Chain history relevant to this work

| Seq | Commit / file | Meaning |
|---:|---|---|
| 1 | `cfb087a` / `HANDOFF_bgm-path-hardening_executable-audio-paths_2026-05-31.md` | executable-relative path hardening and MP3 playlist setup |
| 2 | `cec4478` / `HANDOFF_bgm-path-hardening_repeat-flag-fix_2026-05-31.md` | fixed StageClear/GameOver infinite looping |
| 3 | current uncommitted diff / this handoff | fixed repeating-key variant rollover dead branch |

### 3. User-locked scope and proof bar

| Question | Locked answer | Consequence |
|---|---|---|
| Improvement scope | `BGM 회귀 + 검증 보강만` | no audio redesign or unrelated fixes |
| Looping behavior | `곡 종료 후 다음 variant로 순환` | repeating keys must rely on observable finish state |
| Engine boundary | `예, game-side만` | no `skeleton-engine` edits |
| Preserve other behavior | `one-shot + current path resolution 보존` | no seq 1 or seq 2 rollback |
| Proof bar | `lib 테스트 + survivor 빌드` | manual QA optional only |

### 4. Old vs new repeat policy

| Key class | Old rodio repeat flag | Old rollover reachability | New rodio repeat flag | New rollover reachability |
|---|---|---|---|---|
| `title/ingame/boss` | `true` | dead: `is_finished("bgm")` never expected to surface | `false` | live: natural finish can trigger re-queue |
| `stageclear/gameover` | `false` after seq 2 | no auto-advance | `false` | still no auto-advance |

### 5. Runtime decision helper behavior matrix

| `current == target` | `finished` | key class | expected `BgmAction` | `next_variant` mutation |
|---|---|---|---|---|
| no | `Some(false)` | any | `PlayPath(next scheduled track)` | increment |
| yes | `Some(false)` | repeating | `KeepCurrent` | no change |
| yes | `Some(true)` | repeating | `PlayPath(next variant or same single variant)` | increment |
| yes | `Some(true)` | one-shot | `KeepCurrent` | no change |

### 6. Borrow-checker failure captured mid-session

| Error | Location | Cause | Fix |
|---|---|---|---|
| `E0503` | `bgm.rs:233` | `self.current` used while `self.ensure_playlists()?` still held a mutable borrow | clone playlist before using other `self` fields |
| `E0499` | `bgm.rs:236` | `self.next_variants[slot]` mutably borrowed while `self` already mutably borrowed for playlists | same clone-based separation |

### 7. Targeted BGM tests before vs after

| State | Test count | Notes |
|---|---:|---|
| seq 2 handoff | 11 | helper/path/one-shot assertions only |
| after first implementation | 15 | +4 runtime-policy tests |
| final | 15 | same count; all pass |

### 8. Full lib test count before vs after

| State | Count |
|---|---:|
| review-only pass earlier in thread | 182 passing |
| after implementation | 186 passing |

### 9. Targeted BGM test result

```text
running 15 tests
...
test survivor::bgm::tests::one_shot_bgm_does_not_auto_advance_after_finish ... ok
test survivor::bgm::tests::repeating_bgm_does_not_restart_while_still_playing ... ok
test survivor::bgm::tests::repeating_bgm_requeues_next_variant_after_finish ... ok
test survivor::bgm::tests::single_variant_repeating_bgm_restarts_same_track_after_finish ... ok
test result: ok. 15 passed; 0 failed; 171 filtered out
```

### 10. Full lib test result

```text
test result: ok. 186 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.28s
```

### 11. Release build result

```text
Finished `release` profile [optimized] target(s) in 3.91s
```

### 12. Release-only cleanup iteration

| Iteration | Result |
|---|---|
| first release build after functional fix | success + `dead_code` warning on `bgm_playlist_advances` |
| second release build after `#[cfg(test)]` | success, warning gone |

### 13. Important code anchors

| Symbol / area | Line(s) |
|---|---:|
| `bgm_restarts_after_finish` | 71-73 |
| `bgm_repeat_flag` | 80-82 |
| `BgmAction` | 84-88 |
| `choose_bgm_action` | 90-113 |
| `play_bgm_file` | 115-117 |
| `BgmSystem::choose_action` | 230-240 |
| steady-state rollover branch | 278-290 |
| key-switch branch | 293-303 |
| new regression tests | 440-506 |

### 14. Audio behavior doc consistency

| Source | Statement | Final implementation status |
|---|---|---|
| `docs/audio_assets.md` | repeating variant BGM should continue into next variant after natural end | now consistent |
| `docs/audio_assets.md` | StageClear/GameOver remain one-shot cues | still consistent |
| `AGENTS.md` current sprite/audio notes | audio assets should stay release-safe and scoped | consistent |

## Code Analysis

- `bgm_repeats(key)` still expresses semantic category only: whether the mode conceptually belongs to the repeating class. It is no longer the same thing as rodio repeat behavior.
- `bgm_restarts_after_finish(key)` is intentionally a semantic bridge: “after this track ends, should the game schedule another track automatically?”
- `bgm_repeat_flag(_key)` now encodes an engine-facing constraint, not a design-facing rule. Because all BGM channels must surface a finished state, the flag is always `false`.
- `BgmAction` is a lightweight policy boundary that made it possible to unit-test runtime behavior without mocking `AudioManager`.
- `choose_bgm_action(...)` is deterministic given `current`, `target`, `playlist`, `next_variant`, and `finished`. This is the right seam for future edge-case tests.
- The implementation preserves the `next_variants` “1 → 2 → 1” schedule by incrementing only when a new path is actually selected.
- The steady-state branch still updates volume every frame regardless of whether a new track is started.
- Empty-playlist handling remains outside the helper in `BgmSystem::run`, so the helper can stay focused on policy rather than asset/log behavior.
- Cloning the slot playlist in `BgmSystem::choose_action` is the main trade-off introduced by this refactor. Given only five channels and tiny per-playlist cardinality, the cost is negligible compared with correctness and borrow simplicity.
- The new tests prove behavior at the policy seam, not at the actual `AudioManager` sink level. This is acceptable under the user-locked “game-side only” constraint, but it means there is still no end-to-end audio device test.

## Files Changed

### Source code
- `crates/game/src/survivor/bgm.rs` — changed repeat policy to manual re-queue, introduced `BgmAction` and `choose_bgm_action`, rewired `BgmSystem` steady-state and key-switch playback, resolved borrow issue with cloned playlist seam

### Tests
- `crates/game/src/survivor/bgm.rs` — added four regression tests covering repeating rollover after finish, no restart during active playback, one-shot no-auto-advance, and single-variant repeating fallback

### Data & results
- no repo-tracked data/result files created in this session

### Config
- no repo config files changed in this session

## User Feedback & Preferences (REQUIRED — never omit)

- Asked first for a code-review-style assessment of recent changes, with a security/safety/regression focus, and explicitly said not to edit code during that review.
- Required the review report to be concise and in Korean.
- After the review, invoked `$grill-me` and requested an “improvement plan,” which means they wanted clarification before implementation rather than an immediate patch.
- During clarification, chose a narrow scope: `BGM 회귀 + 검증 보강만`.
- Locked the intended product behavior: repeating BGM must advance to the next variant after natural finish.
- Rejected engine-scope expansion implicitly by choosing `game-side only`.
- Explicitly preserved prior work by choosing to keep StageClear/GameOver one-shot behavior and current executable-relative path resolution unchanged.
- Required automatic proof stronger than helper tests by choosing “런타임 전이 자동 테스트까지 필요.”
- Selected `lib 테스트 + survivor 빌드만` as the mandatory proof set, making manual QA optional rather than required.
- Resolved the QA contradiction by classifying manual QA as `선택적 보조 확인`, not a completion blocker.
- After planning, said `PLEASE IMPLEMENT THIS PLAN`, which is a direct preference for execution rather than more design discussion.
- Did not request a commit after implementation, so the safe assumption was to leave the diff uncommitted.
- Requested a dedicated handoff at the end via `$handoff`, indicating they want future-session continuity preserved explicitly.

## Where We're Going

1. Run optional manual QA for `title`, `ingame`, and `boss` to confirm actual audible variant rollover in the real app, not just policy-seam tests.
2. If manual QA reveals any mismatch between `AudioManager::is_finished` and real sink behavior, capture that as a new engine-boundary investigation or `docs/ENGINE_CHANGE_REQUESTS.md` prompt rather than patching ad hoc.
3. Decide whether to commit the current `bgm.rs` change alone or bundle it with any pending audio asset work already in the repo history.
4. If this BGM chain is considered done after optional QA, update any stale doc references that still imply repeating keys use rodio loop directly.
5. Resume the broader release-hardening or visual-QA priorities from `CLAUDE.md` / `docs/NEXT_WORK_PLAN.md` once audio confirmation is complete.

## Risks & Blockers

- No end-to-end audio-device QA was run, so the current proof is strong at the policy seam but not at the audible application level.
- `bgm_repeat_flag` now always returns `false`; if future contributors reintroduce rodio looping for repeating keys without understanding why, this regression can return.
- The worktree still contains only an uncommitted change; it can be lost or conflated with later audio edits if not committed or handed off clearly.
- The helper seam does not prove anything about weird backend-specific sink timing behavior; only actual runtime QA would expose that.

## Open Questions

- Does real runtime playback in `cargo run -p game --bin survivor` audibly roll over from `title1 -> title2`, `ingame1 -> ingame2`, and `boss1 -> boss2` as expected?
- Should this fix be committed immediately as a focused audio patch, or held until optional manual QA is done?
- Are there any docs besides `docs/audio_assets.md` that still describe repeating-key implementation in a way that implies direct rodio looping?

## Quick Start for Next Session

```bash
# Reference docs
sed -n '1,220p' AGENTS.md
sed -n '1,220p' docs/audio_assets.md
sed -n '1,260p' plans/handoffs/HANDOFF_bgm-path-hardening_repeat-flag-fix_2026-05-31.md

# Key files to read first (not exhaustive — explore adjacent code too)
sed -n '1,520p' crates/game/src/survivor/bgm.rs
sed -n '1,220p' docs/audio_assets.md
sed -n '1,220p' docs/ENGINE_CHANGE_REQUESTS.md

# Evidence / data files
plans/handoffs/HANDOFF_bgm-path-hardening_repeat-flag-fix_2026-05-31.md
plans/handoffs/HANDOFF_bgm-path-hardening_variant-rollover-regression_2026-06-01.md

# Verify current state
cargo test -p game --lib --locked -- --test-threads=1
cargo build -p game --bin survivor --release --locked

# Next action
# Run optional manual audio QA to confirm real title/ingame/boss variant rollover behavior.
cargo run -p game --bin survivor
```
