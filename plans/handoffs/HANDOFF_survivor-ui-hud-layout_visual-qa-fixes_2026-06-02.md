# Survivor UI visual QA fixes confirmed and ready to commit

**Date:** 2026-06-02
**Status:** COMPLETED
**Bead(s):** none
**Epic:** Survivor visual QA / responsive UI
**Chain:** `survivor-ui-hud-layout` seq `3`
**Parent:** `plans/handoffs/HANDOFF_survivor-ui-hud-layout_visual-qa-replay_2026-06-01.md`
**Prior chain:** `HANDOFF_survivor-ui-hud-layout_2026-06-01.md` -> `HANDOFF_survivor-ui-hud-layout_visual-qa-replay_2026-06-01.md` -> this handoff

---

## Related Handoffs

- `plans/handoffs/HANDOFF_survivor-ui-hud-layout_2026-06-01.md` — seq 1, shared layout model and no-distortion policy.
- `plans/handoffs/HANDOFF_survivor-ui-hud-layout_visual-qa-replay_2026-06-01.md` — seq 2, initial replay of the visual QA plan, compact menu issues, and remaining visual checks.
- `plans/handoffs/HANDOFF_bgm-path-hardening_variant-rollover-regression_2026-06-01.md` — separate audio chain, related only as nearby repo history.

## Since Last Handoff

- The seq 2 handoff said the UI visual QA plan was only partially complete.
- The remaining visual checks were completed through a mix of user screenshots and direct reruns.
- F5/F6 debug pickup sprites were confirmed broken by user screenshots, then fixed and user-confirmed.
- Initial Title screen world-sprite leakage was confirmed by a user screenshot, then fixed and user-confirmed.
- The manual QA record was expanded from the prior partial state to include both new issues and their fixes.
- Automated validation moved from 186 passing lib tests in seq 2 to 188 passing lib tests after the new regression tests.
- The chain is now ready for commit/push unless the user asks for more visual fixes.

## Reference Documents

- `AGENTS.md` — repo rules: main target `crates/game`, do not edit external `skeleton-engine`, use `apply_patch`, update visual QA notes for visual changes.
- `docs/manual_qa_checklist.md` — primary QA record for this pass; now includes the 2026-06-01 UI Visual QA Replay section with final fixes.
- `crates/game/src/survivor/debug_input.rs` — F5/F6 debug pickup implementation and regression test.
- `crates/game/src/survivor/meta.rs` — Title mode cleanup and compact achievements pagination tests.
- `crates/game/src/survivor/hud.rs` — compact modal/header/help text placement changes.
- `crates/game/src/survivor/ui_layout.rs` — shared modal/menu/shop layout math.

## The Goal

The user wanted the Rust Survivors UI visual QA pass finished enough to catch and fix obvious regressions from the recent image/layout work.
The most important regressions were visible at 800x600: compact modal overlap, debug pickup sprites appearing as gray/white blocks, and initial Title showing the player sprite behind the Start button.
The end state requested by the user was practical: fix the visible bugs, verify with tests/build, run the game for user confirmation, then create a handoff, commit, and push.
No engine checkout edits were allowed or needed.

## Where We Are

- Workspace: `/Users/jkl/Projects/rust-survivors`.
- Branch at handoff creation: `main`.
- Remote: `origin https://github.com/ChunSam/rust-survivors.git`.
- Current workstream: `survivor-ui-hud-layout`.
- Parent chain exists and is direct continuation context.
- The latest user confirmation is: `수정 확인 했어`.
- The user also confirmed F5/F6 sprite fix: `이미지 적용 정상확인`.
- The current code changes are game-side only except `Cargo.lock` and docs.
- No external `skeleton-engine` checkout was edited.
- No `docs/ENGINE_CHANGE_REQUESTS.md` entry was needed.
- No asset PNG/MP3/WAV was added or modified by this pass.
- `Cargo.lock` changed `skeleton-engine` package version from `1.0.0` to `1.1.0` because the local path dependency currently resolves to version `1.1.0`.
- `crates/game/src/survivor/debug_input.rs` now calls the shared `spawn_pickup` path for F5/F6 debug pickups.
- `debug_input.rs::debug_pickups_use_atlas_sprites` verifies Bomb/Rosary debug pickups have texture sprites and `UvRect`.
- `crates/game/src/survivor/meta.rs` now cleans leftover player entities when `SurvivorMode::Title` is processed.
- `meta.rs::title_mode_cleans_initial_player_entity` verifies the initial setup player is removed in Title mode and `GameState` becomes `Paused`.
- `meta.rs` also keeps compact achievements pagination at 8 items per page.
- `crates/game/src/survivor/hud.rs` contains compact CharacterSelect header/gold positioning fixes.
- `hud.rs` uses `layout.footer_baseline` for menu help text rather than the modal inner footer baseline.
- `hud.rs` uses a smaller Settings row step of `44.0` so Resolution/help text avoids lower frame decoration.
- `crates/game/src/survivor/ui_layout.rs` contains the shared menu footer and compact list-bottom safety changes.
- `ui_layout.rs::ShopLayout::visible_rows` allows 7 rows at compact height.
- `ui_layout.rs::ShopLayout::footer_pos` places compact Shop help below the modal frame without colliding with the bottom ornament.
- `docs/manual_qa_checklist.md` records the UI QA pass, fixes, and latest validation count.
- The game was re-run after the Title cleanup fix so the user could visually confirm.
- The user confirmed the Title player leakage fix after the rerun.
- No staging had happened before this handoff request.

## What We Tried (Chronological)

1. The session began with engine-change inspection and next-priority planning.
   - The local external `skeleton-engine` checkout had changes such as persistent resources and erased resource helpers.
   - Game-side validation passed against the updated engine.
   - No immediate game code change was required for those engine changes.

2. The user asked for the next work and then invoked `$grill-me`.
   - A visual QA plan was created for Rust Survivors UI at `800x600`, `1280x720`, `1600x900`, and `1920x1080`.
   - Scope included Title, Settings, CharacterSelect, StageSelect, Achievements, Shop, InGame HUD, boss bar, Level-up, PauseMenu, GameOver, and debug keys.
   - Scope excluded audio QA, StageClear, long play, engine checkout edits, staging, and commits.

3. The user explicitly said `PLEASE IMPLEMENT THIS PLAN`.
   - The game was run with `cargo run -p game --bin survivor`.
   - `screencapture` and CoreGraphics window-bounds checks were used for manual visual QA.
   - The workflow preserved existing dirty UI changes.

4. Compact menu/modal issues were found and fixed.
   - 800x600 Settings lower help text and Resolution row were too close to the lower modal ornament.
   - 800x600 CharacterSelect title/gold overlapped.
   - 800x600 Achievements title/summary/rows were too dense.
   - 800x600 Shop last visible row and help text collided with the lower frame.
   - Fixes landed in `ui_layout.rs`, `hud.rs`, and `meta.rs`.

5. The first automated validation for the compact menu fixes passed.
   - `cargo fmt --check` passed.
   - `cargo test -p game --lib --locked -- --test-threads=1` passed with 186 tests.
   - `cargo build -p game --bin survivor --release --locked` passed.

6. The user asked which parts they should test.
   - Manual test guidance was given: focus 800x600 UI screens, Shop footer, Settings Resolution row, Level-up cards, InGame slots, boss bar, F5/F6, and high-res Settings/Title.

7. The user asked to run the game.
   - `cargo run -p game --bin survivor` was launched.
   - The app processed some old or rapid input and moved through Shop/Start/debug keys/Settings during one run.
   - This did not block later fixes.

8. The user sent many screenshots and reported that F5/F6 Bomb/Rosary showed gray/white blocks.
   - Screenshot evidence showed F5/F6 generated square placeholders while normal sprites rendered correctly.
   - Code inspection found `debug_input.rs::spawn_pickup_near` directly created `Sprite::colored(...)`.
   - This bypassed `pickup::spawn_pickup`, which maps `PickupKind::Bomb` and `PickupKind::Rosary` to `SurvivorSprite::Bomb` and `SurvivorSprite::Rosary`.

9. F5/F6 debug pickup fix was implemented.
   - `debug_input.rs` import changed from `Pickup` and manual `Sprite`/`Entity` construction to `spawn_pickup`.
   - `spawn_pickup_near` now computes `spawn_pos` and calls `spawn_pickup(world, spawn_pos, kind)`.
   - The debug print remained.
   - A regression test verifies the spawned Bomb/Rosary pickup entities have textured sprites and `UvRect`.

10. F5/F6 fix was validated.
    - `cargo fmt --check` passed.
    - `cargo test -p game --lib --locked -- --test-threads=1` passed with 187 tests.
    - `cargo build -p game --bin survivor --release --locked` passed.
    - The user later confirmed: `이미지 적용 정상확인`.

11. The user then reported Title screen player leakage.
    - User screenshot showed the bottom of the player sprite visible under the Start button.
    - This happened on initial Title after `setup_survivor_world` spawned Player while mode stayed `Title`.

12. Title cleanup fix was implemented.
    - `ModeTransitionSystem` now calls `clean_title_world_if_needed(world)` before title input handling.
    - `clean_title_world_if_needed` checks whether a `Player` entity exists.
    - If yes, it calls `super::death::reset_to_title_world(world)`.
    - That removes run `Transform` entities, pauses game state, and resets camera.

13. Title cleanup regression test was added.
    - `title_mode_cleans_initial_player_entity` creates a world, inserts `GameState::Playing`, runs `setup_survivor_world`, and asserts Player exists.
    - Then it runs `ModeTransitionSystem`.
    - It asserts the Player is gone and `GameState` is `Paused`.

14. Title cleanup validation passed.
    - `cargo fmt --check` passed.
    - `cargo test -p game --lib --locked -- --test-threads=1` passed with 188 tests.
    - `cargo build -p game --bin survivor --release --locked` passed.
    - The game was run again with `cargo run -p game --bin survivor`.
    - The user confirmed the visual fix.

15. The user requested handoff, commit, and push.
    - This file is the handoff requested by `$handoff`.
    - The next actions after writing and validating this file are staging, committing, and pushing.

## Key Decisions

- Keep all fixes inside `crates/game` UI/game code; no engine change was needed.
- Use the existing public pickup helper for debug F5/F6 instead of duplicating entity construction.
- Validate debug pickup sprites through ECS components (`Sprite.texture` and `UvRect`) rather than brittle pixel tests.
- Clean Title mode by removing leftover Player entities, because initial setup creates gameplay state before the first Title frame.
- Keep `reset_to_title_world` as the cleanup primitive instead of inventing a separate partial cleanup path.
- Do not hide the Title leakage by moving the Start button or changing z order; the real issue was stale world state in Title.
- Do not add or regenerate assets for F5/F6; the correct Bomb/Rosary atlas mappings already existed.
- Keep compact Achievements at 8 items per page, because 10 rows are too dense at 800x600.
- Record user screenshot findings in `docs/manual_qa_checklist.md` because visual fixes need durable QA notes.
- Commit the existing seq 2 handoff as a parent context file even though it was untracked before this request.

## Evidence & Data

### Validation Summary

| Stage | Command | Result |
|---|---|---|
| Compact menu fixes | `cargo fmt --check` | passed |
| Compact menu fixes | `cargo test -p game --lib --locked -- --test-threads=1` | 186 passed |
| Compact menu fixes | `cargo build -p game --bin survivor --release --locked` | passed |
| F5/F6 sprite fix | `cargo fmt --check` | passed |
| F5/F6 sprite fix | `cargo test -p game --lib --locked -- --test-threads=1` | 187 passed |
| F5/F6 sprite fix | `cargo build -p game --bin survivor --release --locked` | passed |
| Title cleanup fix | `cargo fmt --check` | passed |
| Title cleanup fix | `cargo test -p game --lib --locked -- --test-threads=1` | 188 passed |
| Title cleanup fix | `cargo build -p game --bin survivor --release --locked` | passed |

### Test Count Progression

| Point | Count | Reason |
|---|---:|---|
| Before F5/F6 fix | 186 | Existing UI visual QA regression coverage |
| After F5/F6 fix | 187 | Added `debug_pickups_use_atlas_sprites` |
| After Title cleanup fix | 188 | Added `title_mode_cleans_initial_player_entity` |

### User Visual Reports

| User report | Evidence | Fix |
|---|---|---|
| F5/F6 Bomb/Rosary render as gray/white blocks | User screenshots at 2026-06-02 around 00:01-00:03 | Route debug pickup creation through `spawn_pickup` |
| F5/F6 image application normal | User text: `이미지 적용 정상확인` | Confirmed fixed |
| Initial Title shows player behind Start | User screenshot `스크린샷 2026-06-02 오전 12.24.31.png` | Clean Player in Title mode |
| Title leakage fixed | User text: `수정 확인 했어` | Confirmed fixed |

### Current Git State Before Commit

| Item | Value |
|---|---|
| Branch | `main` |
| Tracked modified files | `Cargo.lock`, `debug_input.rs`, `hud.rs`, `meta.rs`, `ui_layout.rs`, `manual_qa_checklist.md` |
| Untracked handoff | `plans/handoffs/HANDOFF_survivor-ui-hud-layout_visual-qa-replay_2026-06-01.md` |
| New handoff | `plans/handoffs/HANDOFF_survivor-ui-hud-layout_visual-qa-fixes_2026-06-02.md` |
| Diff stat before new handoff | `6 files changed, 144 insertions(+), 44 deletions(-)` |

### Recent Commit Log Before This Commit

| Hash | Summary |
|---|---|
| `fc8869b` | Fix BGM variant rollover regression |
| `f8f5b16` | Stabilize survivor UI layout and assets |
| `5d7c224` | session: repeat-flag-fix [bgm-path-hardening] |
| `cec4478` | Fix stageclear/gameover BGM looping forever instead of playing once |
| `cfb087a` | Harden survivor BGM asset resolution |
| `4944312` | Polish survivor text UI assets |
| `9e9223a` | Preserve survivor UI and sprite image aspects |
| `b074062` | Document game states and assets |
| `45edbef` | Add architecture overview docs |
| `984cba1` | Refactor survivor combat and UI flow |

### Commands Useful for Reproduction

```bash
cargo run -p game --bin survivor
cargo fmt --check
cargo test -p game --lib --locked -- --test-threads=1
cargo build -p game --bin survivor --release --locked
git status --short
```

### Screenshot / Runtime Details

| Detail | Value |
|---|---|
| Main visual stress resolution | `800x600` |
| Default restored resolution in QA | `1280x720` |
| macOS titlebar affected capture crop | yes |
| Earlier observed 800x600 window bounds | `X=560`, `Y=135`, `Width=800`, `Height=632` |
| Useful content top for that capture | `Y + 32 = 167` |
| Non-blocking warnings | `GB18030Bitmap` font warning, egui framebuffer warning |

## Code Analysis

- `setup_survivor_world(world)` always calls `spawn_player(world)` before inserting `SurvivorMode::Title` if mode is absent.
- Because `TitleVisualSystem` draws title images but the renderer also draws world sprites, the initial Player could leak visually behind title buttons.
- `reset_to_title_world(world)` already existed for menu return and correctly despawns `Transform` entities, resets `GameStats`, removes pending level-up, resets boss/camera resources, sets `GameState::Paused`, and resets camera.
- `ModeTransitionSystem` is registered before most gameplay/render support systems in `src/bin/survivor.rs`, making it the right place to clean the state before other systems use it.
- `debug_input.rs::spawn_pickup_near` previously duplicated `pickup::spawn_pickup` logic but used `Sprite::colored`, so it missed `SurvivorSprite::Bomb`, `SurvivorSprite::Rosary`, `RenderLayer`, `UvRect`, and `AnimationPlayer`.
- `pickup::spawn_pickup` maps all pickup kinds to atlas sprites and applies `add_sprite`, so it should remain the single source for pickup visuals.
- `world.get::<Sprite>(entity).unwrap().texture.is_some()` is the minimum assertion that the pickup is not a plain colored square.
- `world.get::<UvRect>(entity).is_some()` confirms the atlas-crop path is present.
- `MenuListLayout::footer_baseline` separates help text from panel-internal row layout, reducing repeated ad hoc footer fixes in `hud.rs`.
- `ACHIEVEMENTS_PER_PAGE = 8` is a visual readability decision, not a data model limit.

## Files Changed

### Source code

- `Cargo.lock` — updated `skeleton-engine` package version from `1.0.0` to `1.1.0` as resolved by the current dependency state.
- `crates/game/src/survivor/debug_input.rs` — F5/F6 debug pickups now use `spawn_pickup`; added regression test for textured atlas pickup sprites.
- `crates/game/src/survivor/hud.rs` — compact menu/header/footer layout changes; Settings row spacing adjusted.
- `crates/game/src/survivor/meta.rs` — Achievements page size set to 8; Title mode cleans leftover Player entity; added regression test.
- `crates/game/src/survivor/ui_layout.rs` — `MenuListLayout.footer_baseline`, compact row safety, compact Shop row count/footer positioning.

### Docs

- `docs/manual_qa_checklist.md` — new 2026-06-01 UI Visual QA Replay section records compact menu fixes, F5/F6 sprite fix, Title cleanup fix, and validation commands.
- `plans/handoffs/HANDOFF_survivor-ui-hud-layout_visual-qa-replay_2026-06-01.md` — prior seq 2 handoff, already present but untracked before this request.
- `plans/handoffs/HANDOFF_survivor-ui-hud-layout_visual-qa-fixes_2026-06-02.md` — this handoff, seq 3.

### Tests

- `crates/game/src/survivor/debug_input.rs::debug_pickups_use_atlas_sprites`.
- `crates/game/src/survivor/meta.rs::title_mode_cleans_initial_player_entity`.
- Existing compact achievements test updated to expect 3 pages with 8/8/4 items.
- Existing layout tests in `ui_layout.rs` now assert menu footer/last-row safety.

### Generated artifacts

- No screenshots were added to the repo.
- No new image/audio assets were added.
- Local `target/` build artifacts were produced by cargo but are not part of the commit.

## User Feedback & Preferences

- User wants direct implementation when they provide `PLEASE IMPLEMENT THIS PLAN`.
- User wants Korean responses.
- User asked to preserve current dirty UI work and not roll it back.
- User cares strongly about visual QA from real macOS windows, not only tests.
- User wanted specific user-test instructions before manual testing.
- User confirmed F5/F6 image application works after the fix.
- User reported visual issues via screenshots and expects those screenshots to drive fixes.
- User noticed and reported the initial Title player leakage immediately after the F5/F6 fix.
- User confirmed the Title leakage fix.
- User now explicitly requested `$handoff`, commit, and push.
- User did not ask for staging-only; they asked for commit and push.
- User previously excluded engine checkout edits unless requested.
- User prefers concise status updates while work is ongoing.

## Where We're Going

1. Commit this completed UI visual QA fix set.
2. Push `main` to `origin`.
3. Next session can resume broader visual QA only if the user reports more screenshots or asks for additional pass.
4. Remaining broader QA outside this completed patch: audio listening, StageClear, longer play, and macOS package verification if future assets/package scripts change.
5. If another UI visual issue appears, inspect whether it is shared layout math (`ui_layout.rs`) before applying per-screen constants.

## Risks & Blockers

- `Cargo.lock` now records `skeleton-engine` version `1.1.0`; make sure this is acceptable for the remote environment.
- The existing seq 2 handoff was untracked before this request; commit should include it if preserving continuity is desired.
- The current branch is `main`; pushing directly to `main` matches the user's request but bypasses a PR workflow.
- `ps` required escalation in this environment; GUI process checks may need approval in future sessions.
- Manual QA screenshots are not stored in repo by design; the durable evidence is the checklist and tests.

## Open Questions

- None for the completed F5/F6 and initial Title leakage fixes.
- Broader future visual QA may still find StageClear or audio-related issues because those were out of scope for this fix set.

## Quick Start for Next Session

```bash
# Restore context
sed -n '1,360p' plans/handoffs/HANDOFF_survivor-ui-hud-layout_visual-qa-fixes_2026-06-02.md

# Parent context
sed -n '1,320p' plans/handoffs/HANDOFF_survivor-ui-hud-layout_visual-qa-replay_2026-06-01.md

# Reference docs
sed -n '1,220p' AGENTS.md
sed -n '1,80p' docs/manual_qa_checklist.md

# Key files to read first
sed -n '1,190p' crates/game/src/survivor/debug_input.rs
sed -n '730,890p' crates/game/src/survivor/meta.rs
sed -n '200,620p' crates/game/src/survivor/ui_layout.rs
sed -n '560,1110p' crates/game/src/survivor/hud.rs

# Verify current state
cargo fmt --check
cargo test -p game --lib --locked -- --test-threads=1
cargo build -p game --bin survivor --release --locked

# Next action
Only continue UI visual QA if the user provides a new screenshot or asks for another pass; otherwise this chain is complete.
```

## Handoff Self-Check

- Chain is valid: `survivor-ui-hud-layout`, seq `3`.
- Parent exists: `plans/handoffs/HANDOFF_survivor-ui-hud-layout_visual-qa-replay_2026-06-01.md`.
- Prior chain lists seq 1, seq 2, then this file.
- Where We Are names real files and current state.
- What We Tried includes compact menu fixes, F5/F6 block fix, and Title leakage fix.
- Evidence & Data includes exact commands and test counts: 186, 187, 188.
- Code Analysis names concrete functions and why each changed.
- Files Changed names source, docs, tests, and generated artifacts.
- User Feedback & Preferences is present.
- Quick Start has a concrete first action.
- No stale identifier section is needed.
