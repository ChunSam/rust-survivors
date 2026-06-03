# Engine Dependency Workflow

Updated: 2026-06-03

## Current Setup

`crates/game` depends on a pinned `skeleton-engine` git revision for release and CI reproducibility:

```toml
engine = {
  package = "skeleton-engine",
  git = "https://github.com/ChunSam/skeleton-engine",
  rev = "61c09f14434910a94afc4c9ea167b3f47e0b203b"
}
```

`Cargo.lock` records the git source and resolved engine package version, so another machine can build without a sibling checkout.

## Why This Changed

The game previously used a local sibling checkout for fast engine/game validation. The release posture now favors a pinned remote dependency:

- CI and package builds no longer require `/Users/jkl/Projects/skeleton-engine`.
- The exact engine revision is visible in `Cargo.toml` and `Cargo.lock`.
- Game work still validates externally completed engine changes through the survivor workload after updating the pinned rev.

Important: while working in `rust-survivors`, do not directly edit the external `skeleton-engine` checkout. If an engine change is needed, leave a request prompt in `docs/ENGINE_CHANGE_REQUESTS.md`.

## Tradeoffs

Benefits:

- Reproducible builds on CI, release machines, and clean developer machines.
- The engine version is explicit and reviewable.
- `--locked` validation checks the real release dependency graph.

Costs:

- Engine iteration requires updating the pinned `rev` and regenerating `Cargo.lock`.
- First build on a clean machine needs network access to fetch the pinned engine repo and crates.io dependencies.

## Recommended Workflow

Validate the game against the pinned engine revision:

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
5. After the engine change lands externally, update the pinned `rev`.
6. Regenerate `Cargo.lock` and run the validation commands above.

## Applied Change

- `crates/game/Cargo.toml`: changed `engine` from local path dependency to pinned git dependency.
- `Cargo.lock`: records `skeleton-engine v2.0.0` from rev `61c09f14434910a94afc4c9ea167b3f47e0b203b`.
- Verification run after the change:

```bash
cargo check -p game --bin survivor --locked
cargo test -p game --lib --locked -- --test-threads=1
```
