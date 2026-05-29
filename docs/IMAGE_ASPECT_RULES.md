# Image Aspect Rules

Updated: 2026-05-29

## Rule

All full-image survivor UI textures must render at their original pixel aspect ratio.

When a layout needs a fixed interaction or content box, keep that box for hit testing and backing
color, then aspect-fit the PNG inside it. Do not stretch the PNG to fill an arbitrary rectangle.

All gameplay sprites must render at the aspect ratio of the exact frame or atlas cell that is
submitted to the renderer. Do not use an old crop ratio, a square fallback, or an arbitrary effect
rectangle as the final sprite scale.

## Scope

- Applies to full PNG UI assets: title logo, title menu buttons, in-game fixed text labels,
  modal title images, restart hints, section labels, modal panels, and slot frames.
- Applies to actor sprites: player, enemies, and bosses from `survivor_actor_frames.png`.
- Applies to atlas gameplay sprites: XP gems, pickups, chests, projectiles, orbiting books,
  weapon effects, garlic aura, holy water pool, lightning, and impact effects.
- Applies to icon sheet cells through UVs: weapon, passive, powerup, and evolution icons.
- Atlas cell rendering uses the source cell/crop dimensions as its aspect source.
- Full-screen/background images should use aspect-fill so the viewport is covered without
  distorting the source image.
- Plain colored rectangles and text-only effects are not image aspect targets.

## Implementation

- Register full-image texture dimensions in `survivor_texture_aspect(...)`.
- Before queueing a full-image `DrawImage`, fit the draw rectangle to that aspect.
- Keep colored backing rectangles at the original layout size when the UI needs a stable panel,
  button, or selection target.
- `SurvivorSprite::fit_scale(...)` uses `render_frame()` so actor sprites are fitted against the
  actual actor frame, not an older main-atlas crop.
- `SurvivorSprite::fit_inside(...)` fits timed effect sprites inside a requested gameplay bounds
  while preserving the rendered frame aspect.
- `add_sprite(...)` and `add_tinted_sprite(...)` re-fit entity `Transform.scale` to the render
  frame aspect after attaching the sprite and UV.

## Audit Notes

The 2026-05-29 pass checked all current image and sprite render paths:

- Title/menu full PNG images through `title_visual.rs`.
- In-game HUD, modal, slot, section, level-up, and gameover PNG images through `hud.rs`.
- Screen-space icon UI and slot/panel skins through `ui_icons.rs`.
- Actor sprites for player, enemies, and bosses through `sprites.rs`.
- Pickups, XP gems, chests, projectiles, orbiting books, area effects, timed effects, and weapon
  effects through their `add_sprite(...)` / `add_tinted_sprite(...)` call paths.

Issues fixed in this pass:

- Actor sprites used actor-sheet UVs but were previously scaled from older main-atlas crop ratios.
- Timed effects re-applied the caller's arbitrary `Vec2` after sprite fitting, which could stretch
  effects such as lightning or whip slashes.

## Validation

Aspect regressions are covered by tests in:

- `sprites.rs`: registered texture aspect values, actor sprite aspect, all `SurvivorSprite`
  fit-scale aspect, and `add_sprite(...)` render-frame aspect.
- `title_visual.rs`: title menu image queue preserves source aspects.
- `hud.rs`: in-game UI texture queue preserves source aspect.
- `ui_icons.rs`: shared screen texture queue preserves source aspect.
- `lightning.rs`: timed effect sprites preserve source aspect inside requested effect bounds.

Latest validation from this pass:

```bash
cargo fmt --check
cargo test -p game --lib --locked -- --test-threads=1
cargo build -p game --bin survivor --release --locked
bash scripts/package_macos.sh
bash scripts/verify_macos_package.sh
```

Result:

- lib tests: 167 passed
- survivor release build: passed
- macOS package verification: passed
