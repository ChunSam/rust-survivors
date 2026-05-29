# AGENTS.md - Rust Survivors

## Scope

`rust-survivors` is the game repository for a Vampire Survivors-style top-down auto-attack roguelite built on `skeleton-engine`.

- Main work target: `crates/game`
- Engine: separate git dependency, `https://github.com/ChunSam/skeleton-engine`
- Do not edit deleted/legacy `crates/engine` paths in this repo.
- Do not directly edit external `skeleton-engine` checkouts from this repo. If an engine change is needed, leave an engine change request prompt in `docs/ENGINE_CHANGE_REQUESTS.md`.
- Preserve the platformer demo binary unless the user explicitly asks otherwise.

## Structure

```text
rust-survivors/
├── Cargo.toml
├── crates/game/
│   ├── src/main.rs              # platformer demo bin: game
│   ├── src/bin/survivor.rs      # survivor bin
│   └── src/survivor/            # actual survivor game logic
├── assets/
└── docs/
```

## Commands

```bash
cargo run -p game --bin survivor
cargo run -p game --bin survivor --release
cargo run -p game --bin game
cargo test -p game --lib --locked -- --test-threads=1
cargo build -p game --bin survivor --release --locked
bash scripts/package_macos.sh
bash scripts/verify_macos_package.sh
```

Avoid `cargo clean` on macOS here; use `rm -rf target` only when cleanup is explicitly needed.

## Controls

- Move: `W/A/S/D` or arrow keys
- Level-up cards: `1/2/3`
- Pause/back: `ESC`
- Debug visual checks: `F5` Bomb, `F6` Rosary, `F7` boss spawn

## Engine Boundary

Game code should use public engine APIs such as:

- ECS: `World`, `Entity`, `System`
- Components: `Transform`, `Sprite`, `AnimationPlayer`, `UvRect`
- App/render/input: `App`, `Camera`, `TextQueue`, `InputState`, `GameState`, `ViewportSize`
- Audio/save/collision: `AudioManager`, `save`, `Collider`, `SpatialGrid`

If an engine API change is needed:

1. Do not directly modify the `skeleton-engine` checkout.
2. Add a dated request prompt to `docs/ENGINE_CHANGE_REQUESTS.md`.
3. Include the problem, expected engine behavior/API, reproduction steps, affected game files, and validation commands.
4. Continue only with game-side changes that do not require engine edits.
5. After the engine change is completed externally, update the dependency or lockfile if needed, then run game tests and survivor build.

## Current State - 2026-05-25

Completed: Phase 0 through Phase 11 polish baseline, including vertical slice, weapon pool, passives, enemy variety, bosses, evolutions, pickups, menu/meta save, characters, stages, settings, achievements, localization scaffolding, macOS packaging, BGM/SFX placeholder assets, and recent sprite/UI asset pass.

Current validation baseline:

- `cargo fmt --check`
- `cargo test -p game --lib --locked -- --test-threads=1`
- `cargo build -p game --bin survivor --release --locked`
- macOS package + package verification when asset packaging changes

Latest verified test count: 118 lib tests passing.

## Recent Sprite Work

The game currently keeps the stable `Sprite::textured(...) + UvRect` path instead of migrating wholesale to engine `AtlasSprite`.

Reason:

- Game sprites need custom crop rects and vertically flipped UVs.
- Engine `AtlasSprite` is useful for uniform grids, but its default UV orientation can reintroduce flipped sprites.
- `survivor_atlas.png` uses hand-tuned crop rectangles for readable actor/item silhouettes.

Stability cleanup already done:

- Removed direct `Sprite { ... }` construction in survivor code.
- Use `Sprite::textured(...)` / `Sprite::colored(...)` where possible.
- Keep manual `UvRect` for crop and vertical flip control.

Key assets loaded by `src/bin/survivor.rs`:

- `survivor_atlas.png`
- `survivor_effects.png`
- `survivor_actor_frames.png`
- `survivor_evolutions.png`
- `survivor_icons.png`
- `survivor_passives.png`
- `survivor_powerups.png`
- `title_backdrop.png`

## Sprite/Application Status

- Actor sprites: `survivor_actor_frames.png` connected through `SurvivorSprite::actor_row`, `AnimationPlayer`, and `AnimationSystem`.
- Combat effects: `survivor_effects.png` used for Whip, Fireball, Cross, HolyWater, HolyBook, Lightning, Garlic, ImpactSpark.
- UI icons: `UiIconSystem` spawns screen-anchored sprites for HUD slots, level-up cards, and shop icons.
- Generated sheets for evolutions/passives/powerups/icons are documented in `docs/ASSET_LICENSES.md`.
- Visual sizes: player/enemies were increased, then reduced from the too-large pass; keep readability but avoid covering the playfield.

## Important Files

- Entry: `crates/game/src/bin/survivor.rs`
- World setup: `crates/game/src/survivor/world_setup.rs`
- Sprite mapping: `crates/game/src/survivor/sprites.rs`
- UI icon sprites: `crates/game/src/survivor/ui_icons.rs`
- HUD text: `crates/game/src/survivor/hud.rs`
- Camera and resolution: `camera_follow.rs`, `meta.rs`
- Manual QA: `docs/manual_qa_checklist.md`
- Asset provenance: `docs/ASSET_LICENSES.md`
- Long history: `docs/PHASE_LOG.md`

## Work Rules

- Do not revert unrelated dirty worktree changes.
- Use `rg`/`rg --files` first for searches.
- Use `apply_patch` for manual edits.
- Keep changes scoped to `crates/game` unless the user asks for docs/assets/package scripts.
- When an engine issue blocks game work, document it as an engine request prompt instead of editing the engine directly.
- Add tests when changing shared game behavior.
- For visual changes, run tests/build and update manual QA notes when relevant.
- Explain borrow checker/lifetime workarounds briefly; the user is a Rust beginner.

## Next Priorities

1. Visual QA pass for actor animation, effects, UI icons, and responsive UI.
2. Improve icon placement/readability at 800x600 and high-DPI windows.
3. Expand localization coverage.
4. Replace placeholder audio/art where final licensed assets are available.
5. Continue release hardening for macOS package.
