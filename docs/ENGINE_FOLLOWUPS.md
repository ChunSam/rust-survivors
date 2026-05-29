# Engine Follow-ups

This file keeps historical context for engine follow-ups that were needed before
the game migrated to `skeleton-engine` commit `0e01b0f`.

Status: the items below were implemented in the engine and consumed by the game
on 2026-05-28, with the sprite quad UV orientation follow-up consumed on 2026-05-29.

## Sprite Quad Top-left UV Orientation

- 구현: shared sprite quad top edge now uses `v = 0.0` and bottom edge uses `v = 1.0`.
- 게임 적용: survivor crop, icon, title, and full-image UI paths no longer need engine-direction Y-flip compensation.
- `flipped_y()` remains available only for intentional vertical mirroring.

## AtlasSprite UV Orientation And Custom Crops

- 구현: `UvRect::from_pixels(...)`, `flipped_x()`, `flipped_y()`, and `AtlasSprite` custom `UvRect` override support.
- 게임 적용: survivor crop tables stay in game code, but manual UV struct construction was replaced with engine helpers.

## Handle-based Sprite Fallback Helper

- 구현: `Sprite::textured_with_handle(path, Option<Handle<ImageAsset>>)`.
- 게임 적용: `SurvivorTextureHandles` still owns the game-specific lookup, while sprite construction uses the engine helper.

## Screen-anchored Sprite Rendering

- 구현: `DrawImage` and `UiImageQueue` for logical viewport coordinate textured quads.
- 게임 적용: HUD, level-up, and shop icon/background drawing now queues `DrawImage` values instead of spawning world entities.
