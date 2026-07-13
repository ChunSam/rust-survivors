# Fixed the Windows DX12 build failure and the magenta-screen asset resolution bug, added Windows zip packaging

**Date:** 2026-07-13
**Status:** COMPLETED (PR #26 open, unmerged)
**Bead(s):** none
**Epic:** Rust Survivors release hardening
**Chain:** `standalone-16232dbe` seq `1`
**Parent:** `none — first in chain`
**Prior chain:** `none — first in chain`

---

## Related Handoffs

- `plans/handoffs/HANDOFF_bgm-path-hardening_executable-audio-paths_2026-05-31.md` — hardened BGM lookup to be executable-relative and explicitly banned `cwd`-relative fallback. **Separate work stream, not a parent, but it DIRECTLY conflicts with a change made this session** (see Key Decisions → "Reintroduced a cwd fallback in bgm.rs" and Risks). Read it before touching `bgm.rs`.
- `plans/handoffs/HANDOFF_stage-tilemaps_textured-backgrounds_2026-06-12.md` — most recent prior handoff; unrelated (tilemap backgrounds).

## Reference Documents

- `AGENTS.md` / `CLAUDE.md` — repo rules, engine boundary (never edit the engine checkout from this repo), validation baseline. Both updated this session with the Windows packaging commands.
- `docs/ENGINE_CHANGE_REQUESTS.md` — engine-boundary request tracker; two new open requests dated 2026-07-13.
- `docs/ENGINE_ASSET_LOADING_REQUEST.md` — **new this session**, the standalone prompt to hand to engine work.
- `docs/audio_assets.md` — documents executable-relative BGM resolution rules; now partially stale (see Risks).

## The Goal

The user asked for a Windows `.exe` build. That surfaced two independent defects that had never been seen because this project is developed on macOS: the Windows release build did not compile at all (the DX12 backend of `wgpu-hal` failed with type errors), and once it did compile, the game rendered a solid magenta window instead of the title screen. Both were fixed. The user then asked how to share the build with someone else, which exposed a third gap — the executable is useless without its 94MB of assets — so a Windows zip packaging script was added, and the two root causes were written up as engine change requests to be handed to the `skeleton-engine` repo. The end state: a working Windows release build, a shareable zip, an engine prompt document, and PR #26 open against `main`.

## Where We Are

- Branch `fix/windows-build-and-asset-root` is pushed with 4 commits; **PR #26 is open and not merged**: https://github.com/ChunSam/rust-survivors/pull/26
- Worktree is clean as of the handoff (all session work committed).
- `Cargo.lock` has ONE hand-edited line: `gpu-allocator 0.28.0`'s dependency entry changed from `"windows 0.58.0"` to `"windows 0.62.2"`. This is what makes the Windows build compile at all.
- New file `crates/game/src/survivor/asset_root.rs` (176 lines, 6 unit tests). Public API: `resolve_asset_root() -> Option<PathBuf>` and `set_working_dir_to_asset_root()`.
- Private helpers in that file: `macos_bundle_resources(exe)`, `push_unique(out, dir)`, `asset_root_candidates(exe, cwd)`, `holds_assets(dir)`. Constant `ANCESTOR_DEPTH = 4`.
- `crates/game/src/survivor/mod.rs` — added `pub mod asset_root;` and `pub use asset_root::set_working_dir_to_asset_root;`.
- `crates/game/src/bin/survivor.rs` — `main()` calls `set_working_dir_to_asset_root()` immediately after `env_logger::init()` and before `App::new()`, i.e. before any `app.load_image(...)` call.
- `crates/game/src/survivor/bgm.rs` — `resolve_audio_base_dir()` now falls back to `normalize_existing_dir(PathBuf::from("assets/audio"))` (working-directory-relative) when executable-relative resolution finds nothing. The `warn!` string changed to "checked executable-relative paths and the working directory".
- `scripts/package_windows.ps1` (69 lines) — release build → `dist/windows/RustSurvivors/` (exe + `assets/` + `ASSET_LICENSES.md` + `audio_assets.md`) → SHA256 manifest → zip → verify. Mirrors `scripts/package_macos.sh`.
- `scripts/verify_windows_package.ps1` (68 lines) — required files, manifest hash re-check, zip entry checks (exe, atlas png, weapons.ron, at least one mp3), and a hard failure if any zip entry contains a backslash.
- `docs/ENGINE_ASSET_LOADING_REQUEST.md` (157 lines) — Request A (asset root + byte-sourced assets + loud failures) and Request B (single `windows` crate version on Windows targets).
- `docs/ENGINE_CHANGE_REQUESTS.md` — two summary entries dated 2026-07-13 pointing at the standalone prompt.
- `CLAUDE.md` and `AGENTS.md` — Commands section now lists `powershell -File scripts/package_windows.ps1` and `...verify_windows_package.ps1`.
- `crates/game/src/survivor/sfx.rs` was NOT edited but is fixed by the same change: `AUDIO_DIR = "assets/audio"` (`sfx.rs:17`) builds `format!("{AUDIO_DIR}/{key}.{ext}")` (`sfx.rs:23`), i.e. cwd-relative — so SFX was silently broken in exactly the same launch contexts as the textures, and the cwd anchoring repairs it without touching the file.
- Test count: **213 passing** (was 207 before this session; +6 from `asset_root.rs`). `cargo fmt --all --check` clean, `cargo clippy -p game --lib --all-targets` exit 0.
- Release binary: `target/release/survivor.exe`, 26MB. Build time from cold-ish cache: 1m18s.
- Package artifact: `dist/windows/RustSurvivors-windows-x64.zip`, 103.5 MB, 79 entries, all under a single `RustSurvivors/` top-level folder.
- Runtime is verified working from a foreign working directory and from an extracted zip (see Evidence).
- The game runs on the **Vulkan** backend on this machine (`wgpu_hal::vulkan::adapter` in the logs) — the DX12 code still has to *compile*, which is why the dependency conflict blocked the build even though DX12 is never used at runtime here.
- `gh` CLI is installed at `C:\Program Files\GitHub CLI\gh.exe` (v2.96.0) and authenticated (ChunSam, scopes `gist`, `read:org`, `repo`, `workflow`). It is on the **Machine** PATH but this Claude Code session inherited a stale environment, so `gh` is not resolvable by name in-session. Call it by full path, or restart the terminal.

## What We Tried (Chronological)

1. **Ran the documented release build command.**
   - Command: `cargo build -p game --bin survivor --release --locked`
   - Result: FAILED, exit 101. `error: could not compile 'wgpu-hal' (lib) due to 10 previous errors` (E0277, E0308).

2. **First hypothesis (WRONG): the `windows` 0.62.2 patch release broke wgpu-hal's DX12 bindings.**
   - The errors mentioned `windows-core 0.62.2` and `CanInto<ID3D12Heap>`, and `wgpu-hal` declares `windows = "0.62"` (any patch), so a bad patch looked plausible.
   - Downgraded via a cascade of `cargo update --precise`: `windows` 0.62.2→0.62.1, then `windows-collections` 0.3.2→0.3.1, `windows-future` 0.3.2→0.3.1, `windows-numerics` 0.3.1→0.3.0 (each blocked the next until downgraded), finally `windows-core` 0.62.2→0.62.1.
   - Rebuilt: **identical 10 errors**. Hypothesis dead. Reverted with `git checkout Cargo.lock`.
   - Lesson: the version cascade cost ~5 tool calls; reading the *first* error would have killed the hypothesis immediately.

3. **Read the actual first errors instead of the tail of the log.**
   - `error[E0308]: mismatched types ... expected 'ID3D12Device', found 'Direct3D12::ID3D12Device'` at `wgpu-hal-29.0.3/src/dx12/suballocation.rs:83`.
   - Two identically-named types from *different crate versions*. That is the signature of a duplicate-dependency problem, not a bad patch.

4. **Traced the dependency graph.**
   - `cargo tree -i windows@0.58.0 --target x86_64-pc-windows-msvc` → consumers: `gilrs-core 0.6.8` and `gpu-allocator 0.28.0`.
   - `wgpu-hal 29.0.3` Cargo.toml declares `windows = "0.62"` and `gpu-allocator = "0.28"`.
   - `gpu-allocator 0.28.0` Cargo.toml declares `windows = ">=0.53, <=0.62"` — a range spanning many incompatible minors, so Cargo satisfies it by *reusing an already-activated node*.
   - Confirmed in the lockfile: `gpu-allocator 0.28.0` → `windows 0.58.0`, while `wgpu-hal 29.0.3` → `windows 0.62.2`. They hand D3D12 handles to each other. That is the bug.

5. **Tried `cargo update --recursive -p gpu-allocator@0.28.0` — made it WORSE.**
   - Result: it removed `windows 0.58.0` entirely, but re-latched `gpu-allocator` onto **`windows 0.54.0`** (another old node already in the graph). Still mismatched with 0.62. Reverted.

6. **Tried to remove the old `windows` nodes at the source.**
   - `windows 0.54.0` consumers: `cpal 0.15.3` + `gpu-allocator 0.28.0`. `cpal 0.15.3` comes from `rodio 0.19.0`, required by BOTH `game` and `skeleton-engine`.
   - `cpal` latest is 0.18.1, but `rodio 0.19` requires `^0.15` → cannot bump `cpal` without bumping `rodio`, which is an engine-repo change.
   - `gilrs 0.11.2` / `gilrs-core 0.6.8` are already the latest, and `gilrs-core` also uses a wide range (`windows = ">=0.44, <=0.62"`), so it just latches onto whatever old node exists.
   - `wgpu-hal 29.0.3` is the latest 29.x (30.0.0 exists but needs the engine on wgpu 30).
   - Conclusion: every "clean" fix is an engine-repo change. Blocked for this session.

7. **Fixed it in the lockfile: pinned gpu-allocator's `windows` edge to 0.62.2.**
   - Hand-edited `Cargo.lock`, changing `"windows 0.58.0"` → `"windows 0.62.2"` inside the `gpu-allocator 0.28.0` dependency block only. `0.62.2` is inside gpu-allocator's declared `>=0.53, <=0.62` range, so `--locked` accepts it.
   - Result: `cargo build -p game --bin survivor --release --locked` PASSED in 1m18s. `target/release/survivor.exe`, 26MB.

8. **User reported the window renders solid magenta (screenshot).**
   - Grepped the game crate for magenta / `1.0, 0.0, 1.0` / `255, 0, 255` → **no hits**. So the color comes from the engine.
   - Engine: `renderer/texture.rs:36` logs `texture load failed ({path}): {e}, using magenta fallback`; `asset/image_loading.rs:146` has `magenta_fallback()`, used at :135 and :141 on failure. Magenta = missing texture, by design.

9. **Ran the exe from the repo root with `RUST_LOG=warn` — and saw NO texture errors.**
   - Only BGM warnings, spamming every frame: `BGM audio root not found for key bgm_title; checked executable-relative paths only`.
   - This briefly looked like the magenta was NOT a load failure. It was a false lead caused by testing from the wrong directory.

10. **Reproduced properly by running from a foreign working directory.**
    - `cd C:/Users/jkl92 && RUST_LOG=warn E:/projects/rust-survivors/target/release/survivor.exe`
    - Result: `image file read failed 'assets/textures/survivor/survivor_atlas.png': 지정된 경로를 찾을 수 없습니다. (os error 3)` — and the same for every one of the ~30 textures.
    - Root cause confirmed: asset paths are **cwd-relative**. The user had launched the exe from somewhere other than the repo root (Explorer double-click), so every texture load failed and the full-screen title backdrop became the magenta fallback.

11. **Checked whether the fix could be "resolve each path at load time".**
    - `App::load_image(path)` (`app/assets.rs:22`) pushes the path string into `pending_textures`, and `Sprite::textured(path)` (`components.rs:77`) stores that same string as `Arc<str>` — **the path IS the texture identity**.
    - So resolving paths only at the load site would desync the keys from the dozens of `Sprite::textured(CONST_PATH)` call sites in `sprites.rs`, `ui_icons.rs`, `hud.rs`. Rejected.

12. **Chose to anchor the process working directory instead — after checking it was safe.**
    - Save files: `meta.rs:146` uses `save::save_path(APP_NAME, SAVE_FILE)`; engine `save.rs:76-84` roots that at `dirs::data_dir()` (absolute). So changing cwd does not move save files. Verified before writing any code.

13. **Wrote `asset_root.rs` and wired it into `main()`.**
    - Candidate order: macOS bundle `Contents/Resources` → executable dir + ancestors (depth 4) → working dir + ancestors (depth 4). First candidate that actually contains `assets/` wins, then `canonicalize`.
    - Deliberately did NOT add a `CARGO_MANIFEST_DIR` dev fallback: ancestor-walking from `target/release/survivor.exe` already reaches the repo root, so debug and release behave identically.

14. **Extended the fix to BGM.**
    - BGM searched executable-relative paths ONLY, so it found nothing even when run from the repo root (that was the warning spam in step 9). Added a working-directory fallback, which now resolves because `main()` anchors cwd to the asset root.

15. **Validated: fmt, clippy, 213 tests, release build, then ran the binary from `C:/Users/jkl92`.**
    - Log now shows `asset root: \\?\E:\projects\rust-survivors` and zero texture/audio errors.
    - Captured screenshots with a PowerShell + `System.Drawing.CopyFromScreen` script (launching via `Start-Process -WorkingDirectory C:\Users\jkl92`): at 3s the title screen renders fully (backdrop, logo plaque, START/CHARACTER/STAGE/SHOP/ACHIEVEMENTS/SETTINGS buttons); at 8s the in-game view renders (tilemap, player, zombies, HUD, weapon/passive slots).

16. **User asked: "if I share only the exe, the same problem happens, right?" — yes.**
    - Measured `assets/`: 94MB total (audio 57MB, textures 33MB, fonts 4.5MB, data 24K, shaders 4K).
    - Checked the engine for a bytes-based image API: none — `load_image`/`load_image_async` take paths only. So `include_bytes!`/`rust-embed` single-exe is not possible without engine work.
    - Presented three options (zip / self-extracting single exe / engine memory-loading). User chose zip.

17. **Wrote `scripts/package_windows.ps1` — four failures before it passed.**
    - (a) `Sort-Object -Culture ordinal` → Windows PowerShell 5.1 does not support `-Culture`. Replaced with `[Array]::Sort($relatives, [StringComparer]::Ordinal)` to match the macOS script's `LC_ALL=C sort`.
    - (b) `Compress-Archive` → the verify step failed with `zip is missing assets/textures/survivor/survivor_atlas.png`. Inspecting the archive showed entries named `RustSurvivors\assets\...` — **PS 5.1 writes zip entries with backslash separators**, which extract as literal `assets\...` filenames on macOS/Linux.
    - (c) `[ZipFile]::CreateFromDirectory` → **same backslash bug** (.NET Framework uses the platform separator). Confirmed by listing entries.
    - (d) Hand-built entries via `ZipArchive.CreateEntry("RustSurvivors/$relative")` → `Unable to find type [System.IO.Compression.ZipArchiveMode]` until `Add-Type -AssemblyName System.IO.Compression` was added alongside `System.IO.Compression.FileSystem`.
    - Final run: PASSED. 79 entries, 103.5 MB.
    - Note: the verify script CAUGHT bug (b) — writing the checks first paid for itself immediately.

18. **Verified the actual recipient scenario.**
    - Extracted the zip to a temp dir, then ran `RustSurvivors/survivor.exe` with cwd = `C:\Windows`.
    - Log: `asset root: ...\unzip\RustSurvivors`, no texture or audio errors. This is the exact case that used to render magenta.

19. **Wrote `docs/ENGINE_ASSET_LOADING_REQUEST.md`, then fact-checked it against the engine source.**
    - Corrected two claims before committing: `AudioManager::play` lives in `src/audio/playback.rs:42` (not `src/audio.rs`), and `AssetServer::load_state(handle)` **is** public (`asset/image_loading.rs:58`) — the accurate criticism is that the *renderer's* texture path only `warn!`s and swaps in magenta, so nothing forces a game to notice.

20. **Committed in 4 logical commits, pushed the branch, opened PR #26.**
    - `gh` was not resolvable by name; found it at `C:\Program Files\GitHub CLI\gh.exe` and used the full path.
    - Also hit a permission denial on `powershell -ExecutionPolicy Bypass -File ...` (security classifier) — invoked the script directly with `& "path.ps1"` instead.

## Key Decisions

- **Pin `gpu-allocator`'s `windows` edge to 0.62.2 in `Cargo.lock` rather than chase a "proper" fix.**
  - Why: every clean fix (bump `rodio` so `cpal` stops pinning `windows 0.54`; bump the engine to wgpu 30) is a `skeleton-engine` change, and `AGENTS.md` forbids editing the engine from this repo. The pin is inside gpu-allocator's declared range, so it is a legal resolution, not a hack of the dependency contract.
  - Rejected: downgrading the `windows` patch version (tried, did nothing); `cargo update --recursive` (tried, latched onto an even older `windows 0.54`); vendoring `gpu-allocator` through `[patch.crates-io]` (heavier, adds a vendored crate to the repo).
  - Recorded as engine Request B so the pin can be deleted later.

- **Anchor the process working directory to the asset root, instead of resolving each asset path.**
  - Why: `Sprite::textured(path)` uses the path string as the texture's identity, so the load site and every draw site must agree on the exact same string. Resolving at the load site alone would silently desync dozens of call sites.
  - Rejected: rewriting every path constant to an absolute path at startup (touches `sprites.rs`, `ui_icons.rs`, `hud.rs`, `icons.rs` and risks key mismatches); teaching the engine to resolve roots (correct, but it is engine work — filed as Request A).

- **Executable-relative candidates take priority over working-directory candidates.**
  - Why: a shipped build must never silently read a stray `assets/` from wherever the user happened to launch it. Packaged layout wins; cwd is only a dev/fallback convenience.

- **No `CARGO_MANIFEST_DIR` dev fallback in `asset_root.rs`** (unlike `bgm.rs`, which keeps a debug-only one).
  - Why: ancestor-walking from `target/release/survivor.exe` already reaches the repo root, so the fallback would be dead code — and omitting it keeps debug and release behaving identically, which is what you want when debugging a packaging bug.

- **Reintroduced a working-directory fallback in `bgm.rs` — this partially reverses the 2026-05-31 decision.**
  - Why: BGM found no audio at all in a dev release run (executable-relative only, and assets do not sit next to `target/release/survivor.exe`). With cwd now anchored to the asset root by `main()`, a cwd fallback is deterministic rather than arbitrary.
  - **Tension:** `HANDOFF_bgm-path-hardening_executable-audio-paths_2026-05-31.md` decided "use executable-relative discovery instead of cwd-relative lookup" precisely to avoid consuming unrelated files, and `docs/audio_assets.md` documents that rule. Executable-relative still wins when it resolves, so packaged builds are unaffected — but the policy is now weaker than documented. See Risks.

- **Zip packaging over a self-extracting single exe.**
  - Why: the engine cannot load assets from memory, so a single exe would mean embedding 94MB and extracting to `%LOCALAPPDATA%` on first run — slower, weirder, and it does not remove the underlying engine limitation. The zip is the honest fix now; the single exe is filed as engine Request A.

- **Write zip entries by hand instead of using `Compress-Archive` / `CreateFromDirectory`.**
  - Why: both write backslash entry names on Windows PowerShell 5.1, producing archives that extract to literal `assets\...` filenames on macOS/Linux. The verify script now fails the package if any entry contains a backslash, so this cannot regress silently.

- **Branch + PR instead of pushing to `main`.**
  - Why: global convention — never push directly to a protected default branch.

## Evidence & Data

### Build failure — the errors that identified the duplicate `windows` crate

| Error | Location | Meaning |
|---|---|---|
| `E0308: expected 'ID3D12Device', found 'Direct3D12::ID3D12Device'` | `wgpu-hal-29.0.3/src/dx12/suballocation.rs:83` | Same type name, two different crate versions |
| `E0277: ResourceCategory: From<&D3D12_RESOURCE_DESC> not satisfied` | `suballocation.rs:299, :338, :377` | `D3D12_RESOURCE_DESC` came from the other `windows` |
| `E0277: &ID3D12Heap: Param<ID3D12Heap, InterfaceType> not satisfied` | `suballocation.rs:306, :345, :384` | `gpu-allocator`'s heap type ≠ `wgpu-hal`'s heap type |

Total: 10 errors, `cargo` exit code 101.

### The `windows` crate graph (Windows target, before the fix)

| `windows` version | Pinned by | Declared requirement |
|---|---|---|
| 0.62.2 | `wgpu-hal 29.0.3` | `windows = "0.62"` (hard) |
| 0.58.0 | `gilrs-core 0.6.8`, **`gpu-allocator 0.28.0`** | gilrs: `">=0.44, <=0.62"`, gpu-allocator: `">=0.53, <=0.62"` (both ranges → latch onto existing nodes) |
| 0.54.0 | `cpal 0.15.3` (hard), `gpu-allocator 0.28.0` after `--recursive` | cpal pins it; `rodio 0.19.0` requires `cpal ^0.15` |
| 0.52.0 | `gilrs-core`, `gpu-allocator 0.26.0` (old wgpu 22 in tree) | — |

### Fix attempts for the build failure

| # | Attempt | Result |
|---:|---|---|
| 1 | `cargo update --precise` downgrade chain: `windows` 0.62.2→0.62.1, `windows-collections` 0.3.2→0.3.1, `windows-future` 0.3.2→0.3.1, `windows-numerics` 0.3.1→0.3.0, `windows-core` 0.62.2→0.62.1 | FAIL — identical 10 errors. Wrong hypothesis. Reverted. |
| 2 | `cargo update --recursive -p gpu-allocator@0.28.0` | WORSE — removed `windows 0.58`, re-latched gpu-allocator onto `windows 0.54`. Reverted. |
| 3 | Bump `cpal` / `rodio` / `gilrs` / `wgpu` | BLOCKED — all are engine-repo changes (`rodio 0.19` requires `cpal ^0.15`; `wgpu-hal 29.0.3` is latest 29.x). |
| 4 | Hand-pin `gpu-allocator 0.28.0` → `windows 0.62.2` in `Cargo.lock` | **PASS** — build succeeded in 1m18s. |

### Runtime verification matrix (the magenta bug)

| Working directory | Before fix | After fix |
|---|---|---|
| Repo root `E:\projects\rust-survivors` | Textures OK, **BGM warning spam every frame** | OK, no warnings |
| `C:\Users\jkl92` (foreign) | **Every texture: `os error 3`** → solid magenta window | `asset root: \\?\E:\projects\rust-survivors`, clean |
| Extracted zip, cwd `C:\Windows` | n/a (no package existed) | `asset root: ...\unzip\RustSurvivors`, clean |

### Validation gates

| Command | Result |
|---|---|
| `cargo fmt --all --check` | pass |
| `cargo clippy -p game --lib --all-targets` | pass (exit 0) |
| `cargo test -p game --lib --locked -- --test-threads=1` | pass — **213 passed; 0 failed** (207 before; +6 from `asset_root.rs`) |
| `cargo build -p game --bin survivor --release --locked` | pass — `target/release/survivor.exe`, 26MB |
| `scripts/package_windows.ps1` | pass — 79 entries, 103.5 MB zip |
| `scripts/verify_windows_package.ps1` | pass |

### Zip packaging iteration history

| Attempt | Approach | Failure |
|---:|---|---|
| 1 | `Sort-Object -Culture ordinal` | PS 5.1 has no `-Culture`: "ordinal is an invalid culture identifier" |
| 2 | `Compress-Archive` | Entries named `RustSurvivors\assets\...` — verify caught it as "zip is missing ..." |
| 3 | `[ZipFile]::CreateFromDirectory` | Same backslash entries (.NET Framework uses platform separator) |
| 4 | `ZipArchive.CreateEntry("RustSurvivors/$rel")` | `Unable to find type [ZipArchiveMode]` — needed `Add-Type -AssemblyName System.IO.Compression` |
| 5 | Same + both assemblies loaded | **PASS** |

### Asset payload (why a bare exe cannot work)

| Directory | Size |
|---|---:|
| `assets/audio` | 57 MB |
| `assets/textures` | 33 MB |
| `assets/fonts` | 4.5 MB |
| `assets/data` | 24 KB |
| `assets/shaders` | 4 KB |
| **total** | **94 MB** |

Zip output: `dist/windows/RustSurvivors-windows-x64.zip` — 103.5 MB, 79 entries, single `RustSurvivors/` root folder.

### The three asset-path schemes that existed before this session (the real shape of the bug)

| Consumer | How it resolved paths | Worked when launched from repo root? | Worked when launched elsewhere? |
|---|---|---|---|
| Textures (`sprites.rs`, `icons.rs`, `ui_icons.rs` path consts → `App::load_image`) | cwd-relative (`assets/textures/...`) | yes | **no** → magenta |
| SFX (`sfx.rs:17` `AUDIO_DIR = "assets/audio"`) | cwd-relative | yes | **no** (silent) |
| BGM (`bgm.rs` `resolve_audio_base_dir`) | executable-relative only | **no** (warning spam) | no |

No launch context made all three work. That is why the fix had to unify them rather than patch one.

### New tests in `asset_root.rs` (all pure — no filesystem fixtures except the last)

| Test | Asserts |
|---|---|
| `macos_bundle_resources_dir_is_the_first_candidate` | `.app/Contents/MacOS/exe` → `Contents/Resources` ranks first |
| `plain_executable_yields_no_bundle_candidate` | a non-bundle exe path does not synthesize a Resources dir |
| `executable_ancestors_reach_the_workspace_root_of_a_dev_build` | `/work/rust-survivors/target/release/survivor` → `/work/rust-survivors` is a candidate |
| `executable_candidates_precede_working_directory_candidates` | packaged layout wins over cwd |
| `candidates_are_deduplicated_when_exe_and_cwd_overlap` | no duplicate entries |
| `resolve_finds_the_repo_asset_root_from_the_test_working_directory` | real FS: resolves the repo root under `cargo test` |

### Visual QA reproduction (how the screenshots were taken)

The game window cannot be captured by the terminal, so verification used a PowerShell script:
`Start-Process -FilePath target\release\survivor.exe -WorkingDirectory C:\Users\jkl92 -PassThru`, sleep,
then `System.Drawing.Graphics.CopyFromScreen` over `[System.Windows.Forms.Screen]::PrimaryScreen.Bounds`,
save PNG, `Stop-Process`. Sleep 3s captures the title screen; 8s lands in-game. Scripts lived in the session
scratchpad (ephemeral) — re-create as needed; the pattern is what matters.

### Engine source on this machine (read-only reference)

| Item | Path |
|---|---|
| Pinned engine checkout (rev `a3369ee` = skeleton-engine 7.0.0) | `C:\Users\jkl92\.cargo\git\checkouts\skeleton-engine-a43429513ac9c1de\a3369ee\src` |
| Older checkout also present | `...\skeleton-engine-a43429513ac9c1de\1c19873\src` |

Never edit these (AGENTS.md engine boundary) — read them to verify claims before writing engine requests.
Two claims in the first draft of `ENGINE_ASSET_LOADING_REQUEST.md` were wrong until checked against this source.

### Engine API facts (verified against the pinned checkout, rev `a3369ee`)

| Fact | Location |
|---|---|
| `App::load_image(path)` pushes into `pending_textures` | `src/app/assets.rs:22-28` |
| `Sprite::textured(path)` stores the path as `Arc<str>` — path IS the texture key | `src/components.rs:77` |
| Renderer reads the file, logs `warn!`, substitutes magenta 1×1 | `src/renderer/texture.rs:28-37` |
| `magenta_fallback()` for failed image assets | `src/asset/image_loading.rs:135, :141, :146` |
| `AssetServer::load_state(handle)` **is public** | `src/asset/image_loading.rs:58` |
| `AudioManager::play(channel, path, repeat)` — path-based | `src/audio/playback.rs:42` |
| `save_path()` uses `dirs::data_dir()` (absolute — unaffected by cwd) | `src/save.rs:76-84` |
| No bytes-based image or audio loading API exists | — |

### Commits on `fix/windows-build-and-asset-root`

| Hash | Summary |
|---|---|
| `bdc606e` | deps: pin gpu-allocator to windows 0.62 so the DX12 build compiles |
| `4e25103` | Resolve assets relative to the executable, not the working directory |
| `d4b87b9` | Add Windows zip packaging scripts |
| `9ad8553` | docs: file engine requests for asset loading and the windows crate conflict |

Diff vs `main`: 11 files, +534 / -21.

### Environment facts

| Item | Value |
|---|---|
| Toolchain | cargo 1.96.0 / rustc 1.96.0 (2026-05-25) |
| Runtime GPU backend | Vulkan (`wgpu_hal::vulkan::adapter`) — DX12 must still compile |
| Pre-existing benign warning | `egui_wgpu::renderer: Detected a linear (sRGBA aware) framebuffer Bgra8UnormSrgb` |
| `gh` CLI | `C:\Program Files\GitHub CLI\gh.exe` v2.96.0, authed as ChunSam |
| `gh` PATH | Machine PATH: **yes**. User PATH: no. This session's env: **stale** → call by full path |
| `bd` (beads) | not installed on this machine |

## Code Analysis

- `asset_root::asset_root_candidates(exe, cwd)` is pure path arithmetic — no filesystem access — so the 5 ordering/dedup tests need no fixtures. `resolve_asset_root()` is the only function that touches the disk (`holds_assets()` → `dir.join("assets").is_dir()`).
- `ANCESTOR_DEPTH = 4` means the executable's directory plus 3 parents. For `<root>/target/release/survivor.exe` the repo root is 2 levels up, so there is one level of headroom.
- `macos_bundle_resources(exe)` only matches when the exe's parent is `MacOS` and its grandparent is `Contents` — the same shape check `bgm.rs::macos_app_audio_dir_for_exe()` uses. The two functions are now near-duplicates; a future cleanup should have `bgm.rs` call `asset_root`.
- `set_working_dir_to_asset_root()` logs `error!` (not `warn!`) when no `assets/` is found anywhere, and names the magenta fallback in the message — so the next person who sees a magenta screen gets told why in the log.
- `bgm.rs::resolve_audio_base_dir()` structure changed: it no longer early-returns on `current_exe()` failure (`.ok()?`), because that would have skipped the new cwd fallback. It now builds `from_exe` as an `Option` and chains `.or_else(...)`.
- The BGM test `no_relative_cwd_fallback_is_used` (`bgm.rs:432`) only exercises `resolve_audio_base_from_exe(&exe, false)` — the *executable* resolver. It therefore still passes even though `resolve_audio_base_dir()` now does have a cwd fallback. **The test name now overstates what it guarantees.**
- `BgmSystem::ensure_playlists()` re-attempts resolution on every frame while `playlists` is `None`, which is why the failure produced one warning per frame rather than one warning total.
- `verify_windows_package.ps1` parses the manifest with `-split '\s+\./', 2`, matching the `<sha256>  ./<path>` format the macOS script produces via `shasum -a 256`, so the two manifests stay diffable.

## Files Changed

### Source code

- `crates/game/src/survivor/asset_root.rs` — **new.** Asset root resolution + `set_working_dir_to_asset_root()`. 6 unit tests.
- `crates/game/src/survivor/mod.rs` — registered the module, re-exported the entry point.
- `crates/game/src/bin/survivor.rs` — call `set_working_dir_to_asset_root()` first thing in `main()`.
- `crates/game/src/survivor/bgm.rs` — working-directory fallback in `resolve_audio_base_dir()`; updated warning text.

### Config / lockfile

- `Cargo.lock` — one line: `gpu-allocator 0.28.0` now depends on `windows 0.62.2` instead of `0.58.0`.

### Scripts

- `scripts/package_windows.ps1` — **new.** Build → stage → SHA256 manifest → zip → verify.
- `scripts/verify_windows_package.ps1` — **new.** File presence, manifest hashes, zip entry checks, backslash-entry rejection.

### Docs

- `docs/ENGINE_ASSET_LOADING_REQUEST.md` — **new.** Engine prompt: Request A (asset root, byte-sourced assets, loud failures), Request B (single `windows` version).
- `docs/ENGINE_CHANGE_REQUESTS.md` — two new open request entries dated 2026-07-13.
- `CLAUDE.md`, `AGENTS.md` — Windows packaging commands added to the Commands section.

## User Feedback & Preferences (REQUIRED — never omit)

- Opening request: "exe 파일 빌드 해줘" — just build the Windows executable.
- On seeing the broken render, the user sent a **screenshot** and said "화면이 정상적이 아니야. 수정해줘". They diagnose visually — screenshots are the medium they trust, which is why the fix was verified with screenshots rather than only with logs.
- The user immediately reasoned ahead to distribution: "만약에 공유를 위해 EXE 파일만 보내줄 경우에 asset이 없으면 동일한 문제가 생기겠네?" — they think about the recipient's environment, not just their own machine. Confirming the failure mode was expected before proposing options.
- "zip 패키징 스크립트로 진행해줘. 그리고 이번 문제를 해결할 프롬프트 작성해서 md 파일로 만들어줘. 엔진작업에 보낼거야." — they run engine work as a *separate* task and want a self-contained prompt document to hand over. Follow the `docs/ENGINE_CHANGE_REQUESTS.md` template; do not edit the engine from this repo.
- "커밋 푸쉬 해줘" — commit and push only when asked, which they did explicitly.
- "gh path 설정 안되어 있어 확인해봐" — when a tool appears missing, the user expects investigation of the environment (is it installed? is it just PATH?) rather than a workaround or an install. The right answer was "it is installed and on the Machine PATH; this session's env is stale."
- Global conventions in force (`~/.claude/CLAUDE.md`): user-facing reports in **Korean**, all code/docs/commit messages in **English**; never pipe a gate's output in a way that hides its exit code; commit/push only when asked; never push to a protected default branch — branch and open a PR.

## Where We're Going

1. **Decide the `bgm.rs` cwd-fallback question** (see Open Questions). Either (a) rename `no_relative_cwd_fallback_is_used` and update `docs/audio_assets.md` to document the new, weaker policy, or (b) drop the fallback and have `bgm.rs` call `asset_root::resolve_asset_root()` so audio and textures share one resolution path. **(b) is the cleaner end state** and would also delete the duplicated macOS-bundle logic.
2. **Get PR #26 reviewed and merged**: https://github.com/ChunSam/rust-survivors/pull/26
3. **Hand `docs/ENGINE_ASSET_LOADING_REQUEST.md` to the engine repo.** Request A (asset root + byte-sourced assets) and Request B (`rodio` bump so `cpal` stops pinning `windows 0.54`) are independent and can be done in either order.
4. **After engine Request B lands:** delete the `Cargo.lock` pin, re-resolve, confirm only one `windows` version remains on `x86_64-pc-windows-msvc`.
5. **After engine Request A lands:** delete `crates/game/src/survivor/asset_root.rs` and its call in `main()`; optionally embed textures for a true single-file build.
6. **Run the macOS package scripts on a Mac** to confirm the cwd anchoring did not disturb the `.app` launcher (it should be a no-op — the launcher already `cd`s to `Contents/Resources`, and bundle Resources is the first candidate).
7. Resume the pre-existing priorities in `CLAUDE.md` (manual visual QA, low-res/high-DPI icon placement, localization coverage, final art/audio).

## Risks & Blockers

- **The `Cargo.lock` pin is fragile.** Any `cargo update` that re-resolves `gpu-allocator` or `windows` can silently revert it, and the Windows build breaks again with 10 confusing type errors. Nothing guards it — no test, no CI. If a future session sees `wgpu-hal` DX12 type mismatches, check `gpu-allocator`'s `windows` edge in the lockfile FIRST.
- **The BGM cwd fallback weakens a policy that a prior session deliberately hardened.** `docs/audio_assets.md` still documents executable-relative-only resolution, and the test named `no_relative_cwd_fallback_is_used` no longer means what its name says. Not a regression for packaged builds (executable-relative still wins), but the docs/tests and the code now disagree.
- **`asset_root`'s ancestor walk could find an unrelated `assets/` directory** up to 4 levels above the executable or the working directory. Low risk in practice, unbounded in principle.
- **No Windows CI.** This entire class of bug (DX12 compile, cwd-relative assets) is invisible from macOS, which is exactly how it reached a release build. Both engine requests ask for a Windows build job.
- **Distribution is a 103.5 MB zip**, not a single file. Only engine Request A changes that.
- The macOS packaging path was not re-run this session (no Mac available), so the cwd-anchoring change is unverified against `.app` launches.

## Open Questions

- Should `bgm.rs` keep its own resolver at all, or delegate entirely to `asset_root::resolve_asset_root()`? The duplicated macOS-bundle detection is a smell, and one resolution path would make the policy statable in one place.
- Should the `no_relative_cwd_fallback_is_used` test be renamed, deleted, or made true again? It currently passes for a reason that no longer reflects the system's behavior.
- Is a Windows CI job worth adding to this repo, or does it belong solely in `skeleton-engine` (where the DX12/`windows` conflict actually originates)?
- Once engine Request A lands, do we want a true single-exe build (embed 94MB) or keep the zip? Embedding audio may not be worth it.

## Quick Start for Next Session

```bash
# Restore context
git log --oneline -5
git status -s
# PR: https://github.com/ChunSam/rust-survivors/pull/26  (branch fix/windows-build-and-asset-root)

# Engine source (READ ONLY — never edit; AGENTS.md engine boundary)
ls C:/Users/jkl92/.cargo/git/checkouts/skeleton-engine-a43429513ac9c1de/a3369ee/src

# Reference docs
sed -n '1,60p' AGENTS.md
sed -n '1,160p' docs/ENGINE_ASSET_LOADING_REQUEST.md     # the prompt to hand to engine work
sed -n '1,40p' docs/ENGINE_CHANGE_REQUESTS.md            # two new open requests, 2026-07-13

# Key files to read first
sed -n '1,100p' crates/game/src/survivor/asset_root.rs   # the workaround; delete once the engine owns roots
sed -n '119,180p' crates/game/src/survivor/bgm.rs        # audio resolution + the new cwd fallback
sed -n '425,440p' crates/game/src/survivor/bgm.rs        # the test whose name is now misleading
sed -n '1,70p' scripts/package_windows.ps1

# Prior decision that this session partially reversed — READ BEFORE TOUCHING bgm.rs
sed -n '154,175p' plans/handoffs/HANDOFF_bgm-path-hardening_executable-audio-paths_2026-05-31.md

# Verify current state
cargo fmt --all --check
cargo test -p game --lib --locked -- --test-threads=1     # expect 213 passed
cargo build -p game --bin survivor --release --locked
# Windows package (PowerShell, invoke directly — do NOT use -ExecutionPolicy Bypass):
#   & scripts/package_windows.ps1

# Confirm the magenta bug stays fixed (run from a foreign cwd)
cd /c/Windows && RUST_LOG=info /e/projects/rust-survivors/target/release/survivor.exe
# expect: "asset root: ..." and zero "image file read failed" lines

# Next action
Resolve the bgm.rs cwd-fallback conflict: have bgm.rs delegate to asset_root::resolve_asset_root()
(preferred), or rename no_relative_cwd_fallback_is_used and update docs/audio_assets.md to match
the new policy. Then get PR #26 merged.
```
