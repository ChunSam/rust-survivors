# Fix 10-minute boss kill not showing StageClear

**Date:** 2026-05-31
**Status:** COMPLETED
**Bead(s):** none
**Epic:** Rust Survivors gameplay bugfix
**Chain:** `boss-clear-trigger` seq `1`
**Parent:** none — first in chain
**Prior chain:** none — first in chain

---

## Related Handoffs

- `HANDOFF_bgm-path-hardening_executable-audio-paths_2026-05-31.md` — separate release-hardening chain for BGM executable-relative lookup and MP3 playlist behavior.
- `HANDOFF_bgm-path-hardening_repeat-flag-fix_2026-05-31.md` — separate follow-up fixing StageClear/GameOver BGM one-shot repeat behavior; relevant only because StageClear audio exists.

## Reference Documents

- `AGENTS.md` — active project guidance: work in `crates/game`, avoid engine edits, add tests for shared game behavior, run lib tests.
- `CLAUDE.md` — short project context, commands, asset state, key files, and next work.
- `docs/manual_qa_checklist.md` — current manual QA history; still lists long-play StageClear/audio QA as manual.
- `crates/game/src/survivor/README.md` — survivor module overview and roadmap pointer.

## The Goal

The user reported a gameplay bug in Korean: they believed the game should show clear after 10 minutes when the boss is defeated, but killing the boss did not show clear. The target was the `rust-survivors` sibling repo, not the `fontmaker` repo active earlier in the session. The immediate goal was to make the 10-minute boss kill raise the existing `StageProgress.cleared` path so the already-implemented `ModeTransitionSystem` can show StageClear. The fix needed to preserve existing tests that still expect `Death` to clear the stage.

## Where We Are

- Active branch is `main`.
- The actual code change is only in `crates/game/src/survivor/boss.rs`.
- `BossKind::GiantSlime` is the 10:00 boss with `spawn_time() == 600.0` and `max_hp() == 1000.0`.
- Before this session, `BossDeathSystem` only set `StageProgress.cleared = true` for `BossKind::Death`.
- That old behavior meant killing the 10-minute `GiantSlime` despawned the boss and incremented kills, but did not transition to StageClear.
- New method: `BossKind::clears_stage(self) -> bool`.
- `clears_stage()` currently returns `true` for `BossKind::GiantSlime | BossKind::Death`.
- `BossDeathSystem` now calls `kind.clears_stage()` instead of `matches!(kind, BossKind::Death)`.
- Existing `Death` clear compatibility is intentionally preserved because older tests in `crates/game/src/lib.rs` expect it.
- `BossKind::GhostKing` remains non-clear in the new focused test.
- New test `survivor::boss::tests::giant_slime_death_marks_stage_clear` proves 10-minute boss defeat sets `StageProgress.cleared`.
- New test `survivor::boss::tests::non_clear_boss_death_does_not_mark_stage_clear` proves `GhostKing` does not set clear.
- Full lib test suite now has 177 tests and passes.
- `cargo fmt --check` currently fails due to pre-existing formatting differences in `crates/game/src/survivor/bgm.rs`, not because of `boss.rs`.
- `boss.rs` itself was run through `rustfmt`.
- The full dirty worktree contains many unrelated asset/audio/doc changes that pre-existed this bugfix.
- Do not revert or stage unrelated dirty files unless the user explicitly asks.
- There is also separate earlier work in `/Users/jkl/Projects/fontmaker` adding a Swift sheet editor; this handoff is for the later `rust-survivors` gameplay bug.

## What We Tried (Chronological)

1. **Started in the wrong repo context and verified scope.**
   - The user described a game bug while the shell cwd was `/Users/jkl/Projects/fontmaker`.
   - Ran `rg` in `fontmaker` for `boss|clear|win|victory|10|600|survivor|game`.
   - Result: no relevant game logic in `fontmaker`; found sibling `../rust-survivors/AGENTS.md`.
   - Decision: move investigation to `/Users/jkl/Projects/rust-survivors`.

2. **Read project guidance before editing.**
   - Opened `/Users/jkl/Projects/rust-survivors/AGENTS.md`.
   - Key constraints observed:
     - main work target is `crates/game`;
     - do not edit deleted legacy `crates/engine`;
     - do not directly edit external `skeleton-engine`;
     - add tests when changing shared game behavior;
     - run `cargo test -p game --lib --locked -- --test-threads=1`.

3. **Searched for StageClear and boss flow.**
   - Ran `rg -n "StageProgress|cleared|SurvivorMode::StageClear|StageClear|GameState::Paused|best_time|unlocked_stages|SelectedStage" crates/game/src/survivor`.
   - Important findings:
     - `StageProgress` lives in `boss.rs`;
     - `ModeTransitionSystem` in `meta.rs` transitions `SurvivorMode::InGame` to `SurvivorMode::StageClear` when `StageProgress.cleared` is true;
     - `BossDeathSystem` in `boss.rs` was the only boss kill site setting `StageProgress.cleared`.

4. **Read `boss.rs` and confirmed the root cause.**
   - `BossKind::GiantSlime` comment: `10:00 등장. HP 1000.`
   - `BossKind::Death` comment: `30:00 등장. HP 6000. 처치 시 StageClear.`
   - In `BossDeathSystem`, clear logic was:
     ```rust
     if matches!(kind, BossKind::Death) {
         progress.cleared = true;
     }
     ```
   - This directly contradicted the user's expectation that the 10-minute boss should clear.

5. **Initial fix made only `GiantSlime` clear.**
   - Added `BossKind::clears_stage()`.
   - First version returned `matches!(self, BossKind::GiantSlime)`.
   - Added focused tests for GiantSlime and GhostKing.
   - Focused boss tests passed.

6. **Full test suite exposed existing `Death` compatibility.**
   - Ran full lib tests.
   - Failures:
     - `tests::death_boss_defeat_triggers_stage_clear`
     - `tests::lethal_enemy_damage_leaves_boss_for_boss_death_system`
   - Both tests in `crates/game/src/lib.rs` expected `Death` to still trigger StageClear.
   - This was not a test problem; it documented existing game semantics.

7. **Adjusted fix to preserve `Death` while adding `GiantSlime`.**
   - Changed `clears_stage()` to:
     ```rust
     matches!(self, BossKind::GiantSlime | BossKind::Death)
     ```
   - Updated comments from “10:00 boss only” to “10:00 boss or Death”.
   - Re-ran full lib tests; all 177 passed.

8. **Handled formatting carefully.**
   - `cargo fmt --check` failed because `bgm.rs` has pre-existing style diffs.
   - Did not run full `cargo fmt`, because it would mutate unrelated `bgm.rs`.
   - Ran `rustfmt crates/game/src/survivor/boss.rs` only.

## Key Decisions

- **Treat `rust-survivors` as the active project for this handoff.** The latest user request and work were a game bug; `fontmaker` work earlier in the session is separate.
- **Do not ask the user whether 10-minute clear is intended.** The user stated the expected behavior clearly enough, and the code already had a clean `StageProgress.cleared` pipeline.
- **Use the existing StageClear path rather than adding a new UI state.** `ModeTransitionSystem` already watches `StageProgress.cleared`; the bug was upstream of that flag.
- **Add a predicate method on `BossKind`.** This centralizes the clear-trigger policy instead of scattering `matches!` logic inside `BossDeathSystem`.
- **Preserve `Death` as a clear trigger.** Full-suite tests proved existing behavior expects `Death` to clear too; changing that would be a behavior regression beyond the user's request.
- **Leave `GhostKing` non-clear.** It is the 20-minute boss and no artifact/user statement indicated it should end the stage.
- **Do not touch unrelated dirty worktree changes.** The repo has many audio/assets/docs modifications and untracked MP3s from prior work.
- **Do not run full `cargo fmt`.** `bgm.rs` has unrelated formatting drift; only `boss.rs` was formatted.

## Evidence & Data

### Boss clear policy after this session

| BossKind | Spawn time | HP | Clears stage now? | Why |
|---|---:|---:|---|---|
| `GiantSlime` | 600.0 sec / 10:00 | 1000.0 | yes | User-reported expected clear boss |
| `GhostKing` | 1200.0 sec / 20:00 | 2500.0 | no | No current requirement; focused test keeps it non-clear |
| `Death` | 1800.0 sec / 30:00 | 6000.0 | yes | Existing tests and comments expected this behavior |

### Commands run and results

| Command | Result |
|---|---|
| `rg -n "boss|clear|win|victory|10|600|survivor|game" .` in `fontmaker` | No game logic; pointed to sibling rust-survivors references |
| `sed -n '1,220p' AGENTS.md` in `rust-survivors` | Confirmed work target and test rules |
| `rg -n "StageProgress|cleared|SurvivorMode::StageClear|StageClear|..." crates/game/src/survivor` | Found `boss.rs` clear flag and `meta.rs` transition |
| `cargo test -p game survivor::boss --lib --locked -- --test-threads=1` | Initially blocked by sandbox target write; rerun escalated and passed 2 tests |
| `cargo test -p game --lib --locked -- --test-threads=1` first full run | 175 passed, 2 failed; failures showed `Death` clear compatibility |
| `cargo test -p game --lib --locked -- --test-threads=1` final full run | 177 passed, 0 failed |
| `cargo fmt --check` | Failed due to unrelated `bgm.rs` formatting diffs |
| `rustfmt crates/game/src/survivor/boss.rs` | Succeeded; scoped formatting only |

### Full-suite failure before preserving `Death`

```text
running 177 tests
...
test tests::death_boss_defeat_triggers_stage_clear ... FAILED
test tests::lethal_enemy_damage_leaves_boss_for_boss_death_system ... FAILED
test result: FAILED. 175 passed; 2 failed
```

The failures printed `BOSS DEFEATED: DEATH`, then asserted that `StageProgress.cleared` should be true. This is why the final predicate includes both `GiantSlime` and `Death`.

### Final focused boss tests

```text
running 2 tests
test survivor::boss::tests::giant_slime_death_marks_stage_clear ... ok
test survivor::boss::tests::non_clear_boss_death_does_not_mark_stage_clear ... ok
test result: ok. 2 passed; 0 failed
```

### Final full lib tests

```text
running 177 tests
...
test result: ok. 177 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.29s
```

### Current dirty worktree snapshot

Only one file in the dirty worktree belongs to this bugfix:

| File | Status | This session? |
|---|---|---|
| `crates/game/src/survivor/boss.rs` | modified | yes |

Unrelated dirty state already present or outside this bugfix includes:

| Area | Examples |
|---|---|
| Audio assets | deleted `assets/audio/bgm_*.wav`, untracked `assets/audio/rustsurvivors *.mp3` |
| UI/menu assets | deleted old menu PNGs, modified levelup/gameover title PNGs |
| Docs | modified `docs/ARCHITECTURE.md`, `docs/ASSET_LICENSES.md`, `docs/PHASE_LOG.md`, `docs/manual_qa_checklist.md` |
| Prior handoff | modified `plans/handoffs/HANDOFF_bgm-path-hardening_repeat-flag-fix_2026-05-31.md` |
| Lockfile | modified `Cargo.lock` |

### Relevant git log

| Hash | Summary |
|---|---|
| `5d7c224` | session: repeat-flag-fix [bgm-path-hardening] |
| `cec4478` | Fix stageclear/gameover BGM looping forever instead of playing once |
| `cfb087a` | Harden survivor BGM asset resolution |
| `4944312` | Polish survivor text UI assets |
| `9e9223a` | Preserve survivor UI and sprite image aspects |

## Code Analysis

- `BossSpawnQueue::default()` still queues `[GiantSlime, GhostKing, Death]`, so the 10-minute boss naturally appears first.
- `BossSpawnSystem` triggers queued bosses when `GameStats.elapsed >= b.spawn_time()`.
- `BossDeathSystem` only runs while `GameState::Playing`.
- `BossDeathSystem` collects dead bosses via `world.query2::<Boss, Health>().filter(|(_, _, h)| h.current <= 0.0)`.
- On boss death, the system:
  - records position;
  - prints `BOSS DEFEATED: ...`;
  - despawns the boss;
  - drops boss pickups at the death position;
  - triggers camera shake;
  - increments `GameStats.kills`;
  - sets `StageProgress.cleared` only if `kind.clears_stage()`.
- `ModeTransitionSystem` in `meta.rs` is responsible for observing `StageProgress.cleared` and changing `SurvivorMode` to `StageClear`.
- `ModeTransitionSystem` also handles meta updates on clear: total gold, total kills, best time, next stage unlock, and save-to-disk.
- Debug input `B` spawns a `GiantSlime` and then sets its HP to 3000. This can be used for manual QA but will require enough damage to kill it.
- The existing long-form tests in `crates/game/src/lib.rs` still cover the `Death` clear path; the new tests in `boss.rs` cover the 10-minute clear path.

## Files Changed

### Source code

- `crates/game/src/survivor/boss.rs` — added `BossKind::clears_stage()` and changed `BossDeathSystem` to use it so `GiantSlime` and `Death` both set `StageProgress.cleared`.

### Tests

- `crates/game/src/survivor/boss.rs` — added `giant_slime_death_marks_stage_clear`.
- `crates/game/src/survivor/boss.rs` — added `non_clear_boss_death_does_not_mark_stage_clear`.

### Data & results

- No result files were intentionally written.
- Cargo target artifacts were written under `target/` during tests.

### Config

- No config files intentionally changed for this bugfix.

## User Feedback & Preferences

- User reported the bug in Korean: “게임이 10분 경과하면 보스 잡고 클리어가 뜨는걸로 알고있는데, 보스를 잡아도 클리어가 안떠”.
- User expected a direct fix, not a plan or clarification loop.
- User’s mental model: 10 minutes elapsed → boss appears → killing that boss should display clear.
- User prefers Korean communication for gameplay bug reports in this repo context.
- User previously asked to implement plans rather than stop at proposals; default to executing scoped fixes when the issue is concrete.
- User explicitly invoked `$handoff` after the bugfix, so write a file rather than only summarizing in chat.
- User did not ask to commit; do not commit automatically.
- User did not ask to clean dirty worktree; preserve unrelated changes.

## Where We're Going

1. Run a manual gameplay smoke: start survivor, reach or debug-spawn `GiantSlime`, kill it, confirm StageClear UI appears.
2. If manual QA is successful, consider updating `docs/manual_qa_checklist.md` with a dated StageClear smoke result.
3. If the intended final design is “any boss clears” rather than only `GiantSlime` and `Death`, change `BossKind::clears_stage()` and update tests explicitly.
4. Decide whether to commit only `crates/game/src/survivor/boss.rs` or bundle it with other gameplay fixes; avoid unrelated asset/audio changes unless approved.
5. Later, resolve unrelated repo-wide `cargo fmt --check` drift in `bgm.rs` in a separate formatting commit if desired.

## Risks & Blockers

- Manual StageClear smoke was not run in a GUI during this handoff session.
- `cargo fmt --check` is red due to unrelated `bgm.rs` formatting, so CI may fail if it enforces full fmt before that is resolved.
- Dirty worktree has many unrelated asset/audio/doc changes; staging carelessly could mix this bugfix with asset pipeline work.
- Debug `B` spawns `GiantSlime` with HP forcibly raised to 3000, so manual smoke via debug input may take longer unless using high damage/debug tools.
- StageClear BGM behavior was recently fixed in a separate chain; if StageClear appears but audio is wrong, read the BGM handoffs first.

## Open Questions

- Should `GhostKing` also clear the stage in the final game design, or is it intentionally a mid-run boss?
- Is `Death` still meant to appear after a `GiantSlime` clear in any mode, or is keeping it as a clear trigger only backward compatibility for tests/future modes?
- Should manual QA docs be updated now, or only after the user confirms the fix in the running game?

## Quick Start for Next Session

```bash
# Restore context
cd /Users/jkl/Projects/rust-survivors
sed -n '1,260p' plans/handoffs/HANDOFF_boss-clear-trigger_2026-05-31.md

# Reference docs
sed -n '1,180p' AGENTS.md
sed -n '1,140p' CLAUDE.md
sed -n '1,220p' docs/manual_qa_checklist.md

# Key files to read first
sed -n '1,420p' crates/game/src/survivor/boss.rs
sed -n '700,875p' crates/game/src/survivor/meta.rs
sed -n '1,120p' crates/game/src/survivor/debug_input.rs

# Verify current state
cargo test -p game survivor::boss --lib --locked -- --test-threads=1
cargo test -p game --lib --locked -- --test-threads=1

# Known fmt caveat
cargo fmt --check
# Expected currently: fails on unrelated crates/game/src/survivor/bgm.rs formatting.

# Next action
# Run manual game smoke and verify killing the 10-minute GiantSlime reaches StageClear.
cargo run -p game --bin survivor --release
```
