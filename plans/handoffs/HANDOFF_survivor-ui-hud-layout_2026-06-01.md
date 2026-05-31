# Survivor UI/HUD layout stabilization and no-distortion image policy

**Date:** 2026-06-01
**Status:** IN PROGRESS
**Bead(s):** none
**Epic:** Survivor visual QA / responsive UI
**Chain:** `survivor-ui-hud-layout` seq `1`
**Parent:** none — first in chain
**Prior chain:** none — first in chain

---

## Related Handoffs

- `plans/handoffs/HANDOFF_bgm-path-hardening_executable-audio-paths_2026-05-31.md` — separate BGM executable-relative path hardening chain.
- `plans/handoffs/HANDOFF_bgm-path-hardening_repeat-flag-fix_2026-05-31.md` — separate follow-up fixing StageClear/GameOver one-shot BGM repeat behavior.
- `plans/handoffs/HANDOFF_boss-clear-trigger_2026-05-31.md` — separate gameplay bugfix for boss death triggering StageClear.

## Reference Documents

- `AGENTS.md` — repository-specific agent rules; main target is `crates/game`, do not edit engine checkout directly, use `apply_patch`, keep visual QA docs updated.
- `CLAUDE.md` — shorter active context mirror; note that it still contains some older `/Volumes/SSD/...` path text, while current workspace is `/Users/jkl/Projects/rust-survivors`.
- `docs/NEXT_WORK_PLAN.md` — current visual/UI and release-hardening priorities; latest listed test count there is stale relative to this session.
- `docs/manual_qa_checklist.md` — manual QA record; this session appended/relies on 2026-05-31 UI layout notes, but further visual QA is still needed.
- `docs/ASSET_LICENSES.md` — asset provenance and hashes; required if future work adds regenerated UI frame assets.

## The Goal

The user wants the Survivor UI/HUD to stop drifting out of alignment across images, text, icons, selection highlights, and hit/click areas. The requested structure is a common screen-space layout model where each screen derives visual rects, text baselines, icon centers, and interaction/hit rects from shared parent rects. The user also corrected the implementation: do not intentionally stretch images to fit arbitrary UI rects, because that visibly smears the art. If an image aspect does not fit the intended UI shape, either keep the original aspect and fit it, use a non-image highlight, or generate/create a new correctly proportioned asset and document it.

## Where We Are

- Active repo is `/Users/jkl/Projects/rust-survivors` on branch `main`.
- Current git worktree is dirty with many pre-existing and session changes; do not assume every dirty file belongs to this UI layout task.
- The newest user correction is: “이미지를 의도적으로 비율을 수정한 부분이 보여. 이러면 이미지가 뭉개지는 문제가 있어. 비율 수정하지 말고 필요하면 이미지 생성 기능을 이용해서 새로운 이미지를 만들어서 고쳐줘”.
- `crates/game/src/survivor/ui_layout.rs` is newly added and exported from `crates/game/src/survivor/mod.rs`.
- `ui_layout.rs` contains `ScreenRect`, `UiLayout`, `ModalLayout`, `MenuListLayout`, `TopHudLayout`, `HudSlotLayout`, `LevelUpLayout`, `ShopLayout`, and title image rect helpers.
- `ScreenRect` is now the shared model for `x/y/w/h`, `right`, `bottom`, `center`, `inflated`, `inset`, `aspect_fit`, containment, and anchor-like parent-derived rects.
- `responsive_ui_scale` moved into `ui_layout.rs` and is used by HUD, icon, level-up, and shop layout code.
- `ModalLayout::centered` now uses `MODAL_PANEL_ASPECT = 1478.0 / 756.0` so modal panel rects match `ui_modal_panel.png` instead of stretching the panel art.
- `TopHudLayout` derives top in-game HP bar, HP value, stat row label/value rects, panel rect, bottom, and boss bar y by `HudDetail`.
- `HudSlotLayout` derives bottom equipment panel, 6x2 slot rects, skin rects, stripe rects, icon centers, and text x from one layout result.
- `LevelUpLayout` derives level-up panel, card rows, icon centers, icon size, text x, text baseline, and font size.
- `ShopLayout` derives shop modal panel, content rect, visible row count, visible start, title/gold positions, row selection rect, icon center, text x, text pos, text width, and footer pos.
- `title_logo_rect`, `title_start_image_rect`, and `title_menu_button_image_rect` keep title image centers aligned with interaction/hit rect centers.
- `crates/game/src/survivor/hud.rs` now reads most screen UI coordinates from layout structs instead of repeating ad hoc constants.
- `hud.rs::queue_ui_texture` uses source aspect via `survivor_texture_aspect(path)` + `ScreenRect::aspect_fit`; no fill/stretch helper remains.
- `hud.rs::queue_modal_panel` and `hud.rs::queue_slot_frame` preserve source image aspects and draw dark backing only inside the aspect-fitted image rect.
- `hud.rs` tests include `queued_ui_texture_preserves_source_aspect` and `panel_and_slot_frame_preserve_source_aspects`.
- In-game top HUD was reorganized around `TopHudLayout`.
- In-game bottom weapon/passive slots now derive frame, stripe, icon, and level text from `HudSlotLayout`.
- CharacterSelect, StageSelect, Achievements, Settings, PauseMenu, StageClear, GameOver were moved toward `ModalLayout`/`MenuListLayout` safe content positioning.
- Several menu selection row frames were intentionally changed from stretched `ui_slot_frame.png` to colored highlight rects to avoid wrong-aspect image stretching.
- `crates/game/src/survivor/ui_icons.rs` now reads HUD/Level-up/Shop icon coordinates from `HudSlotLayout`, `LevelUpLayout`, and `ShopLayout`.
- `ui_icons.rs::queue_screen_texture` again preserves source aspect with `survivor_texture_aspect` + `aspect_fit`.
- `ui_icons.rs::queue_shop_icons` no longer draws a long `ui_slot_frame.png` over shop row selection; it draws the colored row highlight and icons.
- `ui_icons.rs::queue_levelup_backgrounds` no longer draws `ui_slot_frame.png` for long card rows; it draws a colored card rect, modal panel, and icons.
- Shop now displays only the visible window of power-ups according to `ShopLayout::visible_rows()` instead of letting 19 rows overflow the panel.
- Shop title/gold were moved above the modal panel border after visual smoke showed the title overlapping the top frame decoration.
- The latest packaged app opened from `/Users/jkl/Projects/rust-survivors/dist/macos/RustSurvivors.app`.
- Visual smoke after the latest package: Title looks correct; clicking Shop opens a non-stretched modal panel with visible rows and no elongated slot-frame row art.
- The Shop screen still deserves manual visual QA because the top title is above the panel and may need final art direction.
- No new generated image assets were created during the no-distortion correction because the immediate distortion could be removed by avoiding mismatched frame art.
- If future design requires ornate long row frames, a new correctly proportioned row-frame asset should be generated or manually created, not stretched from `ui_slot_frame.png`.

## What We Tried (Chronological)

1. The user first asked for a plan to stabilize HUD coordinates, image/text alignment, and class inheritance-like structure.
   - Rust has no class inheritance, so the plan chose composition via `ScreenRect`, `UiLayout`, and per-screen layout structs.
   - User invoked `$grill-me`, requested a plan, then explicitly asked to implement it.

2. Initial implementation added `ui_layout.rs` and replaced some duplicated layout math.
   - Goal: common source-of-truth rects for InGame HUD, Level-up, Shop, title image/hit rects, and later modal screens.
   - Validation at that time reached passing tests/build, but visual screenshots showed many screens still misaligned.

3. A packaging/run issue caused stale visuals.
   - The release binary had been built but the user was launching the macOS app bundle.
   - Rebuilding `target/release/survivor` alone was not enough; `bash scripts/package_macos.sh` was needed to copy the new binary/assets into `dist/macos/RustSurvivors.app`.
   - After repackaging and opening the app, remaining layout issues were real, not just stale app state.

4. The first visual correction overused `fill`.
   - `queue_ui_texture_fill` and `queue_screen_texture` were changed to fill source-of-truth rects exactly.
   - This made image/text coordinates align more predictably but stretched `ui_modal_panel.png` and `ui_slot_frame.png`.
   - Tests were temporarily changed to assert “fills source-of-truth rects”.
   - User rejected this: stretched images looked smudged/distorted.

5. The no-distortion correction reverted the image policy.
   - `hud.rs::queue_ui_texture` and `ui_icons.rs::queue_screen_texture` now fit images inside requested rects while preserving source aspect.
   - The fill helper was removed.
   - Tests now assert image aspect is preserved and the fitted image stays inside the requested bounds.

6. Modal layout was adjusted to the panel asset instead of arbitrary panel aspect.
   - `MODAL_PANEL_ASPECT = 1478.0 / 756.0` was added.
   - `ModalLayout::centered` creates a panel rect aspect-fitted inside max bounds using the same panel aspect, so panels can be both layout source and image-size compatible.
   - This reduces letterboxing/shrink mismatch for major modal panels.

7. Long row frame use was removed where the source art did not match.
   - `ui_slot_frame.png` aspect is about `2125 / 473 = 4.492`.
   - Shop row/content spans are much wider than that, and Level-up card skin rects were also not close enough.
   - Instead of stretching, Shop selection and Level-up card rows use colored rects.
   - This preserves the user’s no-distortion requirement without waiting on a new asset.

8. Shop overflow was fixed by visible-window logic.
   - Before this pass, all 19 `PowerUpKind::ALL` rows could extend past the panel bottom.
   - `ShopLayout::visible_rows()` currently clamps to 8..13 rows based on content height.
   - `ShopLayout::visible_start(cursor_index, total_rows)` keeps the selected row visible as the cursor moves.
   - Text and icon loops in `hud.rs`/`ui_icons.rs` use the same visible window.

9. Visual smoke caught Shop title overlap after no-distortion changes.
   - After opening the app and clicking Shop, `POWERUP SHOP` overlapped the top frame decoration.
   - `ShopLayout::title_pos()` and `gold_pos()` were moved above the panel using `(panel.y - 38).max(24)` and `(panel.y - 32).max(30)`.
   - Latest app smoke showed the title/gold above the panel and no stretched long row frame.

10. Verification and packaging were rerun after the latest changes.
    - `cargo fmt --check` passed.
    - `cargo test -p game --lib --locked -- --test-threads=1` passed: 182 tests.
    - `cargo build -p game --bin survivor --release --locked` passed.
    - `bash scripts/package_macos.sh` passed and verified packaged assets.
    - App was relaunched via `open dist/macos/RustSurvivors.app`.

## Key Decisions

- Use layout composition, not inheritance. Rust has no class inheritance, and the repository style favors structs/functions over class-like abstractions.
- Keep “row/content/interaction rect is source of truth” for coordinates, but do not force image pixels to fill that rect if the aspect does not match.
- Preserve image aspect at every texture queue boundary. `aspect_fit` is acceptable; arbitrary fill/stretch is not.
- Use colored highlight rects for long selection/card rows when the existing `ui_slot_frame.png` is the wrong aspect.
- Do not generate new assets unless the UI truly needs an ornate row frame with a new aspect. Current fix removed the need for immediate image generation.
- Keep engine APIs unchanged. All work stayed in game/UI code; no `skeleton-engine` edits were made.
- Keep `UiImageQueue`, `UiQueue`, and `TextQueue` separate. The change is layout and coordinate derivation, not renderer API redesign.
- For title UI, preserve hit/interaction semantics and align image rect centers to existing hit rect centers.
- For Shop, prioritize no overlap and visible rows over showing all 19 power-ups simultaneously.
- Repackage before GUI validation. Running the app bundle without `scripts/package_macos.sh` can show stale code.

## Evidence & Data

### Validation commands run in this session

| Command | Result |
|---|---|
| `cargo fmt --check` | passed |
| `cargo test -p game --lib --locked -- --test-threads=1` | passed, 182 tests |
| `cargo build -p game --bin survivor --release --locked` | passed |
| `bash scripts/package_macos.sh` | passed, package manifest verification included |
| `open dist/macos/RustSurvivors.app` | launched packaged app |

### Latest visual smoke

| Screen | Result |
|---|---|
| Title | Opened packaged app; logo/backdrop/buttons visible |
| Shop | Clicked Shop; modal panel visible, power-up rows visible, no stretched `ui_slot_frame` over long selected row |
| Shop after title adjustment | `POWERUP SHOP` and `Gold: 100` moved above panel, no longer on top of the border decoration |

### Asset aspect constraints

| Asset | Known source size/aspect | Current policy |
|---|---:|---|
| `ui_modal_panel.png` | `1478x756`, aspect `1.955` | `ModalLayout::centered` uses this aspect; queue uses aspect fit |
| `ui_slot_frame.png` | `2125x473`, aspect `4.492` | Used only where fitting preserves aspect; not stretched for long Shop/Level-up rows |
| Title button/logo PNGs | transparent PNGs, language variants | Existing title image rect helpers preserve image/hit rect centers |

### Dirty worktree at handoff time

| Category | Paths / notes |
|---|---|
| Current branch | `main` |
| Source modified by this UI work | `crates/game/src/survivor/hud.rs`, `crates/game/src/survivor/ui_icons.rs`, `crates/game/src/survivor/title_visual.rs`, `crates/game/src/survivor/meta.rs`, `crates/game/src/survivor/mod.rs`, new `crates/game/src/survivor/ui_layout.rs` |
| Docs touched in nearby work | `docs/ASSET_LICENSES.md`, `docs/PHASE_LOG.md`, `docs/manual_qa_checklist.md`, `docs/ARCHITECTURE.md` |
| Other dirty files likely unrelated/pre-existing | `Cargo.lock`, audio MP3/WAV changes, `crates/game/src/survivor/bgm.rs`, `crates/game/src/survivor/boss.rs`, handoff files, UI title/font prompt docs |
| Deleted legacy/generated assets in status | old `assets/audio/bgm_*.wav`, old menu button PNGs, `title_logo_plaque.png`, `survivor_atlas_source.png`, `title_backdrop.png` |
| Untracked assets | `assets/audio/rustsurvivors *.mp3`, `docs/UI_FONT_STYLE_PROMPT.md`, `docs/UI_TITLE_STYLE_PROMPT.md`, new handoffs |

### Recent commit log

| Hash | Summary |
|---|---|
| `5d7c224` | `session: repeat-flag-fix [bgm-path-hardening]` |
| `cec4478` | `Fix stageclear/gameover BGM looping forever instead of playing once` |
| `cfb087a` | `Harden survivor BGM asset resolution` |
| `4944312` | `Polish survivor text UI assets` |
| `9e9223a` | `Preserve survivor UI and sprite image aspects` |
| `b074062` | `Document game states and assets` |
| `45edbef` | `Add architecture overview docs` |
| `984cba1` | `Refactor survivor combat and UI flow` |
| `4169c29` | `Modernize survivor engine integration` |
| `fdf641c` | `fix: update survivors for skeleton-engine package` |

### Test count progression observed in docs/session

| Point | Count |
|---|---:|
| AGENTS baseline note | 118 lib tests |
| Earlier UI layout docs in manual QA | 181 passed |
| Latest no-distortion correction | 182 passed |

### Commands and app paths

```bash
cargo fmt --check
cargo test -p game --lib --locked -- --test-threads=1
cargo build -p game --bin survivor --release --locked
bash scripts/package_macos.sh
open dist/macos/RustSurvivors.app
```

Packaged app path:

```text
/Users/jkl/Projects/rust-survivors/dist/macos/RustSurvivors.app
```

## Code Analysis

- `ScreenRect::aspect_fit(aspect)` is now the core anti-distortion primitive. It reduces width or height while keeping center/containment inside the requested rect.
- `hud.rs::queue_ui_texture(world, x, y, w, h, path, z)` wraps `ScreenRect::aspect_fit` and queues `DrawImage::textured_with_handle` using `UvRect::FULL`.
- `hud.rs::queue_modal_panel` performs aspect fit before drawing both backing and texture, so the backing does not show a stretched dark rectangle outside the fitted panel.
- `hud.rs::queue_slot_frame` mirrors modal behavior for `ui_slot_frame.png`.
- `ui_icons.rs::queue_screen_texture(world, rect, path, z)` also aspect-fits before pushing backing + texture.
- `ShopLayout::visible_rows()` computes rows from safe content height and clamps to 8..13 rows.
- `ShopLayout::visible_start(cursor_index, total_rows)` scrolls the visible window so a selected cursor near the bottom remains in view.
- `TopHudLayout` centralizes boss bar y by `panel.bottom()` + padding to avoid top HUD overlap.
- `LevelUpLayout::card_rect()` is still a fixed-size card row around screen center; only the card background rendering changed from slot-frame image to colored rect.
- `MenuListLayout::selection_rect()` and `ShopLayout::selection_skin_rect()` are currently `#[cfg(test)]` where only tests need them, avoiding release dead-code warnings.

## Files Changed

### Source code

- `crates/game/src/survivor/ui_layout.rs` — new shared layout module with screen rect primitives and screen-specific layout structs.
- `crates/game/src/survivor/mod.rs` — exports/includes the new `ui_layout` module.
- `crates/game/src/survivor/hud.rs` — moved HUD/menu/modal coordinates to layout structs; restored image aspect preservation; added layout/aspect regression tests.
- `crates/game/src/survivor/ui_icons.rs` — moved HUD/Level-up/Shop icon/background coordinates to layout structs; restored image aspect preservation; removed stretched long row frame use.
- `crates/game/src/survivor/title_visual.rs` — title image rects align with hit rect centers and preserve image aspect.
- `crates/game/src/survivor/meta.rs` — minor title/hit rect alignment integration.

### Tests

- `hud.rs::queued_ui_texture_preserves_source_aspect`
- `hud.rs::panel_and_slot_frame_preserve_source_aspects`
- `ui_icons.rs::queued_screen_texture_preserves_source_aspect`
- `ui_icons.rs::hud_icons_leave_text_room_at_800x600`
- `ui_icons.rs::levelup_icons_stay_inside_card_text_margin_at_common_resolutions`
- `ui_icons.rs::shop_icons_do_not_overlap_powerup_text_at_common_resolutions`
- `ui_layout.rs::common_layouts_fit_supported_presets`
- `title_visual.rs::title_menu_images_preserve_source_aspects`
- `title_visual.rs::title_image_rects_share_hit_rect_centers`

### Docs

- `docs/manual_qa_checklist.md` — contains 2026-05-31 UI rect stabilization and earlier visual QA notes; may need an additional 2026-06-01 no-distortion note next time.
- `docs/ASSET_LICENSES.md` — dirty from surrounding asset work; update if future generated row-frame assets are added.
- `docs/PHASE_LOG.md` and `docs/ARCHITECTURE.md` — dirty from nearby broader work; review before committing.

### Generated/package artifacts

- `dist/macos/RustSurvivors.app` was regenerated by `bash scripts/package_macos.sh`.
- The package script verified packaged assets and manifest during the run.

## User Feedback & Preferences

- User wants implementation, not just plans, when they provide “PLEASE IMPLEMENT THIS PLAN”.
- User explicitly invoked `$grill-me` earlier to pressure-test plans before implementation.
- User wants `ScreenRect`/layout struct composition as the Rust equivalent of “class inheritance” for shared parent coordinates.
- User’s top priority is coordinate consistency: image rect, text baseline, icon center, hit rect should come from the same row/content source.
- User showed screenshots where HUD text and image frames were offset and said the requested changes were not applied.
- User corrected that a stale packaged app can make it look like changes are not visible; future work must package and run the app bundle when visually validating.
- User said “화면 안보여” during a run attempt; make app launch/status obvious and verify window state if asked to run.
- User rejected intentional image aspect changes because it makes images look smeared.
- User allowed image generation if needed, but the key rule is do not distort existing images.
- User expects screen screenshots to be checked directly and concrete visual issues addressed.
- User is a Rust beginner per AGENTS; explain non-obvious Rust workarounds briefly.
- User is working in Korean; respond in Korean unless code/docs require English.

## Where We're Going

1. Run a focused visual QA pass on the latest packaged app at 1280x720 and 800x600.
2. Check Title, Shop, CharacterSelect, StageSelect, Achievements, Settings, PauseMenu, InGame HUD, Level-up, GameOver, StageClear for text/image overlap.
3. If ornate row frames are still desired for Shop/Level-up/menu selections, create or generate new correctly proportioned row assets instead of stretching `ui_slot_frame.png`.
4. Update `docs/manual_qa_checklist.md` with the 2026-06-01 no-distortion correction and actual manual smoke outcomes.
5. Separate current dirty worktree into logical commits or at least logical review groups before committing anything.
6. Re-run `bash scripts/package_macos.sh` after any asset or app-bundle-relevant change.

## Risks & Blockers

- Dirty worktree contains multiple workstreams: UI layout, BGM/audio assets, boss fix, docs, and asset replacement. Do not commit blindly.
- Visual QA is still incomplete across all requested screens and preset resolutions.
- Existing frame assets are not nine-slice; the engine does not appear to provide a nine-slice UI API. Arbitrary decorative frames will either need aspect-compatible generated assets or non-image highlights.
- `docs/NEXT_WORK_PLAN.md` and `CLAUDE.md` contain stale counts/paths relative to the current session; prefer `AGENTS.md` and current command output.
- Manual GUI testing depends on the app bundle being repackaged; otherwise stale visuals can mislead.

## Open Questions

- Should the final visual direction for Shop/Level-up row selection be plain colored highlight, or should new generated ornate row-frame assets be created for each row aspect?
- Does the user prefer title/gold text outside the Shop panel, or should the panel content be lowered and title kept inside a safe header zone?
- Should `docs/manual_qa_checklist.md` be updated immediately with a 2026-06-01 no-distortion smoke entry, or after a fuller multi-resolution pass?
- Which dirty files belong to the current UI/HUD chain versus prior audio/asset chains before any commit?

## Quick Start for Next Session

```bash
# Restore context
sed -n '1,360p' plans/handoffs/HANDOFF_survivor-ui-hud-layout_2026-06-01.md

# Reference docs
sed -n '1,220p' AGENTS.md
sed -n '1,220p' docs/manual_qa_checklist.md

# Key files to read first
sed -n '1,760p' crates/game/src/survivor/ui_layout.rs
sed -n '180,380p' crates/game/src/survivor/hud.rs
sed -n '120,260p' crates/game/src/survivor/ui_icons.rs
sed -n '1,240p' crates/game/src/survivor/title_visual.rs

# Verify current state
cargo fmt --check
cargo test -p game --lib --locked -- --test-threads=1
cargo build -p game --bin survivor --release --locked
bash scripts/package_macos.sh

# Next action
open dist/macos/RustSurvivors.app
# Then manually inspect Shop, Level-up, Settings, CharacterSelect, StageSelect, Achievements,
# PauseMenu, InGame HUD, GameOver, and StageClear for overlap and aspect distortion.
```
