# Stage Tilemap Textured Backgrounds Applied

**Date:** 2026-06-12
**Status:** COMPLETED
**Bead(s):** none
**Epic:** Rust Survivors stage identity / asset polish
**Chain:** `stage-tilemaps` seq `1`
**Parent:** none — first in chain
**Prior chain:** none — first in chain

---

## Related Handoffs

- `plans/handoffs/HANDOFF_engine-audio-update_gameplay-actions_2026-06-04.md` — most recent gameplay/action baseline before this workstream; related because it documents current engine/API posture and validation discipline.
- `plans/handoffs/HANDOFF_engine-audio-update_sfx-bus-hidpi_2026-06-03.md` — related audio/settings chain only as nearby project history.
- `plans/handoffs/HANDOFF_survivor-ui-hud-layout_visual-qa-fixes_2026-06-02.md` — related visual QA chain; useful if future stage background QA finds UI or sprite layering issues.

## Reference Documents

- `AGENTS.md` — repository instructions, engine boundary, validation commands, controls, and active-plan rule.
- `CLAUDE.md` — companion agent context; currently dirty before this session and should not be treated as the active plan.
- `docs/NEXT_WORK_PLAN.md` — single active upcoming-work plan.
- `docs/manual_qa_checklist.md` — manual QA log and remaining window/speaker checks.
- `docs/ASSET_LICENSES.md` — asset provenance and hashes for release packaging.
- `docs/ENGINE_CHANGE_REQUESTS.md` — only place for new engine API requests if game-side work cannot proceed.

## The Goal

The user wanted the generated tilemap images saved and applied to the game before further stage-polish work. The intended end state was three selectable stages with visibly distinct world background moods, using engine rendering features rather than a separate bespoke background renderer. The change needed to preserve the existing survivor sprite policy: handle-backed `Sprite` plus manual `UvRect`, no broad migration to engine `AtlasSprite`, and no direct edits to the external `skeleton-engine` checkout. The user then asked for `/handoff` plus commit and push.

## Where We Are

- Branch at handoff creation: `main`.
- Engine dependency actually resolves through `crates/game/Cargo.toml` and `Cargo.lock` to `skeleton-engine v6.0.0` at git rev `77b4465558286c0fe90c37886affcabb7863ebeb`.
- `docs/NEXT_WORK_PLAN.md` now records that actual engine rev/package and the current verified test count of 210.
- Three generated stage tilemap PNGs are saved under `assets/textures/survivor/stages/`.
- `assets/textures/survivor/stages/mad_forest_tiles.png` is a 1254 x 1254 RGB PNG.
- `assets/textures/survivor/stages/inlaid_library_tiles.png` is a 1254 x 1254 RGB PNG.
- `assets/textures/survivor/stages/dairy_plant_tiles.png` is a 1254 x 1254 RGB PNG.
- The original generated image source directory was `/Users/jkl/.codex/generated_images/019eb744-517a-7f22-83c4-3a1ea3f8269b/`.
- Original generated file for library: `ig_02c2a470b54b58cf016a2b86820a5081919762451a8b528fc3.png`.
- Original generated file for forest: `ig_02c2a470b54b58cf016a2b85be267c8191b84386852e6e0414.png`.
- Original generated file for dairy: `ig_02c2a470b54b58cf016a2b8738b3cc81919ca41fc1f0d299e3.png`.
- `crates/game/src/survivor/sprites.rs` defines `MAD_FOREST_TILES_PATH`, `INLAID_LIBRARY_TILES_PATH`, and `DAIRY_PLANT_TILES_PATH`.
- `SurvivorTextureHandles` now owns `mad_forest_tiles`, `inlaid_library_tiles`, and `dairy_plant_tiles`.
- `SurvivorTextureHandles::handle_for` now maps the three new tilemap paths to their loaded image handles.
- `crates/game/src/bin/survivor.rs` imports the three tilemap path constants and loads them with `app.load_image(...)`.
- `crates/game/src/survivor/mod.rs` re-exports the three tilemap path constants for the survivor binary and background system.
- `crates/game/src/survivor/background.rs` no longer uses per-tile solid-color `Sprite::colored(...)` for stage mood.
- `BackgroundSystem` still uses the existing pooled background entity model and keeps camera/zoom coverage behavior.
- Background tile entities now receive a handle-backed textured `Sprite`, a `UvRect`, and `RenderLayer(RENDER_LAYER_BACKGROUND)`.
- `stage_tileset(StageKind)` maps Mad Forest, Inlaid Library, and Dairy Plant to separate tilemap paths and tint multipliers.
- `stage_tile_sprite(world, tileset)` builds a `survivor_textured_sprite(...)` and applies the stage tint.
- `tile_variant_index(stage, x, y)` uses the existing deterministic `hash01` helper plus stage offsets to choose one of 16 cells.
- `tile_uv(stage, x, y)` converts the variant index to `UvRect::from_grid(col, row, 4, 4)`.
- The stage tilemaps are therefore sampled as 4x4 uniform grids, not one large repeated texture.
- This deliberately uses the same engine `Sprite` + `UvRect` path already used for survivor sprites and UI icons.
- It does not require an engine change and did not edit any external `skeleton-engine` checkout.
- `docs/ASSET_LICENSES.md` records the new `assets/textures/survivor/stages/*.png` row and SHA-256 hashes.
- `docs/manual_qa_checklist.md` has a new `2026-06-12 Stage Tilemap Background Pass` section.
- Manual QA still needs actual macOS window inspection for stage mood, seams, empty edges, and gameplay readability.
- Automated validation completed after the image/code/docs work.
- Package validation confirmed the new tilemap files are present in both the folder package and `.app` Resources.
- Current worktree had many dirty files before this handoff request. Do not assume every dirty file belongs to this tilemap change.
- The intended commit for this handoff should stage only the tilemap work, its docs, and this handoff unless the user explicitly asks to include the older dirty files too.

## What We Tried (Chronological)

1. The user first asked for a completion-oriented game development plan that maximizes engine feature use.
   - The active planning source was consolidated into `docs/NEXT_WORK_PLAN.md`.
   - The plan emphasized public engine APIs: ECS `System`, `World` resources, `Sprite`, `UvRect`, `Camera`, `ViewportSize`, `AudioManager`, and `SpatialGrid`.
   - This set the context for using engine rendering primitives for stage backgrounds.

2. The user asked what the completion criteria are and whether phases 1-5 were already done.
   - Current state from `AGENTS.md` and repo docs showed Phase 0 through Phase 11 polish baseline already existed.
   - Remaining work was framed as verification, stage identity, polish, UX, asset finalization, and release-candidate hardening.

3. The user asked for a plan to apply tilemap images so the three stages feel visually different.
   - The selected approach was to keep `BackgroundSystem` and apply stage-specific tile textures there.
   - Rejected alternative: build a separate renderer or edit engine tilemap APIs. Not needed because `Sprite` + `UvRect` already covers the use case.
   - Rejected alternative: broad `AtlasSprite` migration. Current repo instructions say survivor art uses custom/manual crop metadata and stable `Sprite::textured(...) + UvRect`.

4. The user asked to generate the tilemap images first using image generation.
   - Three 1254 x 1254 RGB tilemaps were generated.
   - The images represented Mad Forest, Inlaid Library, and Dairy Plant moods.
   - They were viewed before copying into the repo.

5. The user asked to save the images and apply them to the game.
   - The files were copied to `assets/textures/survivor/stages/`.
   - `shasum -a 256` was run and the hashes were recorded.
   - `file` confirmed all three PNG dimensions and RGB format.

6. Source inspection focused on existing texture and background patterns.
   - `crates/game/src/survivor/background.rs` already pooled background tile entities and computed camera/zoom coverage.
   - `crates/game/src/survivor/sprites.rs` already centralized texture path constants and handle lookup.
   - `crates/game/src/bin/survivor.rs` already loaded images into `SurvivorTextureHandles`.
   - Engine `Sprite` supports `textured_with_handle(path, handle)` with path fallback and runtime image handle priority.
   - Engine `UvRect::from_grid(col, row, cols, rows)` supports uniform grid sampling.

7. `sprites.rs` was updated first.
   - Added `MAD_FOREST_TILES_PATH`, `INLAID_LIBRARY_TILES_PATH`, `DAIRY_PLANT_TILES_PATH`.
   - Added three handle fields to `SurvivorTextureHandles`.
   - Added three match arms in `handle_for`.
   - Updated the test helper `texture_handles_with`.
   - Extended the handle preference test to assert the new tile paths are registered.

8. `mod.rs` and `src/bin/survivor.rs` were updated next.
   - `mod.rs` re-exports the tile path constants.
   - `src/bin/survivor.rs` imports them and calls `app.load_image(...)`.
   - This preserves the existing handle-backed texture startup path used by other survivor assets.

9. `background.rs` was then changed from color-noise tiles to textured tiles.
   - The existing tile pool, coverage math, `TILE_SIZE`, and `TILE_Z` were preserved.
   - Spawned tile entities now get textured `Sprite`, `UvRect`, and `RenderLayer`.
   - Per-frame updates refresh transform position, texture path/handle, tint, and UV cell.
   - The `tile_color(...)` helper was removed because stage mood now comes from the generated texture plus tint.

10. A borrow-checker-sensitive area was adjusted proactively.
   - `stage_tile_sprite(world, tileset)` borrows `world` immutably to fetch the image handle.
   - The code creates `let sprite = stage_tile_sprite(world, tileset);` before `world.add_component(...)`.
   - The code creates `let next_sprite = stage_tile_sprite(world, tileset);` before `world.get_mut::<Sprite>(entity)`.
   - This shortens immutable borrows before mutable ECS writes.
   - This is important for Rust beginners: the workaround is not a clone-heavy hack; it is scoped borrowing.

11. Tests were added in `background.rs`.
   - `stage_tilesets_use_generated_asset_paths`.
   - `stage_tileset_assets_exist`.
   - `tile_variants_are_deterministic_and_inside_stage_atlas`.
   - `background_tiles_use_selected_stage_texture_and_uvs`.
   - Existing coverage tests for zoomed camera background coverage were kept.

12. `cargo fmt --check` initially failed.
   - Failure was only import order and line wrapping.
   - `cargo fmt` was run.
   - `cargo fmt --check` passed afterward.

13. `cargo test -p game --lib --locked -- --test-threads=1` passed.
   - Result: 210 passed, 0 failed.
   - The command compiled `skeleton-engine v6.0.0` from rev `77b446...`.
   - This test count includes the new background tests and previous automated work.

14. Documentation was updated.
   - `docs/ASSET_LICENSES.md` records stage tilemap provenance and hashes.
   - `docs/manual_qa_checklist.md` records automated checks and remaining manual QA.
   - `docs/NEXT_WORK_PLAN.md` records the actual engine rev/package, 210 passed status, and that stage backgrounds now use generated 4x4 tilemap PNGs.

15. Release build and packaging were run because assets changed.
   - `cargo build -p game --bin survivor --release --locked` passed.
   - `bash scripts/package_macos.sh` passed.
   - `bash scripts/verify_macos_package.sh` passed.
   - Package output explicitly listed the three `assets/textures/survivor/stages/*.png` files as OK.

16. The user then requested `/handoff 하고 커밋 푸쉬`.
   - This handoff is the requested structured handoff.
   - Next actions are self-validation, careful staging, commit, and push.
   - Because the worktree contains many unrelated dirty files, stage only intended files unless the user says otherwise.

## Key Decisions

- Use existing `BackgroundSystem` instead of introducing a new stage renderer.
  - Reason: it already handles tile pooling, camera follow, viewport coverage, and boss zoom-out coverage.

- Use engine `Sprite` and `UvRect` instead of a bespoke tilemap draw path.
  - Reason: this follows the repo's active sprite policy and uses current public engine APIs.

- Keep tilemaps as 4x4 sheets sampled by `UvRect::from_grid`.
  - Reason: the generated images were square 1254 x 1254 sheets and can provide 16 visual variants per stage without new metadata files.

- Apply a stage tint multiplier on top of the texture.
  - Reason: background readability matters; tint keeps art from overpowering enemies, pickups, damage numbers, and HUD.

- Keep tile selection deterministic from world grid coordinates.
  - Reason: background should not shimmer as the camera moves or as systems rerun.

- Use stage offsets in the tile hash.
  - Reason: even though each stage has a different texture, offsets avoid identical variant layouts if comparing stages at the same world coordinate.

- Preserve existing camera/zoom tile coverage tests.
  - Reason: a prior QA issue exposed empty background edges during 800x600 boss zoom-out, and this change must not regress that.

- Register new images in `SurvivorTextureHandles`.
  - Reason: packaged and dev runs should both prefer loaded image handles and keep path fallback for tests.

- Document asset hashes immediately.
  - Reason: release packaging policy requires provenance and SHA-256 for generated assets.

- Leave visual confirmation as manual QA.
  - Reason: unit tests can confirm path/UV/asset existence, but cannot judge actual mood, seams, or combat readability in a wgpu window.

- Commit scope should be narrow.
  - Reason: `git status -s` showed many pre-existing modified/deleted docs and code files. The tilemap commit should not accidentally include unrelated dirty state.

## Evidence & Data

### 1. Generated Stage Asset Files

| Stage | Repo path | Source mood | Format |
|---|---|---|---|
| Mad Forest | `assets/textures/survivor/stages/mad_forest_tiles.png` | moss, roots, dirt, green overgrowth | PNG RGB |
| Inlaid Library | `assets/textures/survivor/stages/inlaid_library_tiles.png` | dark stone, wood, red carpet, brass | PNG RGB |
| Dairy Plant | `assets/textures/survivor/stages/dairy_plant_tiles.png` | gray metal, concrete, pipes, grates | PNG RGB |

### 2. Asset Dimensions

```text
assets/textures/survivor/stages/mad_forest_tiles.png:     PNG image data, 1254 x 1254, 8-bit/color RGB, non-interlaced
assets/textures/survivor/stages/inlaid_library_tiles.png: PNG image data, 1254 x 1254, 8-bit/color RGB, non-interlaced
assets/textures/survivor/stages/dairy_plant_tiles.png:    PNG image data, 1254 x 1254, 8-bit/color RGB, non-interlaced
```

### 3. Asset Hashes

| File | SHA-256 |
|---|---|
| `mad_forest_tiles.png` | `c1e6742e1d9ac6e442e4ab7f949437f104815457b00e62e2ed235e87bde634c7` |
| `inlaid_library_tiles.png` | `1ed2470f436d244c4fb99cfae822b5474f7fe71cb8fe52a844eb1f8f02351fbb` |
| `dairy_plant_tiles.png` | `28fbb22193b69bc0c22585a460db04da5ac8aa5795dbe1d020d6bdb648ab6e06` |

### 4. Validation Commands

| Command | Result |
|---|---|
| `cargo fmt --check` | passed after running `cargo fmt` once |
| `cargo test -p game --lib --locked -- --test-threads=1` | 210 passed, 0 failed |
| `cargo build -p game --bin survivor --release --locked` | passed |
| `bash scripts/package_macos.sh` | passed |
| `bash scripts/verify_macos_package.sh` | passed |
| `git diff --check` | passed |

### 5. Cargo Test Summary

```text
running 210 tests
test result: ok. 210 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.12s
```

### 6. Release Build Summary

```text
Compiling skeleton-engine v6.0.0 (https://github.com/ChunSam/skeleton-engine?rev=77b4465558286c0fe90c37886affcabb7863ebeb#77b44655)
Compiling game v0.1.0 (/Users/jkl/Projects/rust-survivors/crates/game)
Finished `release` profile [optimized] target(s)
```

### 7. Package Evidence

```text
./assets/textures/survivor/stages/dairy_plant_tiles.png: OK
./assets/textures/survivor/stages/inlaid_library_tiles.png: OK
./assets/textures/survivor/stages/mad_forest_tiles.png: OK
./Contents/Resources/assets/textures/survivor/stages/dairy_plant_tiles.png: OK
./Contents/Resources/assets/textures/survivor/stages/inlaid_library_tiles.png: OK
./Contents/Resources/assets/textures/survivor/stages/mad_forest_tiles.png: OK
```

### 8. Relevant Diff Size

| File | Insertions | Deletions |
|---|---:|---:|
| `crates/game/src/bin/survivor.rs` | 10 | 6 |
| `crates/game/src/survivor/background.rs` | 148 | 22 |
| `crates/game/src/survivor/mod.rs` | 19 | 17 |
| `crates/game/src/survivor/sprites.rs` | 16 | 0 |
| `docs/ASSET_LICENSES.md` | 2 | 1 |
| `docs/NEXT_WORK_PLAN.md` | 199 | 209 |
| `docs/manual_qa_checklist.md` | 20 | 1 |

### 9. Git State at Handoff Creation

| Field | Value |
|---|---|
| Branch | `main` |
| Recent HEAD | `801f334 deps: bump skeleton-engine pin 5.0.0 -> 6.0.0 (77b4465)` |
| Worktree | dirty before this handoff |
| Untracked stage assets | `assets/textures/survivor/stages/` |
| New handoff | `plans/handoffs/HANDOFF_stage-tilemaps_textured-backgrounds_2026-06-12.md` |

### 10. Dirty Files to Treat Carefully

The following dirty files existed at handoff time. Some are part of this work, some are pre-existing and should not be staged blindly:

```text
M AGENTS.md
M CLAUDE.md
M crates/game/src/bin/survivor.rs
M crates/game/src/survivor/README.md
M crates/game/src/survivor/background.rs
M crates/game/src/survivor/data.rs
M crates/game/src/survivor/locale.rs
M crates/game/src/survivor/mod.rs
M crates/game/src/survivor/sfx.rs
M crates/game/src/survivor/sprites.rs
M crates/game/src/survivor/stage.rs
M docs/ASSET_LICENSES.md
M docs/NEXT_WORK_PLAN.md
M docs/manual_qa_checklist.md
?? assets/textures/survivor/stages/
```

There are also several dirty/deleted docs outside the tilemap scope. Do not stage them unless intentionally committing the broader previous doc cleanup.

## Code Analysis

- `BackgroundSystem::run(&mut self, world: &mut World, _dt: f32)` still exits unless `SurvivorMode` is `InGame` or `PauseMenu`.
- `BackgroundSystem` still reads `Camera`, `ViewportSize`, and `SelectedStage` as resources.
- `tile_grid_size(viewport, camera.zoom)` still computes coverage as `viewport / zoom` plus padding, preserving boss zoom-out behavior.
- `TILE_SIZE` remains `96.0`; `TILE_Z` remains `-10.0`.
- `TILE_ATLAS_COLS` and `TILE_ATLAS_ROWS` are both `4`.
- Stage tints are:
  - Mad Forest: `[0.62, 0.74, 0.58, 1.0]`
  - Inlaid Library: `[0.70, 0.60, 0.52, 1.0]`
  - Dairy Plant: `[0.60, 0.66, 0.72, 1.0]`
- Tile variant stage offsets are:
  - Mad Forest: `0`
  - Inlaid Library: `10_000`
  - Dairy Plant: `20_000`
- `tile_variant_index` clamps to `variant_count - 1` after flooring to guard against edge-case float multiplication.
- Existing `hash01(x, y)` remains deterministic and is still used for background variation.
- New tests directly instantiate `BackgroundSystem`, insert `SurvivorMode::InGame`, `ViewportSize`, `Camera`, and `SelectedStage`, then inspect spawned tile components.
- `survivor_textured_sprite(world, path)` remains the single helper for creating handle-backed survivor texture sprites.
- The fallback behavior remains intact: tests without `SurvivorTextureHandles` can still use texture path strings.
- No save schema, stage unlock, wave data, enemy spawn, combat, UI controls, audio behavior, or platformer demo binary was intentionally changed by the tilemap work.

## Files Changed

### Source code

- `crates/game/src/survivor/background.rs` — converted background tiles from generated color noise to stage-specific textured tiles; added `StageTileset`, `stage_tileset`, `stage_tile_sprite`, `tile_uv`, `tile_variant_index`, and tests.
- `crates/game/src/survivor/sprites.rs` — added stage tilemap texture path constants and `SurvivorTextureHandles` fields/mapping.
- `crates/game/src/survivor/mod.rs` — re-exported stage tilemap path constants.
- `crates/game/src/bin/survivor.rs` — loaded the three tilemap PNGs through `App::load_image`.

### Assets

- `assets/textures/survivor/stages/mad_forest_tiles.png` — generated Mad Forest 4x4 tilemap.
- `assets/textures/survivor/stages/inlaid_library_tiles.png` — generated Inlaid Library 4x4 tilemap.
- `assets/textures/survivor/stages/dairy_plant_tiles.png` — generated Dairy Plant 4x4 tilemap.

### Tests

- `crates/game/src/survivor/background.rs` — added tests for stage path mapping, asset existence, UV range, deterministic variant selection, and selected-stage texture switching.
- `crates/game/src/survivor/sprites.rs` — extended texture handle mapping test to include the new stage assets.

### Documentation

- `docs/ASSET_LICENSES.md` — added stage tilemap asset row and hashes.
- `docs/manual_qa_checklist.md` — added Stage Tilemap Background Pass section and validation results.
- `docs/NEXT_WORK_PLAN.md` — updated actual engine baseline, validation count, and current stage background implementation note.
- `plans/handoffs/HANDOFF_stage-tilemaps_textured-backgrounds_2026-06-12.md` — this handoff.

### Generated outputs not intended for commit unless already tracked/desired

- `dist/macos/RustSurvivors`
- `dist/macos/RustSurvivors.app`

These were regenerated by packaging commands but are expected to be ignored or handled by release packaging, not blindly committed.

## User Feedback & Preferences

- User wants the game finished, not just open-ended planning.
- User asked to maximize use of engine features.
- User asked for completion criteria.
- User asked whether phases 1-5 had already been completed.
- User wants three stages to feel visually distinct through tilemap images.
- User asked to plan before implementing the stage tilemap image application.
- User explicitly asked to generate the tilemap images first.
- User then explicitly asked to save the images and apply them to the game.
- User now asked `/handoff 하고 커밋 푸쉬`, so a handoff file, commit, and push are expected.
- User generally accepts direct execution once scope is clear; avoid stopping at proposals when implementation is feasible.
- User is a Rust beginner per repo instructions, so explain borrow-checker-sensitive changes briefly when reporting.
- Repo instructions say to keep active planning consolidated in `docs/NEXT_WORK_PLAN.md`, not duplicated across agent docs.
- Repo instructions say not to directly edit external `skeleton-engine` checkouts.
- Repo instructions say preserve the platformer demo binary unless explicitly asked otherwise.
- Repo instructions say run tests/build and update manual QA notes for visual changes.

## Where We're Going

1. Finish this handoff self-validation.
2. Stage only intended tilemap/source/doc/handoff files, not the unrelated dirty worktree.
3. Commit with a concise message such as `Add stage tilemap backgrounds`.
4. Push `main`.
5. Next actual development step should be manual macOS window QA for the three stage backgrounds:
   - StageSelect -> Mad Forest run.
   - StageSelect -> Inlaid Library run.
   - StageSelect -> Dairy Plant run.
   - Check 800x600, default resolution, and boss zoom-out.
6. If readability is poor, tune only `stage_tileset` tint values first before regenerating art.
7. Continue M2 stage identity work from `docs/NEXT_WORK_PLAN.md`: wave pacing, enemy mix, pickup pacing, and boss pressure.

## Risks & Blockers

- The worktree contains many dirty files unrelated to the tilemap implementation; careless `git add -A` would commit unrelated doc deletions and previous code/docs work.
- Visual quality is not fully verified until the game is run in a real macOS window.
- Generated images are placeholders/internal AI-generated assets; final release may need replacement with final licensed art.
- `docs/NEXT_WORK_PLAN.md` has a large diff from HEAD, partly from earlier planning cleanup. Include it only if committing the current active-plan state intentionally.
- Tints may need adjustment after manual QA if backgrounds reduce enemy/pickup readability.

## Open Questions

- Should each stage eventually get collision-free decorative props or parallax layers, or are textured floor tiles enough for the current release target?
- Should `TILE_SIZE` stay at 96.0 for all stages, or should later stage polish use stage-specific tile scale?
- Should the 4x4 generated sheets be replaced by hand-authored uniform tile atlases before release?
- Should `docs/NEXT_WORK_PLAN.md` be committed as part of this tilemap checkpoint despite its broader active-plan rewrite?
- Should future visual QA use screenshots through Computer Use/Browser-style automation, or manual player confirmation only?

## Quick Start for Next Session

```bash
# Restore context
sed -n '1,360p' plans/handoffs/HANDOFF_stage-tilemaps_textured-backgrounds_2026-06-12.md

# Reference docs
sed -n '1,220p' AGENTS.md
sed -n '1,240p' docs/NEXT_WORK_PLAN.md
sed -n '1,80p' docs/manual_qa_checklist.md

# Key files to read first
sed -n '1,180p' crates/game/src/survivor/background.rs
sed -n '1,190p' crates/game/src/survivor/sprites.rs
sed -n '130,155p' crates/game/src/bin/survivor.rs

# Evidence / data files
ls -l assets/textures/survivor/stages
shasum -a 256 assets/textures/survivor/stages/*.png

# Verify current state
cargo fmt --check
cargo test -p game --lib --locked -- --test-threads=1
cargo build -p game --bin survivor --release --locked

# Next action
cargo run -p game --bin survivor
# Then use StageSelect to start each of the three stages and visually check background mood/readability.
```

## Handoff Self-Check

- Chain is `stage-tilemaps` seq `1`.
- Parent is `none — first in chain`.
- Related handoffs are listed but not treated as parents.
- "Where We Are" names real source, docs, assets, commands, and current git state.
- "What We Tried" captures image generation, save/apply path, borrow-scope fix, tests, docs, package validation, and the commit/push request.
- "Evidence & Data" includes exact file paths, hashes, dimensions, command results, and package manifest evidence.
- "Code Analysis" names functions, constants, and resource/component behavior.
- "User Feedback & Preferences" is present and includes the latest `/handoff` plus commit/push request.
- "Quick Start for Next Session" has a concrete first action: run the game and visually QA all three stage backgrounds.
