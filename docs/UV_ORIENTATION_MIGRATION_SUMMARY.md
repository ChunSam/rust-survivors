# UV Orientation Migration Summary

Date: 2026-05-29

## Context

The sibling `skeleton-engine` checkout now uses top-left-origin UVs for the shared
sprite quad. That makes normal PNGs render upright through `Sprite`, `DrawImage`,
`AtlasSprite`, `UvRect::FULL`, `UvRect::from_grid(...)`, and
`UvRect::from_pixels(...)`.

Before this engine change, `rust-survivors` compensated for the engine quad direction
by applying vertical UV flips in game code. After the engine fix, those compensations
would double-flip textures.

## Changes Applied

- Removed engine-direction Y-flip compensation from `crates/game/src/survivor/sprites.rs`.
- Removed the `vertically_flipped_full_uv()` helper.
- Removed Y-flip compensation from title backdrop setup and title menu `DrawImage` queues.
- Removed Y-flip compensation from HUD/modal/slot full-image UI queues.
- Removed Y-flip compensation from weapon/passive/powerup icon grid UVs.
- Updated unit tests to assert top-left-origin UVs and default `UvRect::FULL` for full-image UI.

## Current Code Policy

- Use `UvRect::from_pixels(...)` directly for hand-tuned crop rectangles.
- Use `UvRect::from_grid(...)` directly for uniform icon sheets.
- Use `UvRect::FULL` for full-image textures.
- Only use `flipped_y()` for intentional vertical mirroring. There are currently no such calls in `crates/game/src`.

## Verification

Passed:

```bash
cargo fmt --check
cargo clippy --all-targets --locked -- -D warnings
cargo test -p game --lib --locked -- --test-threads=1
cargo build -p game --bin survivor --release --locked
bash scripts/package_macos.sh
```

Observed result:

- `cargo test`: 151 passed
- macOS folder and `.app` package verification passed through `scripts/package_macos.sh`
- `rg -n "flipped_y\\(|vertically_flipped_full_uv" crates/game/src`: no matches

## Visual QA Result

Performed with the packaged app at:

```text
dist/macos/RustSurvivors.app
```

Confirmed:

- Title backdrop, logo plaque, Start button art, and Character/Stage/Shop/Settings button art render upright.
- InGame player actor, enemy actors, HUD frame art, weapon slot icon, and slot frames render upright.
- Debug boss spawn shows the boss actor and boss HP frame/label upright and below the top HUD.
- GameOver modal panel renders upright.
- Shop modal frame, row selection frame, and power-up icons render upright.
- Level-up card frame and passive icons render upright at 1280x720 and 800x600.
- 800x600 Title, Settings, InGame HUD/slot, and Level-up smoke pass did not show text/icon overlap regressions.

Not exhaustively confirmed:

- Long-session actor/effect animation quality and audio playback were not assessed.

QA helper added:

- `L` in InGame debug input sets the player XP to the next threshold so the next
  `LevelUpSystem` pass opens the Level-up card screen. This makes card/frame/icon
  orientation QA reproducible without waiting for combat drops.

## Visual QA Checklist

Run at the default resolution and 800x600:

- Title backdrop
- Title menu image buttons
- HUD slot frames
- Modal panels
- Level-up card frames
- Shop row frames
- Weapon icons
- Passive icons
- Powerup icons
- Actor animation frames
- Combat effect sprites

Expected result: all textures appear in the original PNG orientation without
double-flipping.
