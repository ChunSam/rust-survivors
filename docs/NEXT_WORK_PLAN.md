# Rust Survivors Next Work Plan

Updated: 2026-05-28

This is the short agent-facing work plan. Long historical notes belong in `docs/PHASE_LOG.md`.

## Current Baseline

- Repo: `/Volumes/SSD/Projects/rust-survivors`
- Main work target: `crates/game`
- Engine: git dependency on `skeleton-engine`
- Engine source repo: `/Volumes/SSD/Projects/skeleton-engine`
- Current game build uses the latest engine commit in `Cargo.lock`.

Validation baseline:

```bash
cargo fmt --check
cargo test -p game --lib --locked -- --test-threads=1
cargo build -p game --bin survivor --release --locked
```

Run package checks after asset/package changes:

```bash
bash scripts/package_macos.sh
bash scripts/verify_macos_package.sh
```

Latest observed lib test status: 147 passing.

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
- Switched survivor crop UVs to `UvRect::from_pixels(...).flipped_y()` and icon sheet UVs to `UvRect::from_grid(...).flipped_y()`.
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
- Vertical UV flip applied where needed for engine quad orientation.
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

- Several sheets need vertical UV flip.
- `survivor_atlas.png` uses custom non-uniform crop rectangles.
- Engine `AtlasSprite` is better for future clean uniform grids, but direct migration can reintroduce sprite flipping/cropping bugs.

Safe future improvement:

- Use `AtlasSprite` only for sheets that are truly uniform and do not require flipped UVs, after adding visual regression checks.

## Immediate Priority

1. Manual visual QA
   - KO/EN language switching for long strings in Title, Settings, Shop, cards, and results
   - Mouse hitbox feel for Title image buttons
   - Character and Stage select image-panel readability
   - Player/enemy/boss actor animation over longer play sessions
   - Whip/effect sprite visibility over longer play sessions

2. Responsive UI tuning
   - Verify high-DPI/Retina perceived sizes.
   - Keep player visually centered during boss spawn and zoom.

3. Localization expansion
   - Ensure Korean/English strings cover Settings, achievements, shop, cards, stages, and game over.

4. Asset polish
   - Replace local generated placeholder art/audio with final licensed assets when available.
   - Keep `docs/ASSET_LICENSES.md` updated with paths and hashes.

5. Release hardening
   - Re-run macOS package verification after every asset list change.
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
