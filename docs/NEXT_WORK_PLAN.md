# Rust Survivors Next Work Plan

Updated: 2026-05-29

This is the short agent-facing work plan. Long historical notes belong in `docs/PHASE_LOG.md`.

## Current Baseline

- Repo: `/Users/jkl/Projects/rust-survivors`
- Main work target: `crates/game`
- Engine: local path dependency on `../../../skeleton-engine`
- Engine source repo: `/Users/jkl/Projects/skeleton-engine`
- Current game build resolves `skeleton-engine` from the sibling checkout for immediate engine/game co-development feedback.
- Engine dependency workflow: `docs/ENGINE_DEPENDENCY_WORKFLOW.md`

Validation baseline:

```bash
cargo fmt --check
cargo clippy --all-targets --locked -- -D warnings
cargo test -p game --lib --locked -- --test-threads=1
cargo build -p game --bin survivor --release --locked
```

Run package checks after asset/package changes:

```bash
bash scripts/package_macos.sh
bash scripts/verify_macos_package.sh
```

Latest observed lib test status: 167 passing.

## Latest Logo Text Transparent PNG Pass

- Regenerated the fixed text PNG set in a stronger logo-style direction after sample approval.
- Scope:
  - `assets/textures/survivor/menu/title_logo_plaque_v3.png`
  - `assets/textures/survivor/menu/menu_button_*_{ko,en}.png`
  - `assets/textures/survivor/ui/ingame_label_*.png`
  - `assets/textures/survivor/ui/levelup_title_*.png`
  - `assets/textures/survivor/ui/gameover_title_*.png`
  - `assets/textures/survivor/ui/restart_hint_*.png`
  - `assets/textures/survivor/ui/section_*.png`
- Style baseline: approved `menu_button_start_ko.png` sample with gold metal text, dark cracked stone plaque, simpler gothic frame, and red gem accents.
- Applied alpha masks with `scripts/make_text_ui_transparent.py`.
  - The script keeps the dark stone plaque and visible metal/gem details.
  - It clears only the outer canvas background.
  - It preserves the existing runtime dimensions.
- Removed the Title-only opaque backing rectangles from `TitleVisualSystem`; title logo/buttons now rely on PNG alpha directly.
- Kept `ui_modal_panel.png` / `ui_slot_frame.png` backing behavior because those shared UI frames were not part of this text PNG transparency pass.
- Documentation:
  - Main pass note: `docs/LOGO_TEXT_IMAGE_PASS.md`
  - Asset hashes: `docs/ASSET_LICENSES.md`
  - Title menu details: `docs/TITLE_MENU_IMAGE_UI.md`
  - In-game static text details: `docs/INGAME_STATIC_TEXT_IMAGE_UI.md`
  - Manual QA: `docs/manual_qa_checklist.md`
- Latest validation:

```bash
cargo fmt --check
cargo test -p game --lib --locked -- --test-threads=1
cargo build -p game --bin survivor --release --locked
bash scripts/package_macos.sh
bash scripts/verify_macos_package.sh
```

- Latest visual smoke: opened `dist/macos/RustSurvivors.app` and confirmed Title logo/buttons render over the background without black/backing rectangles.

## Latest InGame Static Text Image UI Pass

- Added language-specific PNG assets for fixed in-game UI text under `assets/textures/survivor/ui/`.
- Replaced top HUD fixed labels `Lv/HP/XP/Gold/Kills/Passives` with image labels.
- Replaced equipment section labels `Weapons/Passives` with image labels.
- Replaced Level-up title, GameOver title, and restart hint with image assets.
- Kept dynamic values in `TextQueue`: time, level number, HP/XP values, gold, kills, passive count, slot `Lv N`, card labels, and result stats.
- Loaded the new assets through `SurvivorTextureHandles` and selected `{ko,en}` paths using `MetaSave::effective_lang()`.
- Documented implementation details in `docs/INGAME_STATIC_TEXT_IMAGE_UI.md`.
- Latest validation: `cargo fmt --check`, `cargo test -p game --lib --locked -- --test-threads=1` (157 passed), `cargo build -p game --bin survivor --release --locked`, `bash scripts/package_macos.sh`, and `bash scripts/verify_macos_package.sh`.

## Latest UV Orientation Migration Pass

- Consumed the sibling `skeleton-engine` checkout where the shared sprite quad now uses top-left UV origin.
- Removed game-side engine-direction Y-flip compensation from survivor crop UVs, icon grid UVs, title backdrop, and full-image UI textures.
- `SpriteFrame::uv()` now uses `UvRect::from_pixels(...)` directly, and `UiIconSystem` icon sheets now use `UvRect::from_grid(...)` directly.
- Full-image `DrawImage` paths now rely on default `UvRect::FULL`.
- Regression tests now assert positive/top-left UV sizes and default full UV for full-image UI queues.
- `rg -n "flipped_y\\(|vertically_flipped_full_uv" crates/game/src` returns no matches.
- Latest validation: `cargo fmt --check`, `cargo clippy --all-targets --locked -- -D warnings`, `cargo test -p game --lib --locked -- --test-threads=1` (151 passed), `cargo build -p game --bin survivor --release --locked`, and `bash scripts/package_macos.sh`.
- Packaged app smoke: Title, InGame HUD/actor, debug boss spawn, GameOver modal, Shop power-up icons, and Level-up cards render upright.
- Added InGame `L` debug input to force the next Level-up screen for repeatable visual QA.
- 800x600 smoke now covers Title, Settings, InGame HUD/slots, and Level-up cards.

## Latest Title Menu Image UI Pass

- Added AI-generated title menu assets under `assets/textures/survivor/menu/`.
- Replaced the Title backdrop path with `title_backdrop_v2.png`.
- Added a text-free logo plaque and Start/Character/Stage/Shop/Settings button art.
- Loaded the new menu textures through `SurvivorTextureHandles`.
- Submitted the title logo/button art through `UiImageQueue` in `TitleVisualSystem`.
- Added opaque dark backing images under title logo/button art after visual QA caught white transparency leakage.
- Kept localized KO/EN menu labels as engine text to avoid generated-image text errors.
- Preserved existing `title_button_layout(...)` and hitbox behavior.
- Documented implementation details in `docs/TITLE_MENU_IMAGE_UI.md`.
- Latest validation: `cargo fmt --check`, `cargo test -p game --lib --locked -- --test-threads=1` (147 passed), `cargo check -p game --all-targets --locked`, `cargo build -p game --bin survivor --release --locked`, `bash scripts/package_macos.sh`, and `bash scripts/verify_macos_package.sh`.

## Latest Image Based UI Pass

- Added AI-generated UI skin assets under `assets/textures/survivor/ui/`.
- Added `ui_modal_panel.png` for modal/menu/result panels and `ui_slot_frame.png` for compact HUD/card/selection frames.
- Loaded the new UI textures through `SurvivorTextureHandles`.
- Replaced InGame top HUD, boss HP frame, equipment slots, level-up card rows, shop selection rows, and major modal panels with image-based frames.
- Added opaque dark backing behind modal/slot frame textures so transparent regions do not render as white blocks on macOS.
- Moved Settings help text below the modal frame after visual QA showed overlap with frame art.
- Reduced equipment slot text from names to icon + `Lv` labels.
- Kept dynamic values and KO/EN localized labels in `TextQueue` to avoid generated-image text errors.
- Documented implementation details in `docs/IMAGE_BASED_UI_PASS.md`.
- Detailed QA/fix summary: `docs/VISUAL_QA_BACKING_FIX.md`.
- Latest validation: `cargo fmt --check`, `cargo test -p game --lib --locked -- --test-threads=1` (147 passed), `cargo check -p game --all-targets --locked`, `cargo build -p game --bin survivor --release --locked`, `bash scripts/package_macos.sh`, and `bash scripts/verify_macos_package.sh`.

## Latest Engine Migration Pass

- Updated the game dependency to `skeleton-engine v1.0.0` at `0e01b0f`.
- Earlier migration switched survivor crop/icon UVs to engine helpers with temporary Y-flip compensation; this was superseded by the UV Orientation Migration Pass.
- Simplified survivor texture creation through `Sprite::textured_with_handle`, with `SurvivorTextureHandles` still owning game-specific path-to-handle lookup.
- Moved HUD slot, level-up, and shop icon/background drawing from per-frame world entity spawn/despawn to `UiImageQueue` + `DrawImage`.
- Latest validation: `cargo fmt --check`, `cargo test -p game --lib --locked -- --test-threads=1` (144 passed), `cargo check -p game --all-targets --locked`, and `cargo build -p game --bin survivor --release --locked`.

## Latest QA Fix Pass

- Fixed boss HP label/bar placement so it starts below the top-left HUD panel in default, 800x600, and Detailed HUD modes.
- Moved HUD slot, level-up card, and shop selection backgrounds that sit behind icons from `UiQueue` rects to screen-space sprites in `UiIconSystem`.
- Kept text in `TextQueue`; sprite backgrounds/icons now share the same sprite pass and order by `Transform.z`.
- Guarded HUD slot icon/background spawning to `SurvivorMode::InGame` so Title does not leak gameplay HUD sprites.
- Latest validation: `cargo fmt --check`, `cargo test -p game --lib --locked -- --test-threads=1` (141 passed), `cargo check -p game --all-targets --locked`, `cargo build -p game --bin survivor --release --locked`, and `bash scripts/package_macos.sh`.

## Latest Modernization Pass

- At that point, the game still used `skeleton-engine v1.0.0` at `78cebcf`; no engine repo or dependency update was made.
- Removed old lifecycle workarounds by using public ECS removal APIs for expired `HitFlash` components and consumed `PendingLevelUp` resources.
- Survivor textures now prefer handle-based sprites through `SurvivorTextureHandles`, with path fallback kept for tests and simple setup paths.
- Background, world, effect, and UI sprites now receive explicit `RenderLayer` values. Existing `Transform.z` values remain useful for ordering inside a layer.
- Engine-side desired improvements are tracked in `docs/ENGINE_FOLLOWUPS.md`.
- Latest validation at that point: `cargo fmt --check`, `cargo test -p game --lib --locked -- --test-threads=1` (139 passed), `cargo check -p game --all-targets --locked`, and `cargo build -p game --bin survivor --release --locked`.

## Recently Completed

- Sprite atlas crop/UV fix: actor/item sprites no longer use only head crops.
- Character/enemy size tuning: readable but reduced from the too-large 150px pass.
- Engine top-left UV orientation is now consumed directly; game-side direction compensation has been removed.
- Generated and documented new sprite sheets:
  - `survivor_effects.png`
  - `survivor_actor_frames.png`
  - `survivor_evolutions.png`
  - `survivor_icons.png`
  - `survivor_passives.png`
  - `survivor_powerups.png`
- Effects connected to weapons/systems:
  - Whip slash
  - Fireball
  - Cross projectile
  - Holy Water pool
  - Holy Book
  - Lightning strike
  - Garlic aura
  - Impact spark mapping
- Actor frame sheet connected through `AnimationPlayer` and `AnimationSystem`.
- UI icon sprites connected through `UiIconSystem` for HUD slots, level-up cards, and shop.
- `Sprite { ... }` direct literals removed from survivor code; use `Sprite::textured` / `Sprite::colored`.
- Asset license table and manual QA checklist updated.

## Sprite/Engine Decision

Keep current implementation for now:

- `Sprite::with_handle(handle)` when a loaded handle exists
- `Sprite::textured(path)` fallback
- manual `UvRect`
- `AnimationPlayer`

Do not do a broad `AtlasSprite` migration yet.

Reason:

- `survivor_atlas.png` uses custom non-uniform crop rectangles.
- Icon, passive, power-up, actor, and effect sheets still carry survivor-specific row/column/index metadata in game code.
- Engine `AtlasSprite` is better for future clean uniform grids, but direct migration still needs visual regression coverage for crop/index behavior.

Safe future improvement:

- Use `AtlasSprite` only for sheets that are truly uniform and do not require flipped UVs, after adding visual regression checks.

## Immediate Priority

1. Gameplay feature debt
   - Implement or intentionally remove currently inert stat effects: `luck`, `greed`, `curse`, and `revival`.
   - Implement Level-up `Reroll`/`Skip`/`Banish` behavior or stop selling/rewarding those upgrades.

2. Manual visual QA
   - KO/EN language switching for long strings in Title, Settings, Shop, cards, and results
   - Mouse hitbox feel for Title image buttons
   - Character and Stage select image-panel readability
   - Player/enemy/boss actor animation over longer play sessions
   - Whip/effect sprite visibility over longer play sessions

3. Responsive UI tuning
   - Verify high-DPI/Retina perceived sizes.
   - Keep player visually centered during boss spawn and zoom.

4. Localization expansion
   - Ensure Korean/English strings cover Settings, achievements, shop, cards, stages, and game over.

5. Asset polish
   - Replace local generated placeholder art/audio with final licensed assets when available.
   - Keep `docs/ASSET_LICENSES.md` updated with paths and hashes.

6. Release hardening
   - Re-run macOS package verification after every asset list change.
   - Confirm real speaker playback for BGM/SFX and boss/chest transitions.
   - Confirm StageClear through a long-play/manual release smoke pass.
   - Decide whether release/CI should keep the local `skeleton-engine` path dependency or switch back to a pinned git revision.
   - Keep save/meta changes simple; save compatibility is not required during active development unless requested.

## Working Rules

- Do not revert unrelated dirty changes.
- Use `rg` before broader inspection.
- Use `apply_patch` for file edits.
- Keep agent-facing docs under 200 lines.
- Keep long chronology in `docs/PHASE_LOG.md`, not here.
- Update `docs/manual_qa_checklist.md` for visual/manual verification changes.

## Done Criteria For Next Visual Pass

- Tests and release build pass.
- Manual QA checklist records checked screens/resolutions.
- No obvious sprite flip, crop, or scale regression.
- UI icons do not hide important HUD text.
- Package verification confirms all new assets are bundled.
