# Rust Survivors Next Work Plan

Updated: 2026-05-28

This is the short agent-facing work plan. Long historical notes belong in `docs/PHASE_LOG.md`.

## Current Baseline

- Repo: `/Volumes/SSD/Projects/rust-survivors`
- Main work target: `crates/game`
- Engine: git dependency on `rust-2d-engine`
- Engine source repo: `/Volumes/SSD/Projects/rust-2d-engine`
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

Latest observed lib test status: 139 passing.

## Latest Modernization Pass

- Game still uses `skeleton-engine v1.0.0` at `78cebcf`; no engine repo or dependency update was made.
- Removed old lifecycle workarounds by using public ECS removal APIs for expired `HitFlash` components and consumed `PendingLevelUp` resources.
- Survivor textures now prefer handle-based sprites through `SurvivorTextureHandles`, with path fallback kept for tests and simple setup paths.
- Background, world, effect, and UI sprites now receive explicit `RenderLayer` values. Existing `Transform.z` values remain useful for ordering inside a layer.
- Engine-side desired improvements are tracked in `docs/ENGINE_FOLLOWUPS.md`.
- Latest validation: `cargo fmt --check`, `cargo test -p game --lib --locked -- --test-threads=1` (139 passed), `cargo check -p game --all-targets --locked`, and `cargo build -p game --bin survivor --release --locked`.

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
   - Title screen
   - InGame at 800x600 and default resolution
   - Player/enemy/boss actor animation
   - Whip/effect sprite visibility
   - Level-up cards
   - Shop icons
   - HUD weapon/passive slots

2. Responsive UI tuning
   - Verify icon/text overlap at low resolution.
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
