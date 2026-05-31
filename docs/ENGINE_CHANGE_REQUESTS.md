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

## 2026-05-31 - Move Audio Codec Policy Into Engine

Engine request prompt:

Context:
- Repository: skeleton-engine
- Triggering game repo: rust-survivors
- Affected game files:
  - Cargo.toml
  - crates/game/Cargo.toml
  - crates/game/src/survivor/bgm.rs

Problem:
- `rust-survivors` now ships MP3-only BGM and plays it through engine `AudioManager`.
- The actual MP3 decoder feature is currently enabled from the game repo by adding a direct `rodio` dependency so Cargo feature unification reaches the engine.
- This works today but is brittle: game code is forced to know an engine implementation detail, and future `rodio` version/feature changes in the engine could silently break MP3 playback.

Expected engine behavior/API:
- `skeleton-engine` should own its supported audio codec policy directly.
- At minimum, the engine's own `rodio` dependency should explicitly enable the codecs required by its public audio API consumers.
- Preferred outcome:
  - engine exposes/document its supported formats,
  - game repos no longer need a direct `rodio` dependency just to make `AudioManager::play(...)` decode a format.

Reproduction:
1. In `rust-survivors`, remove the direct `rodio` dependency or the `mp3` feature from workspace dependency configuration.
2. Build and run survivor with MP3-only BGM assets.
3. Observe that playback support depends on whether the engine's transitive `rodio` feature set happens to include MP3.

Validation:
- In `skeleton-engine`, enable/document codec support in the engine-owned `rodio` dependency.
- Verify an engine audio smoke test can decode MP3 through `AudioManager`.
- Run:
  - `cargo test`
  - `cargo test -p game --lib --locked -- --test-threads=1`
  - `cargo build -p game --bin survivor --release --locked`

Notes:
- After completion, `rust-survivors` should remove the temporary direct `rodio` dependency if it is no longer needed outside the engine boundary.

## Completed Requests

## 2026-05-29 - Expose Audio Channel Playback State

Engine request prompt:

Context:
- Repository: skeleton-engine
- Triggering game repo: rust-survivors
- Affected game files:
  - crates/game/src/survivor/bgm.rs

Problem:
- `rust-survivors` now has two BGM tracks per gameplay situation and can alternate tracks when entering a situation.
- The game cannot cleanly advance to the next track when the current non-looping track ends because `AudioManager` does not expose channel playback state.
- The needed state is already available internally through the channel `Sink`, but game code cannot query it without an engine API.
- Directly editing the external `skeleton-engine` checkout from this repo is not allowed.

Expected engine behavior/API:
- Add a public `AudioManager` API for channel playback state.
- Suggested API:
  - `is_playing(channel: &str) -> bool`
  - or `is_finished(channel: &str) -> Option<bool>`
- The API should let game code distinguish at least:
  - channel does not exist / has never played,
  - channel exists and still has queued audio,
  - channel exists and has reached the end.
- This should support playlist-style BGM where the game starts the next track after the current one finishes.

Reproduction:
1. In `rust-survivors`, add two BGM files for a situation such as `assets/audio/rustsurvivors stageclear1.mp3` and `assets/audio/rustsurvivors stageclear2.mp3`.
2. Play one through `AudioManager::play("bgm", path, false)`.
3. Observe that game code has no public way to know when the channel has finished so it can play the next variant.

Validation:
- Add engine tests for the new channel playback-state API.
- Verify missing channels are reported predictably.
- Verify a short non-looping tone/file reports unfinished while queued and finished after playback drains.
- Run:
  - `cargo test`
  - `cargo test -p game --lib --locked -- --test-threads=1`
  - `cargo build -p game --bin survivor --release --locked`

Notes:
- Completed 2026-05-30: `skeleton-engine` now exposes `AudioChannelState`, `AudioManager::playback_state`, `AudioManager::is_finished`, and `AudioManager::is_playing`.
- `rust-survivors` now uses `AudioManager::is_finished("bgm")` to advance repeating multi-track BGM playlists.
- The game should not depend on engine-private `Sink` internals.

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
- Before completion, `rust-survivors` worked around this by dropping handles whose renderer key did not match the requested texture path.

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
- Completed 2026-05-29: `skeleton-engine` now aliases canonical and requested texture cache keys without a public API change.
- `rust-survivors` removed the `survivor_texture_handle` renderer-key compatibility workaround and now passes loaded `SurvivorTextureHandles` handles directly.
