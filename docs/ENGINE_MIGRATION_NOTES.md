# Engine Migration Notes

This document tracks rust-survivors work that should remain game-specific after the engine follow-ups are implemented in `skeleton-engine`.

## What Moved Into The Engine

The following capabilities are general engine features now consumed from the pinned `skeleton-engine` git dependency:

- Top-left-origin sprite quad UVs for `Sprite`, `DrawImage`, and `AtlasSprite`.
- `UvRect::from_pixels(...)`, `UvRect::from_grid(...)`, `flipped_x()`, and `flipped_y()` for custom crop rectangles and explicit mirroring.
- `AtlasSprite` rendering now honors a `UvRect` component on the same entity as a custom UV override.
- `Sprite::textured_with_handle(path, Option<Handle<ImageAsset>>)` for handle-first rendering with a path fallback.
- `DrawImage` and `UiImageQueue` for camera-independent screen-space textured UI quads.

## Game-Specific Work To Keep Here

### Survivor Sprite Crop Data

Keep the survivor sprite mapping tables in game code.

Files:
- `crates/game/src/survivor/sprites.rs`
- `crates/game/src/survivor/icons.rs`
- `crates/game/src/survivor/ui_icons.rs`

Why:
- `survivor_atlas.png` contains hand-tuned crop rectangles.
- Actor frame rows, effect frame slots, pickup crops, and icon indexes are content data, not engine policy.
- Engine APIs should only provide UV/crop primitives, not survivor-specific sprite names or atlas coordinates.

Current policy:
- Use `UvRect::from_pixels(...)` directly for pixel crop rectangles.
- Use `UvRect::from_grid(...)` directly for uniform sheets.
- Only use `flipped_x()` / `flipped_y()` for intentional visual mirroring, not engine orientation correction.
- Keep `SurvivorSprite`, `IconSource`, `passive_index`, and `powerup_index` in game code.

### Texture Handle Lookup Policy

Keep `SurvivorTextureHandles` or an equivalent game resource in game code.

Why:
- The engine cannot know which game texture resource corresponds to a string path.
- The lookup table is tied to rust-survivors asset loading and test setup.

Current pattern:
- `survivor_textured_sprite(world, path)` calls:

```rust
Sprite::textured_with_handle(
    path,
    world
        .resource::<SurvivorTextureHandles>()
        .and_then(|textures| textures.handle_for(path))
        .cloned(),
)
```

This keeps the existing test fallback behavior while using the engine helper.

### HUD, Level-Up, And Shop Icon Layout

Keep layout policy in game code.

File:
- `crates/game/src/survivor/ui_icons.rs`

Why:
- Slot positions, responsive scale, level-up card offsets, shop rows, and z choices are rust-survivors UI design.
- The engine should render screen-space images; it should not decide where survivor HUD icons belong.

Current pattern:
- Keep `hud_icon_layout`, `levelup_icon_center`, `shop_icon_center`, and icon source selection in game code.
- Submit screen-space UI background/icon sprites from `UiIconSystem` when z-order must place icons above their backgrounds.
- Use `UiImageQueue` for full-image UI that does not need sprite-pass z interleaving.
- Push `DrawImage` instances using logical viewport coordinates when using `UiImageQueue`:

```rust
world.resource_mut::<UiImageQueue>().unwrap().push(
    DrawImage::textured_with_handle(x, y, size, size, texture, handle)
        .with_uv(uv)
        .with_z(z),
);
```

`UiIconSprite`, `despawn_previous_icons`, camera `screen_to_world` conversion, and UI `RenderLayer`
assignment remain game-side because the current HUD/card/shop backgrounds and icons are intentionally
ordered together in the sprite pass.

## Migration Order

1. Keep the `engine` dependency pinned to a commit that includes top-left sprite quad UVs.
2. Use `UvRect::from_pixels(...)`, `UvRect::from_grid(...)`, and `UvRect::FULL` without engine-direction Y-flip compensation.
3. Keep `survivor_textured_sprite` on `Sprite::textured_with_handle`.
4. Keep `UiIconSystem` responsible for screen-space icon/background drawing.
5. Run visual QA for title, gameplay, HUD slots, level-up cards, shop icons, pickups, and effects at default size and 800x600.

## Applied Status - 2026-05-28

- Game dependency updated to `skeleton-engine` commit `0e01b0f383202acea4d2ff606be4c62f73371471`.
- `sprites.rs` now uses engine UV flip/crop helpers and `Sprite::textured_with_handle`.
- `ui_icons.rs` now queues screen-space `DrawImage` values through `UiImageQueue`; `UiIconSprite` and per-frame icon entity despawn are removed.
- Automatic validation passed: `cargo fmt --check`, game lib tests (144 passed), all-target check, and release survivor build.

## Applied Status - 2026-05-29

- Game dependency temporarily resolved `skeleton-engine` from the sibling checkout at `/Users/jkl/Projects/skeleton-engine`.
- That sibling engine checkout included top-left sprite quad UVs.
- Game-side engine-direction Y-flip compensation was removed from survivor crop UVs, icon grid UVs, title backdrop, and full-image UI queues.
- `rg -n "flipped_y\\(|vertically_flipped_full_uv" crates/game/src` returns no matches.
- Automatic validation passed: `cargo fmt --check`, `cargo clippy --all-targets --locked -- -D warnings`, game lib tests (151 passed), release survivor build, and macOS package script.
- Packaged app smoke confirmed upright Title, InGame actor/HUD, debug boss spawn, GameOver modal, Shop power-up icons, and Level-up cards.
- 800x600 smoke confirmed Title, Settings, InGame HUD/slots, and Level-up cards.

## Applied Status - 2026-06-04

- Game dependency now resolves `skeleton-engine v2.0.0` from pinned git rev `61c09f14434910a94afc4c9ea167b3f47e0b203b`.
- `Cargo.lock` records the git source and rev, so release builds no longer require a sibling engine checkout.
- Survivor code retains engine v2 ECS compatibility by sorting entities with `Entity::index()`.
- SFX uses engine audio bus APIs (`assign_bus`, `set_bus_volume`) so user SFX volume applies consistently to file and tone effects.
- Automatic validation passed: `cargo fmt --check`, game lib tests (200 passed), release survivor build, macOS package script, and package verification.

## Applied Status - 2026-06-08

- Game dependency bumped from `skeleton-engine` v2.0.0 (`rev 61c09f14...`) to **v4.0.0**
  (`rev 60328fae...`); `Cargo.lock` records `skeleton-engine 4.0.0` (game commit `da11775`,
  2026-06-07).
- The **Color newtype (#11)** was the only v2→v4 breaking change that touched the game: `Sprite` /
  `DrawImage` `.color` assignments and reads were converted at each boundary with `Color::from(..)`
  / `Color::rgba(..)` / `.to_array()` (explicit, to avoid the simba `[T; N]: Into` ambiguity from
  rapier). `PhysicsWorld` (#13) is unaffected — the game owns a raw `PhysicsWorld` directly in the
  platformer demo (`src/main.rs`), and `PhysicsWorld::new(gravity)` is unchanged. The joint API
  (#28) is unused.
- Verification (toolchain 1.88.0, 2026-06-08): `cargo fmt --check` (stable 1.9.0 + 1.88.0),
  `cargo clippy --all-targets --locked -- -D warnings`, `cargo test -p game --lib` (200 passed),
  and the release survivor build — all green.
- Cleanups folded in during this verification (not in migration commit `da11775`, none
  migration-related): a `cargo fmt` fix for two `.to_array()` lines the commit left unwrapped, and
  32 pre-existing clippy style lints — `uninlined_format_args` ×27 and `manual_div_ceil` ×2
  (auto-fixed via `clippy --fix`), plus `manual_let_else`→`?`, `too_many_arguments` (`#[allow]` on
  a layout constructor), and `if_same_then_else` in `hud.rs` (an always-identical compact-detailed
  stat-line branch — collapsed to the single expression, which dropped the now-dead
  `TopHudLayout.detail` field; the HUD detail-level feature itself still works via the other
  `HudDetail` match arms).

## Applied Status - 2026-06-15

- Game dependency now resolves `skeleton-engine v7.0.0` from pinned git rev
  `a3369eef3ac632069b434b1ff4b0bbe8b4158466`.
- Engine v7 is primarily a renderer dependency migration: wgpu 22 -> 29, glyphon 0.6 -> 0.11,
  egui 0.29 -> 0.34, and engine MSRV 1.88 -> 1.92. Survivor code does not call the changed
  low-level `GpuContext`, `RenderTarget`, or `DebugUi` APIs.
- The engine now owns `rodio` codec features for `wav`, `vorbis`, and `mp3`, so the game removed
  its temporary direct `rodio` dependency.
- The game also removed its direct `wgpu` dependency, which previously pulled an unused duplicate
  wgpu 22 stack beside engine wgpu 29, and replaced direct `winit` imports with engine re-exports
  for `KeyCode` and `MouseButton`.
- BGM action selection no longer clones the full playlist vector each frame; it borrows the cached
  playlist and only clones the selected path when a track actually changes.

## Do Not Move Into The Engine

- `SurvivorSprite` enum and frame table.
- `SurvivorTextureHandles`.
- Weapon/passive/power-up icon indexes.
- HUD/card/shop responsive layout constants.
- Any generated survivor atlas metadata that names game concepts directly.
