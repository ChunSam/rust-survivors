# Engine Dependency Workflow

Updated: 2026-05-29

## Current Setup

`crates/game` now depends on the local engine checkout instead of the remote git dependency:

```toml
engine = { package = "skeleton-engine", path = "../../../skeleton-engine" }
```

The path is relative to `crates/game/Cargo.toml`, so it resolves to:

```text
/Users/jkl/Projects/skeleton-engine
```

`Cargo.lock` no longer records a git source for `skeleton-engine`; Cargo resolves the engine package from the local checkout.

## Why This Changed

This project is currently in a phase where game work can expose engine API or rendering issues. A local path dependency gives immediate feedback after engine work lands externally:

- Updates in `/Users/jkl/Projects/skeleton-engine` are reflected by the next rust-survivors build.
- No lockfile update is needed for each local engine revision while the path dependency is active.
- The game can validate externally completed engine changes through its real survivor workload.

Important: while working in `rust-survivors`, do not directly edit the external `skeleton-engine` checkout. If an engine change is needed, leave a request prompt in `docs/ENGINE_CHANGE_REQUESTS.md`.

## Tradeoffs

Benefits:

- Fast local feedback after an engine fix has been applied externally.
- Easier to validate engine APIs against survivor gameplay.
- Useful when both repositories are available on the same machine.

Costs:

- Builds now require a sibling `skeleton-engine` checkout at `/Users/jkl/Projects/skeleton-engine`.
- CI or another developer machine must either have the same relative checkout layout or switch back to a git dependency.
- The exact engine revision is no longer visible from this repo's `Cargo.lock`.

## Recommended Workflow

Use the path dependency for validating the game against the current local engine checkout:

```bash
cargo check -p game --bin survivor --locked
cargo test -p game --lib --locked -- --test-threads=1
cargo build -p game --bin survivor --release --locked
```

When game work appears to require an engine change:

1. Do not directly modify `/Users/jkl/Projects/skeleton-engine` from this repo.
2. Add a dated request prompt to `docs/ENGINE_CHANGE_REQUESTS.md`.
3. Include the problem, expected engine behavior/API, reproduction steps, affected game files, and validation commands.
4. Continue only with game-side changes that do not require engine edits.
5. After the engine change lands externally, run the rust-survivors validation commands above.
6. If CI or release packaging needs a remote dependency, switch `crates/game/Cargo.toml` back to a pinned git dependency with an explicit `rev`.
7. Update `Cargo.lock` and run the same validation again.

Suggested shared/release form:

```toml
engine = {
  package = "skeleton-engine",
  git = "https://github.com/ChunSam/skeleton-engine",
  rev = "<engine-commit-hash>"
}
```

## Applied Change

- `crates/game/Cargo.toml`: changed `engine` from git dependency to local path dependency.
- `Cargo.lock`: removed the git source entry from the `skeleton-engine` package.
- Verification run after the change:

```bash
cargo check -p game --bin survivor --locked
```
