# Engine Follow-ups

This game currently targets `skeleton-engine v1.0.0` at commit `78cebcf`.
Do not change the engine from this repository; use this file to collect follow-up ideas for the separate engine repo.

## AtlasSprite UV Orientation And Custom Crops

- 문제: survivor sprite sheets need vertically flipped UVs and hand-tuned crop rectangles, while `AtlasSprite` is grid-only and uses the default orientation.
- 현재 게임 코드 workaround: keep `Sprite::with_handle` or `Sprite::textured` plus manual `UvRect` for actor, effect, pickup, and UI sprites.
- 원하는 엔진 API/동작: let `AtlasSprite` accept a flip flag or a custom `UvRect` override without losing the atlas handle path.
- 우선순위: High.

## Handle-based Sprite Fallback Helper

- 문제: runtime code should prefer `Handle<ImageAsset>`, but tests and small isolated worlds often do not have texture handle resources.
- 현재 게임 코드 workaround: `survivor_textured_sprite(world, path)` checks `SurvivorTextureHandles` and falls back to `Sprite::textured(path)`.
- 원하는 엔진 API/동작: provide an engine-side helper or sprite constructor that can cleanly prefer a handle and retain a path fallback.
- 우선순위: Medium.

## Screen-anchored Sprite Rendering

- 문제: HUD and card icons are screen-space UI sprites, but they are spawned as world entities and converted through the camera each frame.
- 현재 게임 코드 workaround: `UiIconSystem` despawns/recreates icon entities from viewport and camera resources, then assigns a UI `RenderLayer`.
- 원하는 엔진 API/동작: add a screen-space sprite component or UI image primitive that can render textured quads directly in logical viewport coordinates.
- 우선순위: Medium.
