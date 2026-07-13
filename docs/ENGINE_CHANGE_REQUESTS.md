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

_None._

## Completed Requests

## 2026-07-13 - Asset Loading Independent of the Working Directory

Resolved in the engine 2026-07-14 — `skeleton-engine` v0.126.0 (PR ChunSam/skeleton-engine#359).

Original prompt: `docs/ENGINE_ASSET_LOADING_REQUEST.md` (Request A).

What the request asked for:

- Engine image APIs are path-based and the renderer resolved those paths against the process working directory,
  so launching the executable from anywhere but the repo root failed every texture load and the window rendered
  solid magenta.
- Asked for an engine-owned asset root (executable-relative, macOS bundle aware), byte-sourced images/audio so a
  game can embed assets and ship a single file, and for failed texture loads to be loud rather than silent.

What the engine shipped:

- `engine::asset_path::resolve` resolves a relative asset path against a root the engine determines — a macOS
  bundle's `Contents/Resources`, then the executable's directory and its ancestors, then the working directory —
  taking the first candidate under which the file actually exists. Executable-derived candidates are searched
  before the working directory, so a packaged build cannot pick up a stray `assets/` from wherever it was
  launched. `App::set_asset_root` pins an explicit root.
- Loud failures: a failed load is now an `error!` naming the roots that were searched, is recorded in
  `App::asset_failures()`, and `App::set_strict_assets(true)` makes it panic at the load instead of falling back.
- **Not shipped: byte-sourced images** (`App::load_image_bytes`, for `include_bytes!` single-file distribution).
  The audio half already existed (`AudioManager::play_bytes`). It was judged not worth it for *this* game either
  way: `assets/` is 93 MB (56 MB of it audio), so embedding would mean a ~100 MB executable. The zip produced by
  `scripts/package_windows.ps1` remains the right way to hand this game to someone.

Status in this repo: **no change, deliberately.** This repo pins `skeleton-engine` at rev `a3369ee` (v7.0.0),
which predates the engine's 0.x version reset — adopting the fix would mean a migration across 100+ releases,
which is not worth paying on a paused project. The workaround in
`crates/game/src/survivor/asset_root.rs` (`set_current_dir` to the resolved asset root at startup) therefore
stays, and it works. Remove it only if this game is ever migrated to a current engine.

## 2026-07-13 - One `windows` Crate Version on Windows Targets

Resolved in the engine 2026-07-14 — `skeleton-engine` v0.125.0 (PR ChunSam/skeleton-engine#358).

Original prompt: `docs/ENGINE_ASSET_LOADING_REQUEST.md` (Request B).

What the request asked for:

- The Windows release build did not compile: the engine's `rodio 0.19` pulled `cpal 0.15.3`, which pins
  `windows 0.54`; `gpu-allocator 0.28.0` (wide range `>=0.53, <=0.62`) reused that node while `wgpu-hal 29.0.3`
  uses `windows 0.62`, so the DX12 backend failed with D3D12 type mismatches.
- Asked the engine to bump `rodio` so `cpal` no longer pins an old `windows`, and to build for
  `x86_64-pc-windows-msvc` in CI.

What the engine shipped:

- `rodio` 0.19 → 0.22.2 (`cpal` 0.15.3 → 0.17.3), which removes `windows 0.54` from the graph entirely — the MSVC
  target now resolves exactly one `windows` version (0.62.2), so `gpu-allocator` and `wgpu-hal` agree.
- A `Build (Windows / DX12)` CI job on `windows-latest`, now a *required* check. It compiles the DX12 backend (no
  engine CI job ever had, which is why this could hide) and asserts the invariant directly: exactly one `windows`
  version on the MSVC target.
- Worth noting for the record: the conflict was confirmed live on the engine's then-current `main` (v0.124.0), not
  just on this repo's old pin. The engine's lockfile happened to have resolved `gpu-allocator` onto the good
  `windows` node, but nothing held it there.

Status in this repo: **no change, deliberately** — same reasoning as Request A. The `Cargo.lock` hand-pin of
`gpu-allocator`'s `windows` edge to `0.62.2` stays and keeps working. Drop it only if this game is ever migrated
to a current engine, at which point it becomes redundant.


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
- Completed 2026-06-15: the pinned `skeleton-engine v7.0.0` dependency owns `rodio` codec features with `wav`, `vorbis`, and `mp3` enabled in the engine crate.
- Game cleanup completed 2026-06-15: removed the temporary direct `rodio` dependency from `rust-survivors`.

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
