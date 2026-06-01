# Survivor UI visual QA replay and compact menu fixes

**Date:** 2026-06-01
**Status:** IN PROGRESS
**Bead(s):** none
**Epic:** Survivor visual QA / responsive UI
**Chain:** `survivor-ui-hud-layout` seq `2`
**Parent:** `plans/handoffs/HANDOFF_survivor-ui-hud-layout_2026-06-01.md`
**Prior chain:** `HANDOFF_survivor-ui-hud-layout_2026-06-01.md` -> this handoff

---

## Related Handoffs

- `plans/handoffs/HANDOFF_survivor-ui-hud-layout_2026-06-01.md`
- This is the direct parent for the current work.
- It established the shared UI layout model.
- It also established the no-distortion image policy.
- It reported that the Shop and other modal screens still needed direct visual QA.
- The current session is a replay of that visual QA plan with actual macOS screenshots.
- The current session narrows the active scope to UI visual regressions.
- The active focus is 800x600 first, then 1280x720 spot checks.
- The active focus is not audio listening.
- The active focus is not long StageClear playthrough.
- The active focus is not engine API redesign.
- `plans/handoffs/HANDOFF_bgm-path-hardening_variant-rollover-regression_2026-06-01.md`
- This is a separate audio chain.
- It is relevant only as background repository state.
- Do not mix the UI QA changes with that audio chain unless the user asks.

## Reference Documents

- `AGENTS.md`
- Main work target is `crates/game`.
- Do not edit the external `skeleton-engine` checkout.
- If an engine change is needed, add a request to `docs/ENGINE_CHANGE_REQUESTS.md`.
- Preserve the platformer demo binary.
- Use `rg` first for searches.
- Use `apply_patch` for manual edits.
- Do not revert unrelated dirty worktree changes.
- For visual changes, run tests/build and update manual QA notes.
- The user is a Rust beginner; explain borrow checker or lifetime workarounds briefly if they become relevant.
- `docs/manual_qa_checklist.md`
- This session added a new `2026-06-01 UI Visual QA Replay` section.
- That section is now the primary manual QA record for this pass.
- It lists found issues, fixes, verification, and remaining manual checks.
- `docs/NEXT_WORK_PLAN.md`
- Keep this as background only.
- The current requested plan supersedes broad backlog items for this session.
- `docs/ENGINE_CHANGE_REQUESTS.md`
- No engine request was added in this session.
- No engine-side blocker was identified while fixing the menu layout issues.
- `crates/game/src/survivor/ui_layout.rs`
- Current source of truth for modal/menu/HUD layout math.
- `crates/game/src/survivor/hud.rs`
- Current source of truth for text placement and modal HUD rendering.
- `crates/game/src/survivor/meta.rs`
- Current source of truth for achievements page size.

## The Goal

The user asked to implement a fixed UI-centered visual QA plan for Rust Survivors.

The plan scope is:

- Title
- CharacterSelect
- StageSelect
- Achievements
- Settings
- Shop
- InGame HUD
- Level-up
- PauseMenu
- GameOver

The primary target is:

- Detect recent image UI/HUD regressions.
- Fix text overlap.
- Fix icon overlap.
- Fix frame overlap.
- Fix clipped UI.
- Catch white rectangle leakage.
- Catch sprite direction or size regressions.
- Check whether button images and hit areas feel aligned.
- Prioritize 800x600 readability.
- If 800x600 is acceptable, do a shorter 1280x720 confirmation.

The execution prerequisite was:

- macOS screen must be unlocked.
- `screencapture` must capture the actual game window.
- If only the lock screen is captured, stop and report environment blockage.

The current session did not fully finish the whole plan.

The menu/modal part was investigated and fixed.

The remaining work is:

- InGame HUD visual QA.
- Boss bar visual QA.
- Level-up visual QA.
- PauseMenu visual QA.
- GameOver visual QA.
- Debug pickup/effect sprite visual QA with `F5`, `F6`, and boss spawn.
- A final visual re-capture of 1280x720 Settings after the last non-compact fix.

## Where We Are

- Workspace: `/Users/jkl/Projects/rust-survivors`.
- Branch: `main`.
- Current date: 2026-06-01.
- Current task status: partial completion with code fixes and validation done.
- Current dirty files are limited to four tracked files.
- Dirty file: `crates/game/src/survivor/hud.rs`.
- Dirty file: `crates/game/src/survivor/meta.rs`.
- Dirty file: `crates/game/src/survivor/ui_layout.rs`.
- Dirty file: `docs/manual_qa_checklist.md`.
- Git diff stat at handoff time:
- `4 files changed, 75 insertions(+), 24 deletions(-)`.
- No assets were edited in this session.
- No package scripts were edited in this session.
- No engine files were edited in this session.
- No external `skeleton-engine` checkout was edited.
- No engine change request was needed.
- The running `survivor` app was killed with `pkill -x survivor`.
- No known long-running command session remains active.
- Earlier GUI process output showed only non-blocking warnings.
- Warning: `failed to load font 'GB18030Bitmap'`.
- Warning: `Detected a linear (sRGBA aware) framebuffer Bgra8UnormSrgb...`.
- These warnings did not block QA.

The current code changes do the following:

- `MenuListLayout` now has a `footer_baseline`.
- `MenuListLayout` separates list row safe area from footer/help text baseline.
- Compact menus use a higher bottom safety margin above the modal panel ornament.
- Non-compact menus also use a higher bottom safety margin than before.
- Compact row and font lower bounds are slightly lower to fit 800x600.
- Help text now sits below the modal frame where possible.
- `ShopLayout::visible_rows` allows 7 rows at 800x600.
- CharacterSelect compact header separates title and gold.
- CharacterSelect compact title/gold font sizes are reduced.
- Achievements compact header is smaller and positioned lower/left.
- Achievements page size changed from 10 to 8.
- Menu help text now uses `layout.footer_baseline` instead of `layout.modal.footer_baseline`.
- Manual QA notes were updated with what was found and what remains.

Visual confirmations already completed:

- 800x600 Title was visible.
- 800x600 Settings was visually rechecked after fixes.
- 800x600 CharacterSelect was visually rechecked after fixes.
- 800x600 Achievements was visually rechecked after fixes.
- 800x600 Shop was visually rechecked after fixes.
- 1280x720 Title was visually rechecked after returning to default resolution.

Visual confirmations not fully completed:

- 1280x720 Settings after the final non-compact fix was not re-captured.
- StageSelect was not specifically re-captured after the final fix.
- InGame HUD was not completed after the menu fixes.
- Level-up was not completed after the menu fixes.
- PauseMenu was not completed after the menu fixes.
- GameOver was not completed after the menu fixes.
- Boss bar and debug sprite paths were not completed after the menu fixes.

## What We Tried (Chronological)

1. The user first asked whether direct visual QA was possible.
- The answer was effectively yes, if the macOS screen is unlocked and screenshots capture the game window.
- The user then asked to clarify the exact checks and plan.
- A UI-centered replay plan was created.
- The user then explicitly asked: `PLEASE IMPLEMENT THIS PLAN`.

2. The session started with repository and context checks.
- `AGENTS.md` was provided in the conversation.
- The plan was treated as the active scope.
- The implementation stayed inside game UI code and docs.
- The engine boundary was preserved.

3. The app was run as a development binary.
- Command used: `cargo run -p game --bin survivor`.
- This required macOS GUI access.
- The app opened as a real window.
- A full screenshot preflight confirmed the actual desktop/game context, not the lock screen.
- This satisfied the plan's screenshot prerequisite.

4. The initial saved resolution was not the target.
- The app initially came up using a larger saved resolution.
- Settings were navigated through the UI.
- The resolution was changed to 800x600 through Settings UI.
- The save file was not directly edited.
- This followed the user plan's constraint.

5. Screenshot cropping initially had a false start.
- Early crop used `-R0,30,800,600`.
- That included macOS titlebar area and cropped some game content.
- The issue was recognized during visual inspection.
- CoreGraphics window bounds were then used to find the actual window geometry.
- For 800x600, the useful content crop was based on the titlebar-adjusted window content.
- Example observed 800x600 bounds:
- `X=560`.
- `Y=135`.
- `Width=800`.
- `Height=632`.
- Content top was `Y + 32 = 167`.
- This correction matters for future QA.

6. 800x600 Settings showed clear overlap.
- The lower help text overlapped or touched the lower modal ornament.
- The Resolution row also hit the lower decoration area.
- This made the compact modal unsafe at 800x600.

7. 800x600 CharacterSelect showed header and footer issues.
- The screen title and Gold label overlapped.
- The lower row and help text hit the modal ornament.
- The issue was not just text length.
- It was caused by shared menu layout baselines being too close to the lower frame.

8. 800x600 Achievements showed severe compact layout issues.
- Title overlapped summary text.
- Summary text overlapped the first achievement row.
- The 10th achievement row hit the lower modal ornament.
- This suggested that 10 achievements per page is too dense for 800x600.

9. 800x600 Shop showed a row count issue.
- The last visible row hit the lower frame.
- Gold and power-up rows were otherwise the intended content.
- No achievements summary was visible in Shop, which matches the plan.

10. The first fix targeted shared menu layout.
- `MenuListLayout::new` was changed.
- It now detects compact viewports.
- Compact viewports use a more conservative `list_bottom`.
- Compact viewports allow smaller row/font lower bounds.
- `footer_baseline` was added separately from modal content/footer math.
- Help text call sites were moved to the new footer baseline.

11. CharacterSelect needed a targeted compact header fix.
- The shared list fix did not solve the title/gold header overlap alone.
- `draw_character_select_hud` now detects compact viewports.
- Compact title size is smaller.
- Compact title x offset is smaller.
- Compact Gold x offset is smaller.
- Compact Gold y is lower.
- Compact Gold font size is smaller.

12. Achievements needed more than shared layout.
- The shared layout fix still left achievements too dense.
- Compact title and stats were moved/sized down.
- Achievements page size was changed from 10 to 8.
- The existing tests were updated to lock the new page count.
- With 20 achievements, pages are now 8, 8, and 4 items.

13. Shop needed a compact row count adjustment.
- `ShopLayout::visible_rows` had a minimum of 8 rows.
- At 800x600 that was too many for the framed panel.
- The minimum is now 7 rows when viewport height is 620 or less.
- Higher viewports still keep the previous minimum of 8.

14. Automated validation was run after code fixes.
- `cargo fmt` was run after edits.
- `cargo fmt --check` passed during the validation sequence.
- `cargo test -p game --lib --locked -- --test-threads=1` passed.
- The test result was `186 passed`.
- `cargo build -p game --bin survivor --release --locked` passed.

15. The 800x600 menu screens were visually rechecked.
- Settings no longer had the obvious lower help/frame overlap.
- CharacterSelect no longer had the obvious title/gold overlap.
- Achievements used 8 rows and no longer hit the lower frame.
- Shop used 7 visible rows and no longer hit the lower frame.
- These results were recorded in `docs/manual_qa_checklist.md`.

16. 1280x720 was partially rechecked.
- Resolution was changed back to the default 1280x720 through Settings/UI navigation.
- Title was visually rechecked and looked normal.
- Settings at 1280x720 showed the same class of lower ornament conflict.
- A follow-up non-compact `MenuListLayout` fix was made.
- Tests/build were rerun after that fix.
- A final visual capture of 1280x720 Settings after the fix was not completed before this handoff.

17. The app was stopped cleanly for handoff.
- `pkill -x survivor` was used.
- The last `cargo run` session exited after the app was killed.
- No active GUI run is expected.

## Key Decisions

- Keep the QA scope UI-centered.
- Keep audio listening out of this pass.
- Keep long StageClear out of this pass.
- Use the development binary first: `cargo run -p game --bin survivor`.
- Do not directly edit the save file.
- Change resolution through Settings UI when possible.
- Use whole-screen preflight before trusting crops.
- Use CoreGraphics window bounds for exact content crops.
- Treat the macOS titlebar height as part of the window bounds but not part of game content.
- Fix shared modal/menu math in `ui_layout.rs` when multiple screens show the same issue.
- Use targeted text placement fixes in `hud.rs` only where the screen has unique header pressure.
- Change Achievements pagination because the 10-row page was visually too dense at 800x600.
- Keep Shop content to Gold and power-up list only.
- Do not reintroduce stretched image assets.
- Do not generate new images for this fix.
- Do not touch the engine.
- Keep this as the same `survivor-ui-hud-layout` chain because it follows directly from the prior shared layout pass.

## Evidence & Data

### Commands Run

| Command | Purpose | Result |
|---|---|---|
| `cargo run -p game --bin survivor` | Start real game window for QA | succeeded |
| `osascript -e ...` | Navigate menus and send key input | succeeded |
| `swift -e ... CoreGraphics...` | Read window bounds/window ids | succeeded |
| `screencapture -x ...` | Capture full screen and window/content crops | succeeded |
| `cargo fmt` | Format after edits | succeeded |
| `cargo fmt --check` | Verify formatting | passed |
| `cargo test -p game --lib --locked -- --test-threads=1` | Game library regression tests | passed, 186 tests |
| `cargo build -p game --bin survivor --release --locked` | Release survivor build | passed |
| `pkill -x survivor` | Stop GUI app | succeeded |

### Screenshot Notes

- Screenshots were stored under `/private/tmp`.
- The exact screenshot filenames are not required as source artifacts.
- Useful observed final screenshots included:
- `/private/tmp/rust_survivors_qa_final_achievements_800.png`.
- `/private/tmp/rust_survivors_qa_final_shop_800.png`.
- `/private/tmp/rust_survivors_qa_final_title_1280.png`.
- There were also intermediate Settings/Character screenshots.
- Some early screenshots used an incorrect crop and should not be treated as final evidence.
- The reliable method is CoreGraphics bounds plus titlebar offset.

### Window Geometry Notes

- 800x600 observed window bounds included titlebar.
- Observed 800x600 bounds:
- `X=560`.
- `Y=135`.
- `Width=800`.
- `Height=632`.
- Content capture should start around `Y=167`.
- 1280x720 observed bounds also included titlebar.
- Observed 1280x720 examples:
- `Width=1280`.
- `Height=752`.
- Content top should be `Y + 32`.
- If the window moves, recompute bounds rather than reusing coordinates.

### Found Issues

| Screen | Resolution | Issue | Status |
|---|---:|---|---|
| Settings | 800x600 | Help text and Resolution row hit lower ornament | visually fixed |
| CharacterSelect | 800x600 | Title and Gold overlapped | visually fixed |
| CharacterSelect | 800x600 | Lower row/help hit lower ornament | visually fixed |
| Achievements | 800x600 | Title/stats/first row overlapped | visually fixed |
| Achievements | 800x600 | 10th row hit lower ornament | fixed by 8-item pages |
| Shop | 800x600 | Last visible row hit lower frame | visually fixed |
| Settings | 1280x720 | Resolution row/help hit lower ornament | code/test fixed, visual recapture pending |

### Validation Output

- `cargo fmt --check`: passed.
- `cargo test -p game --lib --locked -- --test-threads=1`: passed.
- Test count: `186 passed`.
- `cargo build -p game --bin survivor --release --locked`: passed.
- Package scripts were not run because no assets or package files changed.

### Recent Commits at Handoff Time

| Hash | Summary |
|---|---|
| `fc8869b` | `Fix BGM variant rollover regression` |
| `f8f5b16` | `Stabilize survivor UI layout and assets` |
| `5d7c224` | `session: repeat-flag-fix [bgm-path-hardening]` |
| `cec4478` | `Fix stageclear/gameover BGM looping forever instead of playing once` |
| `cfb087a` | `Harden survivor BGM asset resolution` |

## Code Analysis

### `crates/game/src/survivor/ui_layout.rs`

- `MenuListLayout` now has:
- `modal`.
- `row_count`.
- `row_step`.
- `row_h`.
- `font_size`.
- `first_row_y`.
- `footer_baseline`.
- The new `footer_baseline` is intentionally not the same as `modal.footer_baseline`.
- The goal is to keep menu rows out of the panel ornament while placing help text below the panel frame.
- Compact detection is:
- `viewport_w <= 900.0 || viewport_h <= 640.0`.
- Compact `list_bottom` is:
- `modal.panel.bottom() - 116.0`.
- Non-compact `list_bottom` is:
- `modal.panel.bottom() - 84.0`.
- Compact `row_step` lower bound is `20.0`.
- Non-compact `row_step` lower bound is `24.0`.
- Compact `font_size` lower bound is `13.0`.
- Non-compact `font_size` lower bound is `14.0`.
- `footer_baseline` is:
- `Vec2::new(modal.content.x, (modal.panel.bottom() + 22.0).min(viewport_h - 20.0))`.
- `ShopLayout::visible_rows` now uses:
- minimum 7 rows when `viewport_h <= 620.0`.
- minimum 8 rows otherwise.
- The common layout test now asserts:
- footer baseline remains inside the viewport.
- last menu row bottom remains above `modal.panel.bottom() - 40.0`.

### `crates/game/src/survivor/hud.rs`

- CharacterSelect uses compact-specific title and gold placement.
- Compact title font is `30.0`.
- Non-compact title font remains `38.0`.
- Compact title x offset is `170.0`.
- Non-compact title x offset remains `210.0`.
- Compact Gold x offset is `128.0`.
- Non-compact Gold x offset remains `168.0`.
- Compact Gold y offset is `30.0`.
- Non-compact Gold y offset remains `7.0`.
- Compact Gold font is `20.0`.
- Non-compact Gold font remains `24.0`.
- Help text now uses `layout.footer_baseline` in:
- CharacterSelect.
- StageSelect.
- Achievements.
- PauseMenu.
- Settings.
- Achievements compact title uses:
- `Vec2::new(layout.modal.content.x + 28.0, layout.modal.panel.y + 48.0)`.
- Achievements compact title font is `26.0`.
- Achievements compact stats uses:
- `Vec2::new(layout.modal.content.x + 28.0, layout.modal.panel.y + 76.0)`.
- Achievements compact stats font is `14.0` when `vh <= 620.0`.

### `crates/game/src/survivor/meta.rs`

- `ACHIEVEMENTS_PER_PAGE` changed from `10` to `8`.
- The test was renamed to `achievements_are_paged_for_compact_readability`.
- The test now expects:
- `achievement_page_count() == 3`.
- page 0 has 8 items.
- page 1 has 8 items.
- page 2 has 4 items.
- page 3 has 0 items.
- This is a behavior change.
- It is intentional for 800x600 readability.
- If the product expectation is to show more rows on large screens, a future pass should make page size viewport-aware.
- Current code keeps a single global page size for simplicity.

### `docs/manual_qa_checklist.md`

- A new top section records the current pass.
- It lists scope.
- It lists commands.
- It lists found issues.
- It lists fixes.
- It lists verified checks.
- It lists remaining manual checks.
- It explicitly states that InGame HUD, Level-up, PauseMenu, GameOver, boss/debug pickup sprite remain for the next UI QA pass.

## Files Changed

### Source Files

- `crates/game/src/survivor/ui_layout.rs`
- Added `footer_baseline` to `MenuListLayout`.
- Adjusted compact/non-compact list bottom safety.
- Adjusted compact row and font minimums.
- Adjusted compact Shop visible row count.
- Added layout assertions.

- `crates/game/src/survivor/hud.rs`
- Adjusted compact CharacterSelect header layout.
- Moved menu help text calls to `layout.footer_baseline`.
- Adjusted compact Achievements title/stats placement and sizes.

- `crates/game/src/survivor/meta.rs`
- Changed Achievements pagination from 10 to 8 per page.
- Updated achievement pagination unit test.

### Documentation

- `docs/manual_qa_checklist.md`
- Added current visual QA replay notes.
- Added discovered issues.
- Added fixes and validation results.
- Added explicit remaining manual QA items.

### Not Changed

- No asset files changed.
- No packaged app scripts changed.
- No engine dependency changed.
- No `Cargo.toml` or lockfile changed in this session.
- No platformer demo code changed.

## User Feedback & Preferences

- User asked whether direct visual QA is possible.
- User asked to make the parts to check clear and plan before proceeding.
- User then provided and approved the concrete plan by saying `PLEASE IMPLEMENT THIS PLAN`.
- User wants actual execution, not only analysis.
- User wants UI-centered QA for this pass.
- User wants 800x600 readability prioritized.
- User wants visible regressions caught in the real window.
- User does not want save-file shortcuts when the plan says to use Settings UI.
- User previously objected to intentional image aspect distortion.
- Preserve image aspect; do not stretch PNG UI art.
- Generate new image assets only if necessary for a new correct aspect, and document them.
- User is likely comfortable with Korean status updates.
- Keep explanations concise but concrete.
- If Rust borrow checker or lifetime details become relevant, explain them briefly and plainly.

## Where We're Going

The next session should continue the same user plan, not restart planning.

First priority:

- Re-run the app.
- Confirm 1280x720 Settings visually after the final non-compact `MenuListLayout` fix.
- Capture using CoreGraphics content coordinates.
- Update `docs/manual_qa_checklist.md` if it is visually confirmed.

Second priority:

- Set or confirm 800x600 again through Settings UI.
- Start a game.
- Verify InGame HUD at 800x600.
- Check top HP/XP/Level/Time/Kills.
- Check bottom weapon/passive slots.
- Check that no icon/text/frame overlap is visible.

Third priority:

- Press boss spawn debug key.
- The user plan said `B`.
- AGENTS lists `F7` boss spawn.
- Current code/session behavior should be verified from source before relying on memory.
- Check boss HP bar against the top HUD.
- Check no background blank area is exposed.

Fourth priority:

- Press `L` to force Level-up if supported.
- Verify 3 cards.
- Verify icons.
- Verify text.
- Verify title image orientation.
- Verify card internals stay inside the card rows.

Fifth priority:

- Press `F5` for Bomb.
- Press `F6` for Rosary.
- Verify pickups/effects are not upside down.
- Verify effects are not overlarge.
- Verify Whip/effect/actor sprites still read correctly.

Sixth priority:

- Press `ESC` for PauseMenu.
- Verify pause modal rows and help text.
- Drive to GameOver path if feasible.
- Verify GameOver modal and title image.

Seventh priority:

- Return to or confirm 1280x720.
- Short-check Title, InGame, and Level-up at 1280x720.

## Risks & Blockers

- macOS screenshot capture can silently capture the wrong Space or lock screen.
- Always do a full-screen preflight when starting a new GUI QA run.
- CoreGraphics window bounds include the titlebar.
- Cropping without titlebar adjustment can create false visual issues.
- `System Events` keypresses may fail if focus changes or Accessibility permissions shift.
- If keys stop working, use Computer Use or manual UI clicks as fallback.
- The game may load a saved state with an unexpected resolution or language.
- The plan says to use Settings UI, not direct save edits.
- Achievements page size is now globally 8.
- This fixes compact readability but may be less dense than desired at large resolutions.
- 1280x720 Settings was fixed in code and tests, but final screenshot evidence is pending.
- StageSelect likely benefits from the shared layout fix, but was not explicitly re-captured in this session.
- InGame HUD and Level-up remain the largest unverified areas.
- `B` vs `F7` boss spawn control is inconsistent between the user plan and AGENTS.
- Source should be checked before the next run.
- Package scripts were not run because no assets/package files changed.
- If the next pass changes assets, run macOS package and verify scripts.

## Open Questions

- Should Achievements page size remain a global 8, or should it become viewport-dependent?
- Should non-compact Settings help text live below the panel frame permanently, or should the art/layout be redesigned?
- Is the 7-row compact Shop acceptable, or should it scroll with a visible scrollbar/page indicator?
- Should StageSelect get its own compact header adjustment, or is shared layout enough?
- Which debug key is authoritative for boss spawn in the current code: `B`, `F7`, or both?
- Should final QA restore the user's preferred saved settings after testing?

## Quick Start for Next Session

Start by checking the exact dirty state:

```bash
git status --short
git diff --stat
sed -n '1,40p' docs/manual_qa_checklist.md
```

Confirm the code still validates:

```bash
cargo fmt --check
cargo test -p game --lib --locked -- --test-threads=1
cargo build -p game --bin survivor --release --locked
```

Then run the app:

```bash
cargo run -p game --bin survivor
```

Before relying on screenshots:

```bash
screencapture -x /private/tmp/rust_survivors_preflight.png
```

Find the window bounds:

```bash
swift -e 'import CoreGraphics; if let list = CGWindowListCopyWindowInfo([.optionOnScreenOnly], kCGNullWindowID) as? [[String: Any]] { for w in list { if String(describing: w[kCGWindowName as String] ?? "").lowercased().contains("survivor") || String(describing: w[kCGWindowOwnerName as String] ?? "").lowercased().contains("survivor") { print(w) } } }'
```

Use the returned bounds to crop content.

For an 800x600 window with observed titlebar-inclusive height 632:

```bash
screencapture -x -R560,167,800,600 /private/tmp/rust_survivors_content_800.png
```

Do not reuse the sample crop if the window moved.

Next concrete action:

- Re-run visual QA at 1280x720 Settings after the final non-compact fix.
- If confirmed, update `docs/manual_qa_checklist.md` from "code/test fixed" to "visually confirmed".
- Then continue the unfinished 800x600 InGame HUD, boss, Level-up, PauseMenu, GameOver, and debug sprite checks.

## Suggested QA Checklist Continuation

- Preflight full screenshot shows desktop/game, not lock screen.
- Title at 1280x720: logo/buttons normal.
- Settings at 1280x720: Resolution row clear of lower ornament.
- Settings at 1280x720: help text clear of lower ornament.
- StageSelect at 1280x720: rows clear of frame.
- Set 800x600 through Settings UI.
- Title at 800x600: no runtime text leakage.
- CharacterSelect at 800x600: title and Gold separated.
- StageSelect at 800x600: rows and help clear.
- Achievements at 800x600: 8 rows per page and no overlap.
- Shop at 800x600: Gold and power-up list only.
- Shop at 800x600: 7 visible rows and no lower frame overlap.
- InGame at 800x600: top HUD readable.
- InGame at 800x600: bottom slots do not overlap.
- Boss spawn at 800x600: boss bar below top HUD.
- Boss spawn at 800x600: no blank background edge.
- Level-up at 800x600: title image upright.
- Level-up at 800x600: three cards fit.
- Level-up at 800x600: icons and text stay inside cards.
- Bomb debug at 800x600: effect visible and not overlarge.
- Rosary debug at 800x600: effect visible and not upside down.
- PauseMenu at 800x600: panel rows and help text fit.
- GameOver at 800x600: title/panel/help text fit.
- Return to 1280x720.
- InGame at 1280x720: short HUD check.
- Level-up at 1280x720: short card check.

## Handoff Self-Check

- This handoff names the parent chain.
- This handoff records the exact scope of the user plan.
- This handoff separates completed and remaining QA.
- This handoff lists dirty files.
- This handoff lists commands and validation results.
- This handoff records visual evidence and crop caveats.
- This handoff records code behavior changes.
- This handoff records user preferences.
- This handoff does not claim InGame/Level-up QA is complete.
- This handoff does not claim 1280x720 Settings was visually confirmed after the final fix.
- This handoff gives a concrete next action.
