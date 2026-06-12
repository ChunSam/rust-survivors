# Actor Facing UV Flip Applied

**Date:** 2026-06-12
**Status:** COMPLETED
**Bead(s):** none
**Epic:** Rust Survivors visual readability / actor motion polish
**Chain:** `actor-facing` seq `1`
**Parent:** none — first in chain
**Prior chain:** none — first in chain

---

## Related Handoffs

- `plans/handoffs/HANDOFF_stage-tilemaps_textured-backgrounds_2026-06-12.md` — immediately preceding visual asset/background workstream; related because it also touched sprite/UV rendering policy and validation counts.
- `plans/handoffs/HANDOFF_survivor-ui-hud-layout_visual-qa-fixes_2026-06-02.md` — previous visual QA chain, useful if future screenshots show rendering/layering issues.
- `plans/handoffs/HANDOFF_engine-audio-update_gameplay-actions_2026-06-04.md` — gameplay/action baseline and broader validation posture.

## Reference Documents

- `AGENTS.md` — repo rules, engine boundary, dirty worktree policy, validation commands, and Rust-beginner explanation preference.
- `docs/NEXT_WORK_PLAN.md` — active plan and latest validation count.
- `docs/manual_qa_checklist.md` — manual visual QA items for this pass.
- `crates/game/src/survivor/sprites.rs` — actor sprite mapping, animation setup, and new facing system.
- `crates/game/src/bin/survivor.rs` — survivor system registration order.

## The Goal

The user reported that characters and enemies looked like they were walking backward when moving left because their images stayed fixed in one horizontal direction. The goal was to make player, enemy, and boss actor sprites flip left/right according to movement direction while keeping the current engine-backed `Sprite` + `UvRect` rendering path. The change needed to work with the engine `AnimationSystem`, which writes fresh animation UVs every frame, and avoid editing the external `skeleton-engine` checkout.

## Where We Are

- Branch at handoff creation: `main`.
- Latest pushed commit before this pass: `093d8a3 Add stage tilemap backgrounds`.
- Current worktree still has many unrelated dirty files from earlier sessions; do not use `git add -A`.
- This pass intentionally changed only:
  - `crates/game/src/bin/survivor.rs`
  - `crates/game/src/survivor/mod.rs`
  - `crates/game/src/survivor/sprites.rs`
  - `docs/manual_qa_checklist.md`
  - `docs/NEXT_WORK_PLAN.md`
  - this handoff file
- `ActorFacing` was added in `sprites.rs` as a private component attached only to actor sprites.
- `ActorFacingSystem` was added in `sprites.rs` as a public system.
- `ActorFacingSystem` compares each actor's current `Transform.position.x` with its previous x coordinate.
- Horizontal movement left sets `direction_x` negative.
- Horizontal movement right sets `direction_x` positive.
- No significant horizontal movement preserves the previous facing direction.
- `ACTOR_FACING_EPSILON` is `0.01`, avoiding noise from tiny float deltas.
- `set_uv_facing` applies `UvRect.flipped_x()` only when the current UV orientation differs from desired facing.
- `ActorFacingSystem` also flips `BlendUv.to` when present, so crossfade output remains direction-consistent if animation blending is used later.
- `add_tinted_sprite` now attaches `ActorFacing` when `SurvivorSprite::actor_row()` is `Some(_)`.
- Non-actor sprites such as pickups and effect sprites do not receive `ActorFacing`.
- `crates/game/src/bin/survivor.rs` registers `ActorFacingSystem::default()` immediately after `AnimationSystem::new()`.
- The registration order matters: engine `AnimationSystem` writes the base animated `UvRect`, then `ActorFacingSystem` flips that current-frame UV if needed.
- `crates/game/src/survivor/mod.rs` re-exports `ActorFacingSystem` for the survivor binary.
- `docs/manual_qa_checklist.md` now has `2026-06-12 Actor Facing Flip Pass`.
- `docs/NEXT_WORK_PLAN.md` now records 213 passed tests and the actor-facing implementation note.
- Automated tests passed with 213 lib tests.
- Release build for `survivor` passed.
- Manual macOS visual QA still remains: confirm player/enemy/boss orientation in the actual window.

## What We Tried (Chronological)

1. Investigated player movement.
   - `PlayerMovementSystem` computes input direction and updates `Transform.position`.
   - It does not meaningfully update the existing `Velocity` component.
   - Therefore using `Velocity` alone would not solve the problem without adding more writes.

2. Investigated enemy movement.
   - `EnemyAiSystem` computes positions for chase, hover, kite, dash, stay, and split AI.
   - It updates `Transform.position` after caching updates to satisfy Rust borrow rules.
   - Enemies also do not share a general velocity component.

3. Investigated actor sprite setup.
   - `add_tinted_sprite` attaches `Sprite`, `RenderLayer`, `UvRect`, and `AnimationPlayer`.
   - Actor sprites are identified by `SurvivorSprite::actor_row()`.
   - Actor frames come from `survivor_actor_frames.png`.

4. Investigated engine animation behavior.
   - Engine `AnimationSystem` writes `UvRect` and `BlendUv` every frame from `AnimationPlayer`.
   - If facing were applied before `AnimationSystem`, animation would overwrite the flip.
   - This drove the decision to add a separate game-side system after `AnimationSystem`.

5. Rejected a transform-negative-scale approach.
   - Flipping `Transform.scale.x` would affect collision/readability assumptions less directly, but this project already uses `UvRect` for sprite orientation.
   - Negative scale could interact with scale bump/hit flash behavior and aspect tests.
   - `UvRect.flipped_x()` is the engine-supported, explicit texture mirroring path.

6. Implemented `ActorFacing` and `ActorFacingSystem`.
   - The system stores only `(Entity, current_x)` in a scratch vector before mutating the world.
   - This follows the existing borrow-checker pattern in `PlayerMovementSystem` and `EnemyAiSystem`.

7. Added tests.
   - Verified actor sprites receive facing state.
   - Verified non-actor sprites do not.
   - Verified moving left flips the UV.
   - Verified moving right restores the original UV.

8. Ran validation.
   - `cargo fmt --check` initially failed only on import formatting.
   - Ran `cargo fmt`.
   - Re-ran `cargo fmt --check`, which passed.
   - Ran lib tests: 213 passed.
   - Ran release build: passed.
   - Ran `git diff --check`: passed.

## Key Decisions

- Use a game-side `ActorFacingSystem`, not an engine change.
  - Reason: engine already exposes `UvRect.flipped_x()` and no engine API gap blocks this.

- Run facing after `AnimationSystem`.
  - Reason: animation overwrites `UvRect`; post-animation is the only stable place to apply direction mirroring.

- Detect facing from world x-position deltas.
  - Reason: player and enemy movement do not share a reliable velocity component today.

- Attach facing state only to actor sprites.
  - Reason: pickups, projectiles, effects, UI, and background tiles should not flip from movement deltas.

- Preserve facing during vertical movement or idle.
  - Reason: when x delta is near zero, switching direction would look jittery or arbitrary.

- Use `UvRect.flipped_x()` rather than negative `Transform.scale.x`.
  - Reason: the repo's sprite policy already uses manual `UvRect` for crop/orientation, and tests directly protect UV behavior.

## Evidence & Data

| Command | Result |
|---|---|
| `cargo fmt --check` | passed after `cargo fmt` |
| `cargo test -p game --lib --locked -- --test-threads=1` | 213 passed, 0 failed |
| `cargo build -p game --bin survivor --release --locked` | passed |
| `git diff --check` | passed |

Test summary:

```text
running 213 tests
test result: ok. 213 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.11s
```

Diff stat for intended files before this handoff:

```text
crates/game/src/bin/survivor.rs     |  46 ++++++------
crates/game/src/survivor/mod.rs     |   2 +-
crates/game/src/survivor/sprites.rs | 143 +++++++++++++++++++++++++++++++++++-
docs/NEXT_WORK_PLAN.md              |   3 +-
docs/manual_qa_checklist.md         |   7 ++
5 files changed, 175 insertions(+), 26 deletions(-)
```

Important code locations:

- `crates/game/src/survivor/sprites.rs:558` — `ActorFacing` definition starts nearby.
- `crates/game/src/survivor/sprites.rs:590` — `ActorFacingSystem`.
- `crates/game/src/bin/survivor.rs:116` — system registration after `AnimationSystem`.
- `docs/manual_qa_checklist.md:8` — manual QA entry.

## Code Analysis

- `ActorFacing { direction_x, last_x }` is private to `sprites.rs`.
- `ActorFacing::new(last_x)` defaults to facing right with `direction_x = 1.0`.
- `ActorFacing::update(current_x)` changes direction only when `abs(current_x - last_x) > 0.01`.
- `ActorFacingSystem` queries `Transform + ActorFacing`, not `Player` or `Enemy`, so bosses and future actor sprites automatically participate.
- `set_uv_facing` is idempotent: it flips only when `facing_left != (uv.u_size < 0.0)`.
- Because `AnimationSystem` writes unflipped frame UVs each frame, the facing system can safely reapply the correct orientation without accumulating repeated flips.
- `BlendUv.to` is flipped too, which keeps future animation crossfades coherent.
- Tests use `actor_frame(0, 0).uv()` as the baseline Hero frame UV.

## Files Changed

### Source code

- `crates/game/src/survivor/sprites.rs` — added actor facing component/system, system tests, and actor-only facing state attachment.
- `crates/game/src/survivor/mod.rs` — re-exported `ActorFacingSystem`.
- `crates/game/src/bin/survivor.rs` — registered `ActorFacingSystem` immediately after `AnimationSystem`.

### Tests

- `crates/game/src/survivor/sprites.rs` — added:
  - `actor_sprites_receive_facing_state`
  - `actor_facing_flips_uv_when_moving_left`
  - `actor_facing_returns_uv_to_right_when_moving_right`

### Documentation

- `docs/manual_qa_checklist.md` — added Actor Facing Flip Pass manual QA section.
- `docs/NEXT_WORK_PLAN.md` — updated test count to 213 and recorded actor-facing behavior.
- `plans/handoffs/HANDOFF_actor-facing_uv-flip_2026-06-12.md` — this handoff.

## User Feedback & Preferences (REQUIRED — never omit)

- User reported that character/enemy sprites looked fixed in one direction.
- User specifically said moving left looked like walking backward.
- User wants practical game polish applied directly, not just planning.
- User previously asked to maximize current engine feature usage.
- User expects handoff, commit, and push when requested with `/handoff 커밋 푸시`.
- Repo guidance says explain Rust borrow-checker workarounds briefly; this implementation uses the same cache-then-mutate pattern already present in movement systems.
- Repo guidance says preserve unrelated dirty worktree changes.

## Where We're Going

1. Stage only the actor-facing files and this handoff.
2. Commit with a focused message such as `Flip actor sprites by movement direction`.
3. Push `main`.
4. Next manual QA: run `cargo run -p game --bin survivor` and confirm player, common enemies, and bosses face left/right correctly.
5. If visual QA shows the art is authored facing left by default, invert the `direction_x` default or the left/right condition in `set_uv_facing`.

## Risks & Blockers

- Existing unrelated dirty files remain in the worktree; avoid broad staging.
- Manual visual QA in the macOS window has not been performed yet.
- Direction is inferred from position delta, so a teleport on x could temporarily set facing from the teleport direction. Current gameplay teleports are rare; not a blocker.

## Open Questions

- Should plant/stationary enemies keep their default right-facing orientation, or should they face the player even without moving?
- Should projectile sprites eventually use direction-based rotation/flip as a separate pass?
- Should `Velocity` be updated by `PlayerMovementSystem` in a future cleanup so player intent can be read directly?

## Quick Start for Next Session

```bash
# Restore context
sed -n '1,230p' plans/handoffs/HANDOFF_actor-facing_uv-flip_2026-06-12.md

# Key files
sed -n '520,635p' crates/game/src/survivor/sprites.rs
sed -n '108,120p' crates/game/src/bin/survivor.rs
sed -n '1,25p' docs/manual_qa_checklist.md

# Verify current state
cargo fmt --check
cargo test -p game --lib --locked -- --test-threads=1
cargo build -p game --bin survivor --release --locked

# Next action
cargo run -p game --bin survivor
# Move left/right and observe player, enemies, and bosses in a real window.
```

## Handoff Self-Check

- Chain is `actor-facing` seq `1`.
- Parent is `none — first in chain`.
- Related handoffs are listed but not used as parents.
- File names and functions are concrete.
- Evidence includes exact commands and result counts.
- User preferences are present.
- Quick Start has one concrete next action.
