# CLAUDE.md - Rust Survivors Agent Notes

This file mirrors the active agent context in `AGENTS.md` and is intentionally kept under 200 lines. Prefer `AGENTS.md` when there is a conflict.

## Repository

- Game repo: `/Volumes/SSD/Projects/rust-survivors`
- Main crate: `crates/game`
- Engine repo: `/Volumes/SSD/Projects/skeleton-engine`
- Engine dependency: `engine = { package = "skeleton-engine", git = "https://github.com/ChunSam/skeleton-engine" }`

Do game/content/UI/asset work in this repo. Do renderer/ECS/input/audio/save engine work in the engine repo, then update the git dependency here.

## Commands

```bash
cargo run -p game --bin survivor
cargo test -p game --lib --locked -- --test-threads=1
cargo build -p game --bin survivor --release --locked
bash scripts/package_macos.sh
bash scripts/verify_macos_package.sh
```

Avoid `cargo clean`; use `rm -rf target` only if explicitly needed.

## Current Status - 2026-05-25

Implemented major roadmap through Phase 11 polish:

- Core survivor loop, 10 weapons, 16 passives, evolutions, pickups
- Enemy waves, bosses, stage clear
- Main menu, settings, meta save, shop, achievements
- Characters, stages, unlock flow
- Damage numbers, hit flash, particles, camera shake/zoom
- BGM/SFX placeholder asset pipeline
- macOS packaging scripts
- Sprite size/UV fixes and generated sprite sheets
- UI icon sprite overlay for HUD/cards/shop

Recent validation: `cargo fmt --check`, lib tests with 118 passing, release survivor build.

## Sprite Decisions

Keep the current sprite path:

- `Sprite::textured(path)`
- manual `UvRect`
- `AnimationPlayer` where animation is needed

Do not migrate everything to `AtlasSprite` yet. The current game needs custom crop rectangles and vertically flipped UVs. Engine `AtlasSprite` is a good future option for clean uniform-grid sheets, but a direct migration can break orientation or crop.

Recent cleanup:

- Removed direct survivor-side `Sprite { ... }` construction.
- Use `Sprite::textured` and `Sprite::colored`.
- Keep UV ownership in `sprites.rs` and `ui_icons.rs`.

## Asset State

Loaded in `src/bin/survivor.rs`:

- `assets/textures/survivor/survivor_atlas.png`
- `assets/textures/survivor/survivor_effects.png`
- `assets/textures/survivor/survivor_actor_frames.png`
- `assets/textures/survivor/survivor_evolutions.png`
- `assets/textures/survivor/survivor_icons.png`
- `assets/textures/survivor/survivor_passives.png`
- `assets/textures/survivor/survivor_powerups.png`
- `assets/textures/survivor/title_backdrop.png`

Applied paths:

- Actor frames: player/enemies/bosses via `SurvivorSprite::actor_row` and `AnimationSystem`.
- Effects: Whip, Fireball, Cross, HolyWater, HolyBook, Lightning, Garlic, ImpactSpark.
- UI icons: `UiIconSystem` for HUD slots, level-up cards, and shop.

## Key Files

- Entry: `crates/game/src/bin/survivor.rs`
- Survivor exports: `crates/game/src/survivor/mod.rs`
- Sprite mapping: `crates/game/src/survivor/sprites.rs`
- UI icon sprites: `crates/game/src/survivor/ui_icons.rs`
- HUD: `crates/game/src/survivor/hud.rs`
- Settings/resolution/meta save: `crates/game/src/survivor/meta.rs`
- World setup: `crates/game/src/survivor/world_setup.rs`
- Asset license table: `docs/ASSET_LICENSES.md`
- Manual visual QA: `docs/manual_qa_checklist.md`
- Long phase history: `docs/PHASE_LOG.md`

## Development Rules

- Never revert unrelated dirty worktree changes.
- Prefer `rg` for searching.
- Use `apply_patch` for manual file edits.
- Keep tests scoped but add coverage for behavior changes.
- Run lib tests after systems, sprite, UI, or save changes.
- For package/asset changes, run macOS package verification.
- If borrow checker workarounds are non-obvious, add a short reason.

## Next Work

1. Manual visual QA for animation, effects, icon overlay, and responsive UI.
2. Tune UI icon placement for low resolution and high-DPI windows.
3. Expand Korean/English localization coverage.
4. Replace placeholder assets with licensed final art/audio.
5. Continue macOS release hardening.
