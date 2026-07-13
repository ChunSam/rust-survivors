# Engine Request - Asset Loading Independent of the Working Directory

Standalone prompt for a `skeleton-engine` task. Written from `rust-survivors` on 2026-07-13 after a
Windows release build rendered nothing but the engine's magenta fallback. Follows the request format
in `docs/ENGINE_CHANGE_REQUESTS.md`.

Two requests. Request A is the one that caused the incident. Request B is a separable dependency fix
that blocked the Windows release build entirely; it can be done independently.

---

## Request A - Asset roots and byte-sourced assets

Engine request prompt:

Context:

- Repository: skeleton-engine (pinned here at `7.0.0`, rev `a3369ee`)
- Triggering game repo: rust-survivors
- Affected engine files:
  - `src/app/assets.rs` (`App::load_image`, `App::load_image_async`)
  - `src/asset/image_loading.rs`, `src/asset/async_loading.rs` (`AssetServer::load_image`)
  - `src/renderer/texture.rs` (file read + magenta 1x1 fallback)
  - `src/audio/playback.rs` (`AudioManager::play(channel, path, repeat)`)
- Affected game files:
  - `crates/game/src/survivor/asset_root.rs` (workaround added for this bug)
  - `crates/game/src/bin/survivor.rs`
  - `crates/game/src/survivor/sprites.rs`, `ui_icons.rs`, `bgm.rs`, `sfx.rs`

Problem:

- Every engine image API takes a path (`App::load_image("assets/textures/...")`), and `Sprite::textured(path)`
  uses that same path string as the texture identity. The renderer reads the file with `std::fs` when it
  materializes the texture, so a relative path is resolved against the process working directory.
- Consequence: the executable only works when launched with a specific working directory. Launching it any
  other way - double-clicking it in a file manager, a desktop shortcut, `cd /elsewhere && game.exe` - fails
  every texture load. The engine substitutes its magenta 1x1 fallback for each one, and because the title
  screen is a full-screen textured quad the entire window turns solid magenta. Nothing else in the game
  signals that anything is wrong.
- The failure is quiet by default. `App::load_image` feeds two paths: the `AssetServer` (whose
  `load_state(handle)` does report `AssetLoadState::Failed`) and `pending_textures`, which the renderer turns
  into GPU textures in `renderer/texture.rs` - and that path only logs a `warn!` before swapping in the magenta
  1x1. Nothing aborts, nothing bubbles up, and a game that does not poll `load_state` on every handle just
  renders magenta. The engine already knows the load failed; it simply does not insist that anyone notice.
- There is no way to source an asset from memory. `include_bytes!` / `rust-embed` cannot be used, so the game
  cannot ship as a single distributable file: `assets/` (94 MB here) must always travel next to the binary.
  The same limit applies to audio - `AudioManager::play` takes a path, so BGM/SFX cannot be embedded either.
- Workaround now living in the game repo: `crates/game/src/survivor/asset_root.rs` resolves the directory that
  actually contains `assets/` (macOS bundle `Contents/Resources`, then the executable's directory and its
  ancestors, then the working directory and its ancestors) and calls `std::env::set_current_dir` at startup.
  This is engine responsibility leaking into game code, and every game built on the engine will need its own
  copy of it.

Expected engine behavior/API:

1. Engine-owned asset root. Resolve relative asset paths against a root the engine determines, not against the
   process working directory. Suggested shape:
   - default root resolution: executable directory, then a macOS bundle's `Contents/Resources`, so a packaged
     build works no matter how it is launched;
   - `App::set_asset_root(impl Into<PathBuf>)` (or `AssetServer::set_root`) to override, for dev builds that
     run from a repo checkout;
   - path-based APIs keep their current signatures - only the resolution changes.
2. Byte-sourced assets, so a game can embed what it ships:
   - `App::load_image_bytes(key: impl Into<String>, bytes: &'static [u8]) -> Handle<ImageAsset>` (or a
     `Cow<'static, [u8]>` variant), registering the decoded image under `key`;
   - `Sprite::textured(key)` must resolve such a key exactly as it resolves a path today, so game code does not
     have to distinguish embedded from on-disk assets;
   - the audio equivalent, e.g. `AudioManager::play_bytes(channel, key, bytes, repeat)` - `rodio` already decodes
     from `Cursor<Vec<u8>>`.
3. Loud failures. A missing texture should be an `error!`, not a `warn!`, and the load state should be reachable
   for renderer-loaded textures - e.g. `AssetServer::failed_assets() -> &[(String, String)]` - so a game can log,
   show a diagnostic, or refuse to start rather than presenting a magenta screen. An opt-in strict mode that
   panics on a failed load in debug builds would be even better.

Reproduction:

1. `cargo build -p game --bin survivor --release --locked`
2. `cd C:\ && E:\projects\rust-survivors\target\release\survivor.exe` (any directory that is not the repo root)
3. Observed, before the game-side workaround: the log fills with
   `image file read failed 'assets/textures/survivor/survivor_atlas.png': The system cannot find the path specified. (os error 3)`
   for every texture, and the window renders solid magenta.
4. Same binary launched from the repo root renders correctly - the only difference is the working directory.

Validation:

- Engine: a test that loads an image with the working directory set somewhere unrelated to the executable and
  asserts the asset resolves; a test that loads an image from bytes and draws it through `Sprite::textured(key)`.
- Game side, after the engine lands:
  - delete `crates/game/src/survivor/asset_root.rs` and its `set_working_dir_to_asset_root()` call in
    `crates/game/src/bin/survivor.rs`;
  - `cargo test -p game --lib --locked -- --test-threads=1`
  - `cargo build -p game --bin survivor --release --locked`
  - launch the executable from an unrelated working directory and confirm the title screen renders.

Notes:

- Keep the path-based API; these additions should be additive so existing games do not have to migrate.
- The macOS packaging scripts (`scripts/package_macos.sh`) rely on assets living in `Contents/Resources` and the
  launcher shell script `cd`-ing there before exec. Engine-side root resolution would let that launcher shim go away.
- Request 2 (byte sources) is what unlocks single-file distribution. Without it, sharing the game means shipping a
  zip - which is what `scripts/package_windows.ps1` now produces.

---

## Request B - One `windows` crate version on Windows targets (separable)

Engine request prompt:

Context:

- Repository: skeleton-engine
- Triggering game repo: rust-survivors
- Affected engine files: `Cargo.toml` (the `rodio` dependency)

Problem:

- The Windows release build of the game does not compile at all: `wgpu-hal 29.0.3` fails with 10 errors in its
  DX12 backend, e.g. in `src/dx12/suballocation.rs`:
  - `error[E0308]: mismatched types ... expected ID3D12Device, found Direct3D12::ID3D12Device`
  - ``error[E0277]: the trait bound `&ID3D12Heap: Param<ID3D12Heap, InterfaceType>` is not satisfied``
- Root cause is two `windows` crate versions in one dependency graph. `wgpu-hal 29.0.3` requires `windows 0.62`.
  `gpu-allocator 0.28.0` (which `wgpu-hal` calls into) declares a wide range, `windows = ">=0.53, <=0.62"`, so
  Cargo satisfies it by reusing whatever `windows` node is already activated. The engine's `rodio 0.19` pulls
  `cpal 0.15.3`, which pins `windows 0.54`; `gpu-allocator` latches onto that node, and the D3D12 types it hands
  `wgpu-hal` then come from a different crate version than the ones `wgpu-hal` expects.
- The game repo is currently working around this by hand-pinning `gpu-allocator`'s `windows` edge to `0.62.2` in
  `Cargo.lock`. That is a lockfile patch, not a fix: it is fragile across `cargo update` and it does not help
  anyone else building the engine on Windows.

Expected engine behavior/API:

- The engine should not drag an old `windows` version into the graph on Windows targets. Concretely: bump `rodio`
  (0.20+/0.21, whichever pulls `cpal >= 0.16`) so `cpal` no longer pins `windows 0.54`, and confirm that a fresh
  resolve leaves exactly one `windows` version for the DX12 path.
- More generally, it would help if the engine documented/asserted that the DX12 backend and the audio stack agree
  on a single `windows` version - e.g. a CI job that builds for `x86_64-pc-windows-msvc`.

Reproduction:

1. On Windows, from a clean `Cargo.lock`: `cargo build -p game --bin survivor --release --locked`
2. `cargo tree -i windows@0.54.0 --target x86_64-pc-windows-msvc` shows `cpal 0.15.3` and `gpu-allocator 0.28.0`
   sharing the same node, while `wgpu-hal 29.0.3` sits on `windows 0.62.x`.
3. `wgpu-hal` fails to compile with the type mismatches above.

Validation:

- `cargo build --target x86_64-pc-windows-msvc` in the engine repo (a plain `cargo check` on Windows is enough to
  surface it, since the DX12 backend is a default feature).
- In the game repo, after the engine bump: drop the `Cargo.lock` hand-pin, re-resolve, and confirm
  `cargo build -p game --bin survivor --release --locked` succeeds and only one `windows` version remains.

Notes:

- The engine and game are developed on macOS, where the DX12 backend never compiles - which is why this stayed
  hidden. A Windows build in CI would have caught it.
- `rodio` is also the subject of the open 2026-05-31 request in `docs/ENGINE_CHANGE_REQUESTS.md` (MP3 codec policy).
  Both touch the same dependency; consider doing them together.
