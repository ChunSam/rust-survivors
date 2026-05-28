# Engine Migration Notes

This document tracks rust-survivors work that should remain game-specific after the engine follow-ups are implemented in `skeleton-engine`.

## What Moved Into The Engine

The following capabilities are general engine features and should be consumed from `engine` once the dependency is updated:

- `UvRect::from_pixels(...)`, `flipped_x()`, and `flipped_y()` for custom crop rectangles and UV orientation.
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

Suggested migration:
- Replace manual `UvRect` construction with `UvRect::from_pixels(...).flipped_y()` where crop rectangles are pixel-based.
- Replace `flipped_grid_uv(...)` helpers with `UvRect::from_grid(...).flipped_y()` where sheets are uniform grids.
- Keep `SurvivorSprite`, `IconSource`, `passive_index`, and `powerup_index` in game code.

### Texture Handle Lookup Policy

Keep `SurvivorTextureHandles` or an equivalent game resource in game code.

Why:
- The engine cannot know which game texture resource corresponds to a string path.
- The lookup table is tied to rust-survivors asset loading and test setup.

Suggested migration:
- Update `survivor_textured_sprite(world, path)` to call:

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

Suggested migration:
- Replace per-frame world-entity spawn/despawn for UI icons with `UiImageQueue`.
- Keep `hud_icon_layout`, `levelup_icon_center`, `shop_icon_center`, and icon source selection in game code.
- Push `DrawImage` instances using logical viewport coordinates:

```rust
world.resource_mut::<UiImageQueue>().unwrap().push(
    DrawImage::textured_with_handle(x, y, size, size, texture, handle)
        .with_uv(uv)
        .with_z(z),
);
```

After this migration, `UiIconSprite`, `despawn_previous_icons`, camera `screen_to_world` conversion, and UI `RenderLayer` assignment should no longer be needed for HUD/card/shop icons.

## Migration Order

1. Update the `engine` dependency to the commit that includes the follow-up APIs.
2. Switch sprite UV helpers to `UvRect::from_pixels(...).flipped_y()` and `UvRect::from_grid(...).flipped_y()`.
3. Switch `survivor_textured_sprite` to `Sprite::textured_with_handle`.
4. Migrate `UiIconSystem` from spawned world sprites to `UiImageQueue`.
5. Run visual QA for title, gameplay, HUD slots, level-up cards, shop icons, pickups, and effects at default size and 800x600.

## Applied Status - 2026-05-28

- Game dependency updated to `skeleton-engine` commit `0e01b0f383202acea4d2ff606be4c62f73371471`.
- `sprites.rs` now uses engine UV flip/crop helpers and `Sprite::textured_with_handle`.
- `ui_icons.rs` now queues screen-space `DrawImage` values through `UiImageQueue`; `UiIconSprite` and per-frame icon entity despawn are removed.
- Automatic validation passed: `cargo fmt --check`, game lib tests (144 passed), all-target check, and release survivor build.

## Do Not Move Into The Engine

- `SurvivorSprite` enum and frame table.
- `SurvivorTextureHandles`.
- Weapon/passive/power-up icon indexes.
- HUD/card/shop responsive layout constants.
- Any generated survivor atlas metadata that names game concepts directly.
