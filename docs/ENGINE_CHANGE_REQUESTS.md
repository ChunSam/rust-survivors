# Engine Change Requests

This file tracks engine-side work that `rust-survivors` needs but should not implement directly from this repository.

Policy:

- Do not directly edit external `skeleton-engine` checkouts while working in `rust-survivors`.
- When game work exposes an engine limitation or bug, add a request prompt here.
- Keep the prompt specific enough that a separate engine task can execute it without rediscovering the context.
- After the engine work lands externally, update this repo's dependency or lockfile if needed and rerun survivor validation.

## Request Template

```text
## YYYY-MM-DD - <short title>

Engine request prompt:

Context:
- Repository: skeleton-engine
- Triggering game repo: rust-survivors
- Affected game files:
  - crates/game/src/...

Problem:
- <What currently fails or forces awkward game-side code?>

Expected engine behavior/API:
- <What should the engine provide or fix?>

Reproduction:
1. <Command or in-game steps>
2. <Observed result>

Validation:
- <Engine tests or examples to run>
- cargo test -p game --lib --locked -- --test-threads=1
- cargo build -p game --bin survivor --release --locked

Notes:
- <Constraints, compatibility concerns, or visual QA notes>
```

## Open Requests

## 2026-05-29 - Unify Image Texture Cache Keys

Engine request prompt:

Context:
- Repository: skeleton-engine
- Triggering game repo: rust-survivors
- Affected game files:
  - crates/game/src/survivor/sprites.rs
  - crates/game/src/survivor/hud.rs
  - crates/game/src/survivor/title_visual.rs
  - crates/game/src/survivor/ui_icons.rs

Problem:
- `App::load_image(path)` queues `path` for GPU upload as the original relative string.
- `AssetServer::load_image(path)` stores `Handle<ImageAsset>::path()` as a canonical absolute path when the file exists.
- `DrawImage::texture_key()` and sprite rendering prefer `image_handle.path()` over the fallback texture path.
- The sprite renderer texture cache is keyed by the relative upload path, so canonical handle paths miss the cache and render with the white fallback texture.
- `rust-survivors` currently works around this by only passing a handle when `handle.path() == requested_path`.

Expected engine behavior/API:
- `pending_textures`, `AssetServer` handles, `SpriteRenderer::texture_cache`, and `DrawImage::texture_key()` should use one consistent texture key policy.
- Either canonicalize all texture keys before GPU upload/cache lookup, or keep renderer keys as the original logical asset path and store canonical filesystem paths separately.
- `Sprite::textured_with_handle(path, Some(handle))` and `DrawImage::textured_with_handle(path, Some(handle))` should not render white when the image loaded successfully.

Reproduction:
1. In `rust-survivors`, load an existing PNG through `App::load_image("assets/textures/survivor/ui/ingame_label_lv_ko.png")`.
2. Queue it through `DrawImage::textured_with_handle(...)`.
3. Observed result before the game-side workaround: image renders as a white rectangle because the handle path is canonical but the GPU texture cache key is relative.

Validation:
- Add an engine test that verifies `AssetServer::load_image(relative_existing_path).path()` resolves to a key that `SpriteRenderer` can find after `App::load_image(relative_existing_path)`.
- Add a UI image render smoke or cache-key unit test for `DrawImage::textured_with_handle`.
- Run:
  - `cargo test`
  - `cargo test -p game --lib --locked -- --test-threads=1`
  - `cargo build -p game --bin survivor --release --locked`

Notes:
- After the engine fix lands, remove the `rust-survivors` workaround in `survivor_texture_handle` and restore always passing matching `SurvivorTextureHandles` handles.
