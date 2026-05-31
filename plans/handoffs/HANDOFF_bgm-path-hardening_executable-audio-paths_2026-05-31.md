# Hardened BGM asset resolution and validation for survivor audio

**Date:** 2026-05-31
**Status:** COMPLETED
**Bead(s):** none
**Epic:** Rust Survivors release hardening
**Chain:** `bgm-path-hardening` seq `1`
**Parent:** `none — first in chain`
**Prior chain:** `none — first in chain`

---

## Reference Documents

- `AGENTS.md` — repo rules, engine boundary, validation baseline, current priorities
- `CLAUDE.md` — shorter agent context and next work summary
- `docs/audio_assets.md` — audio naming and runtime lookup rules
- `docs/ENGINE_CHANGE_REQUESTS.md` — engine-boundary follow-up requests
- `docs/manual_qa_checklist.md` — manual QA destination for future smoke checks
- `docs/PHASE_LOG.md` — long historical notes; some recent asset/audio updates already landed there

## The Goal

The immediate objective in this session was to implement the previously planned hardening pass for BGM lookup and playback behavior in `rust-survivors`. The main problem was not a memory-safety bug but an execution-path and packaging safety issue: BGM lookup depended on relative paths such as `assets/audio` and `../../assets/audio`, so running the binary from a different current working directory could pull in unintended files or fail inconsistently. The user wanted the plan implemented end-to-end, including code changes, tests, packaging validation, and documentation of the temporary dependency coupling. The desired end state was a survivor binary that resolves audio relative to the executable or macOS bundle, avoids per-frame filesystem scanning, documents the temporary `rodio/mp3` coupling, and leaves the engine-side codec policy change as an explicit request rather than editing the engine directly.

## Where We Are

- The main runtime change lives in [`crates/game/src/survivor/bgm.rs`](/Users/jkl/Projects/rust-survivors/crates/game/src/survivor/bgm.rs).
- `BgmSystem` still owns `current` and `next_variants`, but now also owns `playlists: Option<[Vec<String>; 5]>`.
- BGM situations remain the same five logical slots: `bgm_title`, `bgm_ingame`, `bgm_boss`, `bgm_stageclear`, `bgm_gameover`.
- Asset extension policy is now explicitly MP3-only through `const EXTENSION: &str = "mp3";`.
- Runtime asset-root discovery no longer uses `"assets/audio"` and `"../../assets/audio"` relative to `cwd`.
- `macos_app_audio_dir_for_exe()` resolves `Contents/Resources/assets/audio` when the executable sits under `.../Contents/MacOS/...`.
- `candidate_audio_dirs_for_exe()` probes in fixed priority order:
  - macOS app bundle resources
  - executable sibling `assets/audio`
  - executable parent `assets/audio`
  - debug-only workspace fallback from `env!("CARGO_MANIFEST_DIR")/../../assets/audio`
- `resolve_audio_base_dir()` uses `std::env::current_exe()` as the only runtime anchor.
- `normalize_existing_dir()` filters non-directories and canonicalizes existing directories when possible.
- `resolve_audio_file_from_base()` returns a canonicalized absolute `String` path when a BGM variant file exists.
- `build_bgm_playlist_from_base()` constructs a per-situation `Vec<String>` from fixed variant stem lists.
- `build_cached_playlists()` memoizes all five playlists at once using `std::array::from_fn`.
- `ensure_playlists()` lazily populates the cache on first `run()`.
- When no audio root is found, `BgmSystem::run()` logs `warn!`, stops `"bgm"`, and sets `self.current = Some(target)` to prevent repeated restart churn.
- When an audio root is found but a situation playlist is empty, `BgmSystem::run()` also logs `warn!` and stops the channel.
- Steady-state playback no longer rebuilds `Vec`s or reruns `Path::exists()` on every frame.
- The repeating-playlist logic remains driven by `AudioManager::is_finished("bgm") == Some(true)`.
- The hot path now clones a selected cached path string only at the moment a track is actually advanced or entered.
- `play_bgm_file()` still maps repeating vs one-shot semantics by `bgm_playlist_advances()`.
- `bgm_repeats()` still treats only `bgm_stageclear` and `bgm_gameover` as one-shot cues.
- Title/InGame/Boss remain repeat-capable multi-track playlists.
- StageClear/GameOver remain one-shot cues even with two variants present on disk.
- The borrow-checker-sensitive part of `run()` was solved by shortening the cached-playlist borrow and cloning the selected path before mutating `next_variants`.
- Unit tests in `bgm.rs` increased from 6 to 10.
- New tests cover:
  - macOS bundle lookup priority
  - executable-relative candidate ordering
  - no `cwd`-relative fallback
  - missing asset resolution
- Existing tests still cover:
  - boss-vs-ingame track selection
  - expected two variants per situation
  - variant wrapping
  - one-shot vs repeating policy
  - stable slot mapping
- [`Cargo.toml`](/Users/jkl/Projects/rust-survivors/Cargo.toml) now contains a comment explaining that `rodio/mp3` is a temporary game-side feature-unification mechanism until engine-owned codec policy exists.
- [`crates/game/Cargo.toml`](/Users/jkl/Projects/rust-survivors/crates/game/Cargo.toml) now contains a matching comment for the direct `rodio` dependency.
- [`docs/ENGINE_CHANGE_REQUESTS.md`](/Users/jkl/Projects/rust-survivors/docs/ENGINE_CHANGE_REQUESTS.md) now has a new open request dated `2026-05-31` titled `Move Audio Codec Policy Into Engine`.
- That request explicitly tells future engine work to move codec support ownership into `skeleton-engine` instead of relying on cross-crate Cargo feature unification.
- [`docs/audio_assets.md`](/Users/jkl/Projects/rust-survivors/docs/audio_assets.md) now documents executable-relative BGM resolution rules and the rationale.
- No engine files were edited; engine-boundary instructions from `AGENTS.md` were preserved.
- No save format, serialization shape, or unsafe Rust usage was changed in this session.
- No build script was added in this repo during this session.
- `cargo test -p game --lib --locked -- --test-threads=1` now reports `174` passing tests in the current local tree.
- `cargo build -p game --bin survivor --release --locked` passes after the hardening changes.
- `bash scripts/package_macos.sh` passes after the hardening changes.
- `bash scripts/verify_macos_package.sh` passes after the hardening changes when run after packaging completes.
- The first attempt to validate package integrity in parallel failed because verification started before packaging finished; this was an execution-order issue, not a packaging defect.
- There is still no manual confirmation in this session for launching from:
  - repo root
  - packaged folder
  - `.app` launcher
  - arbitrary foreign `cwd`
- The worktree remains dirty outside this session's changes, including audio asset swaps, UI image replacements, and docs updates that predated or accompanied this work.
- `Cargo.lock` is modified in the tree from the MP3 dependency addition and remains dirty.

## What We Tried (Chronological)

1. Reviewed the last automation memory and the current dirty worktree to isolate source-impacting changes.
   - Hypothesis: only a few code/config files actually mattered for the security review and later implementation.
   - Result: source changes were concentrated in `Cargo.toml`, `Cargo.lock`, `crates/game/Cargo.toml`, and `crates/game/src/survivor/bgm.rs`; many asset/doc changes were adjacent but not central to the logic change.

2. Diffed the BGM migration and dependency updates before editing anything.
   - Hypothesis: most risk came from relative path lookup and the new MP3 decoder dependency graph rather than from unsafe code.
   - Result: confirmed `rodio` gained `mp3`, `game` added a direct `rodio` dependency, and `bgm.rs` switched from single-file lookup to variant playlists with runtime `exists()` checks.

3. Reviewed engine-side `AudioManager` behavior in the dependency checkout.
   - Hypothesis: the game-side design depended on `AudioManager::play`, `stop`, `set_volume`, and `is_finished`, so the fix had to respect that API surface.
   - Result: confirmed the engine already provides the needed playback-state API and there was no need for new engine code in this session.

4. Ran targeted `bgm` tests before changing code.
   - Command: `cargo test -p game --lib --locked bgm -- --test-threads=1`
   - Result: the pre-change BGM test subset passed, confirming the baseline behavior and giving a small regression suite to extend.

5. Planned the fix around executable-relative lookup instead of `cwd` lookup.
   - Hypothesis: `current_exe()` plus fixed relative probes could satisfy development builds, packaged folders, and macOS `.app` launches without broad fallback search.
   - Result: this became the final design; no user clarification was needed.

6. Rewrote `crates/game/src/survivor/bgm.rs` to introduce cached playlists and executable-relative discovery.
   - Change set included `macos_app_audio_dir_for_exe`, `candidate_audio_dirs_for_exe`, `resolve_audio_base_dir`, and `build_cached_playlists`.
   - Result: compile-time structure was correct, but the first draft created borrow checker conflicts in `BgmSystem::run()`.

7. Hit a Rust borrow-checker error after the first code rewrite.
   - Command: `cargo test -p game --lib --locked bgm -- --test-threads=1`
   - Errors:
     - `E0502` on `self.current`
     - `E0503` and `E0506` on `self.next_variants`
   - Cause: a mutable borrow through `self.playlist_for(target)` overlapped with later immutable and mutable accesses to other `self` fields.

8. Resolved the borrow-checker failure by removing `playlist_for()` and shortening borrows.
   - Hypothesis: cloning a selected cached path string at the point of use would be simpler and safer than trying to juggle long-lived references into `self.playlists`.
   - Result: the second draft compiled cleanly and kept the hot path free of repeated filesystem scans.

9. Added documentation and engine follow-up instead of changing the engine directly.
   - Files changed:
     - `Cargo.toml`
     - `crates/game/Cargo.toml`
     - `docs/audio_assets.md`
     - `docs/ENGINE_CHANGE_REQUESTS.md`
   - Result: repo-local behavior is hardened now, while the long-term codec-policy cleanup is explicitly queued for engine work.

10. Re-ran the BGM-only tests after the borrow-checker fix.
    - Result: `10` BGM tests passed.

11. Re-ran the full library test suite.
    - Command: `cargo test -p game --lib --locked -- --test-threads=1`
    - Result: `174 passed; 0 failed`.

12. Rebuilt the release survivor binary.
    - Command: `cargo build -p game --bin survivor --release --locked`
    - Result: passed in the local environment after the hardening changes.

13. Ran packaging and verification.
    - First attempt used parallel execution for `bash scripts/package_macos.sh` and `bash scripts/verify_macos_package.sh`.
    - Result: verification failed because it began before the package artifacts and manifests were fully written.
    - Follow-up: waited for `package_macos.sh` to finish, then reran `verify_macos_package.sh` sequentially.
    - Final result: both packaging and verification passed.

14. Updated automation memory after implementation and validation.
    - Goal: leave the security-code-monitor thread with implementation status, exact commands, and the remaining manual QA gap.
    - Result: memory now reflects both the review phase and the implementation phase.

## Key Decisions

- Use executable-relative asset discovery instead of `cwd`-relative lookup.
  - Why: `cwd`-relative lookup can load unintended files and behaves inconsistently across launch contexts.
  - Rejected alternative: keep `assets/audio` and `../../assets/audio` and only document it; rejected because that preserves the core risk.

- Give macOS bundle resources top priority over sibling/parent `assets/audio`.
  - Why: packaged apps should prefer their own sealed resources rather than any nearby loose folder.

- Keep a narrow, debug-only workspace fallback.
  - Why: it preserves ergonomic local development without reopening broad runtime search behavior in release-like contexts.
  - Rejected alternative: remove development fallback entirely; rejected because it would make local debug runs less convenient than necessary.

- Cache playlists inside `BgmSystem` instead of recalculating on each frame.
  - Why: removes repeated `Path::exists()` calls, repeated formatting, and repeated `Vec<String>` rebuilds from the hot path.

- Stop BGM and log warnings when the base dir or playlist is missing.
  - Why: failing closed is safer than accidentally consuming unrelated files from outside the package.
  - Rejected alternative: silently continue with broad filesystem fallback; rejected because it hides bad packaging/configuration.

- Keep the direct `rodio` dependency for now, but document it as temporary.
  - Why: the game currently needs MP3 decoding through engine `AudioManager`, and the engine’s public codec policy is not yet self-contained.
  - Rejected alternative: remove the direct `rodio` dependency immediately; rejected because it could break MP3 playback before engine follow-up lands.

- Record the long-term codec policy fix as an engine change request.
  - Why: `AGENTS.md` explicitly forbids editing the engine checkout from this repo and requires engine requests to be documented instead.

- Prefer cloning cached path strings in `run()` to fight the borrow checker cleanly.
  - Why: this keeps the borrow scope short and the code easy to reason about for a Rust beginner.
  - Rejected alternative: more complex interior borrowing helpers returning references into `self`; rejected after compile errors showed it made the code fragile.

- Preserve the existing logical BGM key model and one-shot/repeating semantics.
  - Why: the session goal was hardening and stability, not changing game audio behavior.

## Evidence & Data

### Validation command summary

| Command | Result | Notes |
|---|---|---|
| `cargo test -p game --lib --locked bgm -- --test-threads=1` | pass | final run: `10` BGM tests |
| `cargo test -p game --lib --locked -- --test-threads=1` | pass | final run: `174` tests total |
| `cargo build -p game --bin survivor --release --locked` | pass | release binary built successfully |
| `bash scripts/package_macos.sh` | pass | package + embedded verification completed |
| `bash scripts/verify_macos_package.sh` | pass | rerun sequentially after package creation |

### Packaging verification sequence

| Attempt | Invocation style | Result | Root cause |
|---|---|---|---|
| 1 | package + verify in parallel | fail | verify started before package output was complete |
| 2 | wait for package, then run verify | pass | correct execution order |

### BGM test coverage delta

| Area | Before | After |
|---|---:|---:|
| BGM-specific tests | 6 | 10 |
| Bundle path coverage | no | yes |
| Executable-relative ordering | no | yes |
| No-`cwd` fallback assertion | no | yes |
| Missing asset resolution | no | yes |

### Runtime path priority table

| Priority | Probe | Release-safe | Purpose |
|---:|---|---|---|
| 1 | `.app/Contents/Resources/assets/audio` | yes | packaged macOS app |
| 2 | executable sibling `assets/audio` | yes | loose packaged folder |
| 3 | executable parent `assets/audio` | yes | `dist/.../survivor` style layout |
| 4 | workspace `assets/audio` via `CARGO_MANIFEST_DIR` | debug-only | developer convenience |

### Dirty worktree context at handoff time

| Category | Examples | Session ownership |
|---|---|---|
| Code/config | `Cargo.toml`, `crates/game/Cargo.toml`, `crates/game/src/survivor/bgm.rs` | touched this session |
| Audio assets | new `assets/audio/rustsurvivors *.mp3`, deleted `bgm_*.wav` | pre-existing / adjacent |
| UI assets | updated gameover/levelup images | pre-existing / adjacent |
| Docs | `docs/ASSET_LICENSES.md`, `docs/PHASE_LOG.md`, `docs/manual_qa_checklist.md` | mostly adjacent; did not edit all of them |

### Exact compiler failure encountered mid-session

| Error code | Symptom | Trigger |
|---|---|---|
| `E0502` | immutable borrow of `self.current` while mutable borrow active | first cached-playlist draft |
| `E0503` | use of `self.next_variants[_]` while `self` still mutably borrowed | first cached-playlist draft |
| `E0506` | assignment to `self.next_variants[_]` while borrowed | first cached-playlist draft |

### Post-fix test result snapshot

```text
running 10 tests
...
test result: ok. 10 passed; 0 failed
```

### Full test result snapshot

```text
running 174 tests
...
test result: ok. 174 passed; 0 failed
```

### Diff size for key session-owned files

| File | High-level delta |
|---|---|
| `crates/game/src/survivor/bgm.rs` | `327` line net-heavy rewrite with path resolution + cache + tests |
| `Cargo.toml` | comment clarifying temporary `rodio/mp3` feature unification |
| `crates/game/Cargo.toml` | matching dependency comment |
| `docs/ENGINE_CHANGE_REQUESTS.md` | new open engine request for codec ownership |
| `docs/audio_assets.md` | new runtime path rule documentation |

### Session-owned files vs reference-only files

| Role | Files |
|---|---|
| edited | `Cargo.toml`, `crates/game/Cargo.toml`, `crates/game/src/survivor/bgm.rs`, `docs/ENGINE_CHANGE_REQUESTS.md`, `docs/audio_assets.md` |
| read-only references | `AGENTS.md`, `CLAUDE.md`, `scripts/package_macos.sh`, `scripts/verify_macos_package.sh` |

### Engine-boundary state

| Item | State |
|---|---|
| Engine code edits from this repo | none |
| New engine request added | yes |
| Completed prior engine request used | `AudioManager::is_finished("bgm")` from 2026-05-30 completion |

### Commands actually run in this session

| Order | Command |
|---:|---|
| 1 | `git status --short` |
| 2 | `git diff -- Cargo.toml crates/game/Cargo.toml crates/game/src/survivor/bgm.rs` |
| 3 | `git diff -- Cargo.lock` |
| 4 | `cargo tree -p game -i rodio` |
| 5 | `cargo test -p game --lib --locked bgm -- --test-threads=1` |
| 6 | `cargo build -p game --bin survivor --release --locked` |
| 7 | `cargo test -p game --lib --locked -- --test-threads=1` |
| 8 | `bash scripts/package_macos.sh` |
| 9 | `bash scripts/verify_macos_package.sh` |

### Raw warning/failure excerpt from the bad packaging order

```text
FAILED open or read
shasum: ... No such file or directory
```

Interpretation:
- This was not a bad manifest.
- This was not a missing asset in the repo.
- Verification had raced ahead of packaging.

## Code Analysis

- `BgmSystem` still implements `System` directly and is registered early in `src/bin/survivor.rs`, before most gameplay systems.
- `BGM_TRACK_COUNT` is hardcoded to `5`; slot order is stable and mirrored by `BGM_KEYS`.
- `bgm_variant_stems()` is still the authority for the two MP3 variant filenames per logical BGM key.
- `candidate_audio_dirs_for_exe()` returns concrete existing directories only; nonexistent directories are filtered out early.
- `resolve_audio_base_from_exe()` deliberately picks the first existing candidate and does not merge multiple roots.
- The debug-only workspace fallback is gated by `cfg!(debug_assertions)`, so release builds should never fall back to workspace assets through that path.
- `build_cached_playlists()` only resolves once per system lifetime, which means runtime asset changes after start are not reloaded; this is intentional for stability and hot-path cost control.
- `resolve_audio_file_from_base()` canonicalizes the chosen file path, so the path passed into `AudioManager::play()` is absolute when canonicalization succeeds.
- `run()` now has two explicit failure modes:
  - no base dir found
  - base dir found but zero variants for a logical key
- In both failure modes the system stops `"bgm"` and marks the current key to avoid thrashing.
- The repeating-playlist branch advances only when `audio.is_finished("bgm") == Some(true)` and `variant_count > 1`.
- The one-shot branch for StageClear/GameOver still uses the same cached playlist mechanism but disables repeat in `play_bgm_file()`.
- The borrow-checker-safe design now depends on cloning the chosen path string out of `self.playlists` before writing `next_variants`.

## Files Changed

### Source code

- `crates/game/src/survivor/bgm.rs` — replaced `cwd`-relative file lookup with executable-relative asset resolution, added cached playlists, warning-and-stop failure behavior, and expanded tests.

### Config

- `Cargo.toml` — added a comment documenting temporary `rodio/mp3` feature unification for engine `AudioManager`.
- `crates/game/Cargo.toml` — added a matching comment next to the direct `rodio` dependency.

### Docs

- `docs/ENGINE_CHANGE_REQUESTS.md` — added an open engine request for moving audio codec ownership into `skeleton-engine`.
- `docs/audio_assets.md` — documented the executable-relative BGM resolution rules and the rationale.

## User Feedback & Preferences (REQUIRED — never omit)

- The user first requested a security-oriented review of recently changed code and explicitly said not to edit code unless instructed.
- The review was to focus on security issues, memory safety risks, unsafe Rust usage, dependency/build-script risks, input/file handling, serialization/save-data risks, and behavioral regressions.
- The requested report language for the review was concise Korean.
- After the review, the user asked for an improvement-direction plan rather than immediate code edits.
- The user then explicitly requested implementation of that BGM hardening plan.
- The user’s plan emphasized:
  - remove `cwd`-dependent audio lookup
  - reduce MP3 decoder coupling risk
  - cache BGM variants to avoid hot-path filesystem work
- The user wanted engine changes handled through `docs/ENGINE_CHANGE_REQUESTS.md` rather than by editing the engine directly.
- The user wanted packaging/docs updated together with the code change where relevant.
- The user expected the listed validation commands to actually be run, not just suggested.
- The user then explicitly asked for a structured handoff using the `handoff` skill.

## Where We're Going

- 1. Run manual launch-path QA for survivor BGM from repo root, packaged folder, `.app`, and a foreign `cwd`.
- 2. If all manual cases pass, update `docs/manual_qa_checklist.md` and optionally `docs/PHASE_LOG.md` with concrete smoke results.
- 3. Coordinate the engine-side follow-up in `docs/ENGINE_CHANGE_REQUESTS.md` so codec support is owned by `skeleton-engine`.
- 4. After engine codec ownership lands, remove the temporary direct `rodio` dependency from the game repo and rerun the full validation suite.
- 5. Continue broader release hardening and visual QA priorities already listed in `AGENTS.md` / `CLAUDE.md`.

## Risks & Blockers

- Manual runtime QA for multiple launch contexts was not performed in this session, so the new path policy is validated by tests and packaging logic but not by actual interactive launches.
- The repo remains in a dirty state with many unrelated asset/doc changes; future sessions must avoid reverting unrelated work.
- MP3 decoding still depends on temporary Cargo feature unification until the engine request is implemented.
- Cached playlists are built once per process; if someone expects live asset replacement during a running debug session, that behavior no longer exists for BGM without restart.

## Open Questions

- Do the manual launch-path smoke tests reveal any discrepancy between macOS Finder `.app` launch and terminal launch from inside `dist/macos`?
- Should `docs/manual_qa_checklist.md` be updated immediately after manual QA, or should those notes wait for a larger release-hardening pass?
- Once engine codec ownership is fixed, is the direct `rodio` dependency still needed anywhere else in `crates/game`, or can it be dropped entirely?

## Quick Start for Next Session

```bash
# Restore context
git status -s

# Reference docs
sed -n '1,220p' AGENTS.md
sed -n '1,220p' docs/audio_assets.md
sed -n '1,220p' docs/ENGINE_CHANGE_REQUESTS.md

# Key files to read first (not exhaustive — explore adjacent code too)
sed -n '1,340p' crates/game/src/survivor/bgm.rs
sed -n '1,220p' docs/audio_assets.md
sed -n '1,220p' docs/manual_qa_checklist.md
sed -n '1,220p' docs/ENGINE_CHANGE_REQUESTS.md

# Evidence / data files
sed -n '1,240p' plans/handoffs/HANDOFF_bgm-path-hardening_executable-audio-paths_2026-05-31.md

# Verify current state
cargo test -p game --lib --locked -- --test-threads=1
cargo build -p game --bin survivor --release --locked
bash scripts/package_macos.sh
bash scripts/verify_macos_package.sh

# Next action
Perform manual BGM smoke tests from all four launch contexts and write the results into docs/manual_qa_checklist.md.
```
