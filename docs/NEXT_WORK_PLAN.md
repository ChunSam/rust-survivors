# Rust Survivors Next Work Plan

Updated: 2026-06-12

This is the only active upcoming-work plan for the repository.

Long chronology belongs in `docs/PHASE_LOG.md`; manual verification details belong in `docs/manual_qa_checklist.md`; release procedure belongs in `docs/release_checklist.md`. Historical pass-summary docs may mention follow-ups from their original pass, but they are not active planning sources.

## Current Baseline

- Repo: `/Users/jkl/Projects/rust-survivors`
- Main work target: `crates/game`
- Engine: pinned git dependency on `https://github.com/ChunSam/skeleton-engine`
- Current engine rev: `77b4465558286c0fe90c37886affcabb7863ebeb`
- Current engine package: `skeleton-engine v6.0.0`
- Dependency workflow: `docs/ENGINE_DEPENDENCY_WORKFLOW.md`
- Engine change requests: `docs/ENGINE_CHANGE_REQUESTS.md`

Do not directly edit external `skeleton-engine` checkouts from this repo. If game work needs an engine API change, write a dated request in `docs/ENGINE_CHANGE_REQUESTS.md` and keep this repo on game-side changes only.

## Validation Baseline

```bash
cargo fmt --check
cargo test -p game --lib --locked -- --test-threads=1
cargo build -p game --bin survivor --release --locked
```

Run package checks after asset/package changes or before release:

```bash
bash scripts/package_macos.sh
bash scripts/verify_macos_package.sh
```

Latest verified status (2026-06-12):

- `cargo fmt --check`: passed
- `cargo test -p game --lib --locked -- --test-threads=1`: 213 passed
- `cargo build -p game --bin survivor --release --locked`: passed
- `bash scripts/package_macos.sh`: passed
- `bash scripts/verify_macos_package.sh`: passed

Manual speaker/window QA is still outstanding.

## Latest Engine / Gameplay Action Baseline

- Current `crates/game` engine dependency resolves to pinned git rev `77b4465558286c0fe90c37886affcabb7863ebeb`.
- Current resolved engine package is `skeleton-engine v6.0.0`.
- Kept engine v2 ECS compatibility by using `Entity::index()` where direct entity tuple-field access was no longer valid.
- Kept SFX on an `sfx` audio bus so `MetaSave.sfx_volume` controls file and tone SFX consistently.
- Wired previously inert meta powerups into runtime behavior:
  - `Luck`: affects normal pickup drop thresholds.
  - `Greed`: affects coin pickup value.
  - `Curse`: affects spawned enemy HP, speed, and contact damage.
  - `Revival`: consumes run-local revivals before GameOver.
  - `Reroll`, `Skip`, `Banish`: level-up actions with run-local remaining counts.
- Added localized level-up action hints for Korean and English.
- Added focused unit tests for new helpers, action state, revival behavior, curse/luck/greed, localization, and `Entity::index()` sorting.

## Current Implementation Notes

- Save schema is unchanged; existing `MetaSave.powerup_levels` values drive the new runtime effects.
- Level-up controls:
  - `1/2/3`: choose card.
  - `R`: reroll current offer if rerolls remain.
  - `S`: skip current level-up if skips remain.
  - `4/5/6`: banish offered card 1/2/3 if banishes remain.
- `LevelUpActions` and `RevivalState` are run-local resources and reset on run reset.
- Stage backgrounds use generated 4x4 tilemap PNGs through handle-backed `Sprite` plus per-tile `UvRect`; Mad Forest, Inlaid Library, and Dairy Plant now have distinct world textures.
- Actor sprites flip horizontally from their world x movement direction after `AnimationSystem`, so player/enemy/boss sprites do not appear to walk backward when moving left.
- New gameplay behavior is automated-test covered, but live feel still needs manual QA.

## Sprite / UI Policy

- Keep `Sprite::textured(...)` / handle-backed textured sprites plus manual `UvRect` for survivor-specific crop data.
- Do not broadly migrate survivor sheets to `AtlasSprite` yet.
- Use `UvRect::from_pixels(...)`, `UvRect::from_grid(...)`, and `UvRect::FULL` directly. `flipped_x()` / `flipped_y()` should mean intentional art mirroring, not engine orientation compensation.
- Keep `SurvivorSprite`, icon indexes, UI slot/card/shop layout, and generated atlas metadata in game code.

## Completion Goal

Ship a small but complete Vampire Survivors-style macOS game built on the current public `skeleton-engine` API:

- One polished run loop from title -> character/stage/shop/settings -> run -> level-up/chest/boss -> stage clear or game over -> meta progression.
- Three stages remain selectable and unlockable, with each stage feeling distinct through wave pacing, boss pressure, background presentation, and reward pacing.
- Existing 10 weapons, passives, evolutions, pickups, meta powerups, characters, achievements, localization, BGM/SFX, and packaging paths are verified rather than replaced.
- Platformer demo binary remains intact.
- No direct edits to external `skeleton-engine` checkouts. Engine gaps become dated prompts in `docs/ENGINE_CHANGE_REQUESTS.md`.

## Engine-First Development Strategy

Use the engine features as the default implementation path before adding game-local infrastructure:

- ECS / systems: keep gameplay behavior in focused `System` implementations registered from `src/bin/survivor.rs`; preserve explicit system order as a gameplay contract.
- Resources: model global run/menu state as resources (`SurvivorMode`, `GameState`, `MetaSave`, `SelectedStage`, `SpawnDirector`, queues) instead of static globals or ad hoc singletons.
- Rendering: keep world sprites on `Sprite::textured(...)` plus `UvRect` for hand-cropped survivor sheets; keep screen-space UI images in `UiImageQueue`; keep text in `TextQueue`.
- Animation: use engine `AnimationPlayer` + `AnimationSystem` for actor/effect frame progression and avoid bespoke animation timers unless the sprite is not frame-based.
- Collision/performance: use `Collider`, `CollisionLayer`, and `SpatialGrid` for all area queries that can touch many enemies or pickups.
- Camera/viewport: route world presentation through `Camera` and responsive UI through `ViewportSize`; do not hard-code a single target resolution.
- Audio: continue using engine `AudioManager` with separate BGM/SFX buses, with game systems pushing `SfxQueue` events instead of borrowing audio directly.
- Save/config: keep persistent progression in engine-backed save paths through `MetaSave`; add schema fields cautiously with migration/default tests.
- Engine requests: if an engine API would remove real duplication or unblock correctness, document the request first; continue with game-side work only when possible.

## Milestone Plan

### M1 - Lock The Current Vertical Slice

Goal: make the current build trustworthy before adding content.

- Run the validation baseline: `cargo fmt --check`, `cargo test -p game --lib --locked -- --test-threads=1`, and `cargo build -p game --bin survivor --release --locked`.
- Complete manual QA for SFX bus behavior:
  - SFX volume at 100%, 50%, and 0%.
  - BGM volume and SFX volume independence.
  - XP pickup, enemy hit, bomb, level-up, boss appear, and chest open.
- Complete manual QA for level-up actions:
  - `R/S/4/5/6` work only during pending level-up.
  - Remaining counts display correctly.
  - Banish excludes the selected card from regenerated offers.
  - 800x600 layout does not overlap cards or HUD.
- Complete runtime feel QA:
  - Luck changes ordinary pickup frequency without overwhelming drops.
  - Greed changes coin value as expected.
  - Curse changes enemy pressure without breaking collision/readability.
  - Revival restores the player once per available count, then GameOver occurs after all revivals are spent.
- Update `docs/manual_qa_checklist.md` with the actual results, including any reproduction steps.

Engine focus: `GameState`, `ViewportSize`, `AudioManager`, `TextQueue`, `UiImageQueue`, and existing ECS resources.

### M2 - Content Balance And Stage Identity

Goal: turn the existing systems into a complete 3-stage progression.

- Review `assets/data/waves_mad_forest.ron`, `waves_inlaid_library.ron`, and `waves_dairy_plant.ron` as player-facing content, not just parser data.
- Tune each stage for:
  - 0-2 minute onboarding pressure.
  - Mid-run enemy density that exercises `SpatialGrid` without tanking readability.
  - Late-run boss pressure and stage-clear timing.
  - Distinct enemy mixes, movement profiles, and pickup pacing.
- Keep stage differences mostly data-driven through wave RON and `StageKind`; add code only when current data cannot express a needed behavior.
- Add or update tests for wave loading, stage unlock prerequisites, boss/stage clear transitions, and any new balancing helpers.
- If stage-specific backgrounds need richer presentation, use existing `BackgroundSystem`, `Camera`, `Sprite`, and `ViewportSize` first.

Engine focus: `World` resources, `System` order, `SpatialGrid`, `Camera`, `ViewportSize`, and data-driven content loading.

### M3 - Combat Feel And Progression Polish

Goal: ensure every weapon/passive/evolution path is understandable, useful, and satisfying.

- Playtest all 10 weapons through normal level-up, evolved state, and chest interaction.
- Tune weapon curves in `assets/data/weapons.ron` and game-side upgrade helpers before adding new weapon types.
- Verify passives affect visible runtime decisions: movement, cooldown, area, amount, magnet, growth, luck, greed, curse, armor, recovery, revival.
- Make damage numbers, hit flashes, particles, and SFX support readability rather than clutter.
- Keep high-volume collision checks on `SpatialGrid`; avoid per-enemy full scans in new combat logic.
- Add focused tests for any changed shared helpers, especially stat recomputation, evolution eligibility, projectile lifetime, pierce, and pickup effects.

Engine focus: `AnimationPlayer`, `AnimationSystem`, `Collider`, `CollisionLayer`, `SpatialGrid`, `Sprite`, `UvRect`, and queued audio/text feedback.

### M4 - UX, Accessibility, And Localization Completion

Goal: remove the friction that makes the game feel like a prototype.

- Verify keyboard-only flows for Title, CharacterSelect, StageSelect, Shop, Achievements, Settings, PauseMenu, LevelUp, StageClear, and GameOver.
- Keep Korean and English strings aligned for all player-facing text and hints.
- Stabilize 800x600, default, and high-DPI layouts; fix overlaps in `ui_layout.rs`, `hud.rs`, or `ui_icons.rs` rather than changing controls.
- Confirm settings persist: language, resolution, HUD detail, BGM volume, and SFX volume.
- Ensure game-over, restart, return-to-title, and pause flows reset only run-local resources and preserve meta progression.
- Add tests for layout math and mode transitions whenever a UI fix depends on geometry or state order.

Engine focus: `InputState`, `GameState`, `ViewportSize`, `WindowConfig`, `PendingResize`, `TextQueue`, and `UiImageQueue`.

### M5 - Asset And Audio Finalization

Goal: make the shipped package coherent and license-clean.

- Replace placeholder art/audio only when final licensed assets are available.
- Keep `docs/ASSET_LICENSES.md` and `docs/audio_assets.md` current for every asset change.
- Verify actor sprites, combat effects, UI icons, title backdrop, menu buttons, BGM, and SFX from both dev run and packaged app.
- Prefer existing texture handle loading from `src/bin/survivor.rs`; avoid runtime asset discovery that can diverge between dev and packaged app.
- Keep survivor-specific crop metadata in `sprites.rs` and `ui_icons.rs`; do not migrate to engine `AtlasSprite` unless visual regression coverage is ready.

Engine focus: `App::load_image`, handle-backed `Sprite`, `UvRect`, `AnimationPlayer`, `AudioManager`, and package-relative asset paths.

### M6 - Release Candidate Hardening

Goal: freeze behavior and produce a reproducible macOS release candidate.

- Re-run:
  - `cargo fmt --check`
  - `cargo test -p game --lib --locked -- --test-threads=1`
  - `cargo build -p game --bin survivor --release --locked`
  - `bash scripts/package_macos.sh`
  - `bash scripts/verify_macos_package.sh`
- Run one complete manual pass from fresh save:
  - new player start
  - first level-up
  - first chest/evolution check
  - boss spawn
  - stage clear
  - next-stage unlock
  - shop purchase
  - quit/relaunch save persistence
- Run one complete failure pass:
  - player death
  - revival if available
  - final GameOver
  - restart
  - return to title
- Update `docs/manual_qa_checklist.md` with release-candidate results and update this file only if priorities change.
- Keep `docs/release_checklist.md` limited to procedure and validation commands.

Engine focus: stable app startup, save/load, audio initialization fallback, render queue correctness, and deterministic system order.

## Priority Order

1. Finish M1 QA and document results.
2. Tune stage/wave progression in M2.
3. Polish combat/progression feel in M3.
4. Close UX/localization persistence gaps in M4.
5. Finalize assets/audio and licenses in M5.
6. Cut a release candidate with M6.

## Working Rules

- Do not revert unrelated dirty changes.
- Use `rg` before broader inspection.
- Use `apply_patch` for manual file edits.
- Add tests when changing shared game behavior.
- Update `docs/manual_qa_checklist.md` for visual/manual verification changes.
- Keep upcoming-work changes consolidated in this file; do not create parallel next-priority lists elsewhere.
