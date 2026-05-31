# Fix stageclear/gameover BGM infinite loop regression found by ultrareview

**Date:** 2026-05-31
**Status:** COMPLETED
**Bead(s):** none
**Epic:** Rust Survivors release hardening
**Chain:** `bgm-path-hardening` seq `2`
**Parent:** `HANDOFF_bgm-path-hardening_executable-audio-paths_2026-05-31.md`
**Prior chain:** `HANDOFF_bgm-path-hardening_executable-audio-paths_2026-05-31.md` > this

---

## Since Last Handoff

- Parent introduced `play_bgm_file()` with `!bgm_playlist_advances(key, variant_count)` as the rodio repeat flag — this was the root bug.
- Parent's "Where We're Going" mentioned visual QA and asset replacement; instead this session opened with an ultrareview finding a critical audio regression in the same code.
- The regression meant every StageClear and GameOver screen played the first variant on infinite loop — directly contradicting `docs/audio_assets.md` and `docs/ARCHITECTURE.md` which document these as 1-shot cues.
- Fix was straightforward once the bug was understood: decouple repeat-flag logic from playlist-advance gating logic.
- Test count in `bgm.rs` went from 10 → 11. All pass.
- Chain is now COMPLETED on the audio side; remaining worktree changes (new MP3s untracked, deleted WAVs, doc updates) are pre-existing and unrelated to this fix.

## Reference Documents

- `CLAUDE.md` — project conventions, asset paths, engine boundary
- `docs/audio_assets.md` — BGM naming rules, 1-shot vs repeat policy
- `docs/ARCHITECTURE.md` — describes stageclear/gameover as "1회 cue"
- `crates/game/src/survivor/bgm.rs` — only file changed this session

## The Goal

The ultrareview (triggered at session start) returned one severity-normal bug: StageClear and GameOver BGM tracks were looping forever. The fix needed to correct the rodio repeat flag in `play_bgm_file`, add a regression-proof test, and commit. No other scope. A second task this session was locating and installing a "plan ambiguity validation" skill from the marketplace.

## Where We Are

- `crates/game/src/survivor/bgm.rs` — bug fixed, committed as `cec4478`.
- `bgm_repeat_flag(key: &str) -> bool` — new thin helper, returns `bgm_repeats(key)`. Exists solely to give the test a precise target and to make the intent of the repeat argument to `audio.play` unambiguous.
- `play_bgm_file(audio, path, key, _variant_count)` — `_variant_count` now explicitly unused. Passes `bgm_repeat_flag(key)` to `audio.play` as the repeat flag (true = rodio loops, false = play once).
- `bgm_playlist_advances(key, variant_count)` — UNCHANGED; still used only for the manual playlist-advance gate in the steady-state branch of `BgmSystem::run`.
- `bgm_repeats(key)` — UNCHANGED; still returns `false` for `bgm_stageclear` and `bgm_gameover`, `true` for the three looping keys.
- New test `bgm_repeat_flag_is_false_for_one_shot_keys` — asserts `bgm_repeat_flag` for all 5 keys: false for stageclear and gameover, true for title/ingame/boss.
- Total BGM unit tests: 11 (10 from prior session + 1 new).
- All 11 tests pass: `cargo test -p game --lib --locked -- --test-threads=1 bgm`.
- `~/.claude/skills/grill-me/SKILL.md` — installed from `tkersey/dotfiles` repo on GitHub, v2.1.0. Skill is now active in the harness.

## What We Tried (Chronological)

1. **Ultrareview result ingested.** Report identified `play_bgm_file` at `bgm.rs:76-78` as the site. Reasoning: `bgm_playlist_advances("bgm_stageclear", 2)` → `false && true` → `false`; `!false` → `true` passed to rodio → infinite loop. Bug was undetected because the existing test `bgm_loop_policy_keeps_clear_and_gameover_one_shot` only exercised `bgm_repeats()` directly, never `play_bgm_file`.

2. **Read `bgm.rs` in full** to understand the two distinct predicates:
   - `bgm_repeats(key)` → one-shot vs looping intent
   - `bgm_playlist_advances(key, variant_count)` → should the runtime auto-advance to next variant?
   These are independent: a key can be one-shot (no repeat, no advance) or looping single-track (repeat, no advance) or looping multi-track (repeat, advance). The prior code conflated them for the repeat flag.

3. **Fix applied:** Added `bgm_repeat_flag(key)` (trivial wrapper over `bgm_repeats`), changed `play_bgm_file` to call it instead of `!bgm_playlist_advances(...)`. Renamed `variant_count` parameter to `_variant_count` since it's no longer used there.

4. **Test added:** `bgm_repeat_flag_is_false_for_one_shot_keys` — covers all 5 keys × the repeat flag. This is the test the prior session was missing; it would have caught the bug before merge.

5. **Skill search:** User asked if any installed skill validates plan ambiguity. None found locally. Searched SkillsMP and LobeHub. Identified `grill-me` (tkersey/dotfiles, v2.1.0) and `llm-council` as the best matches. User chose `grill-me`.

6. **Skill install:** Fetched raw SKILL.md via `gh api repos/tkersey/dotfiles/contents/codex/skills/grill-me/SKILL.md`. Created `~/.claude/skills/grill-me/SKILL.md`. Harness picked it up immediately (visible in next system-reminder).

## Key Decisions

- **Used `bgm_repeat_flag` wrapper instead of inlining `bgm_repeats(key)` directly in `play_bgm_file`.** Rationale: gives the unit test a named, testable surface that maps exactly to the argument passed to `audio.play`. Without it, the test would need to call `play_bgm_file` and observe `AudioManager` side effects — much heavier.

- **Did NOT change `bgm_playlist_advances`.** The predicate's name and semantics are correct for its actual use (gating manual playlist advance). The bug was in the call site misusing it as a repeat flag, not in the predicate itself.

- **Installed `grill-me` from `tkersey/dotfiles` rather than `reedmayhew18/claude-code-expert`.** The tkersey version (v2.1.0) was more complete: full lane-status matrix, domain interrogation packs, anti-drift checkpoint, structured `grill_decision_packet` YAML output. The reedmayhew18 version was unavailable via `gh api` (404).

- **Did NOT commit the untracked MP3s or deleted WAVs.** These are pre-existing worktree changes from the prior session's asset pipeline work. Committing them without understanding the full asset state would be risky. They remain untracked.

## Evidence & Data

### Bug proof chain (stageclear with 2 variants)

| Step | Expression | Value |
|---|---|---|
| `bgm_repeats("bgm_stageclear")` | `!matches!("bgm_stageclear", "bgm_gameover" \| "bgm_stageclear")` | `false` |
| `bgm_playlist_advances("bgm_stageclear", 2)` | `false && (2 > 1)` | `false` |
| `!bgm_playlist_advances(...)` (old code) | `!false` | `true` ← **rodio loops** |
| `bgm_repeat_flag("bgm_stageclear")` (fix) | `bgm_repeats("bgm_stageclear")` | `false` ← **rodio plays once** |

### Repeat flag truth table (all 5 keys)

| Key | `bgm_repeats` | Old repeat arg (`!bgm_playlist_advances(k,2)`) | New repeat arg (`bgm_repeat_flag(k)`) | Correct? |
|---|---|---|---|---|
| `bgm_title` | true | true | true | ✓ (was already correct) |
| `bgm_ingame` | true | true | true | ✓ |
| `bgm_boss` | true | true | true | ✓ |
| `bgm_stageclear` | false | **true** ← BUG | false | ✓ fixed |
| `bgm_gameover` | false | **true** ← BUG | false | ✓ fixed |

### Test results

```
running 11 tests
test survivor::bgm::tests::bgm_file_selection_wraps_between_available_variants ... ok
test survivor::bgm::tests::bgm_key_switches_to_boss_track_only_during_ingame_boss ... ok
test survivor::bgm::tests::bgm_loop_policy_keeps_clear_and_gameover_one_shot ... ok
test survivor::bgm::tests::bgm_playlist_advances_only_for_repeating_multi_track_bgm ... ok
test survivor::bgm::tests::bgm_repeat_flag_is_false_for_one_shot_keys ... ok   ← new
test survivor::bgm::tests::bgm_variant_slots_are_stable ... ok
test survivor::bgm::tests::bgm_variants_exist_for_each_survivor_situation ... ok
test survivor::bgm::tests::executable_relative_dirs_are_checked_before_dev_fallback ... ok
test survivor::bgm::tests::macos_bundle_audio_dir_has_priority ... ok
test survivor::bgm::tests::missing_asset_resolves_to_none ... ok
test survivor::bgm::tests::no_relative_cwd_fallback_is_used ... ok
test result: ok. 11 passed; 0 failed; 0 ignored
```

### Commit log (this chain)

| Hash | Summary |
|---|---|
| `cfb087a` | Harden survivor BGM asset resolution (prior session) |
| `cec4478` | Fix stageclear/gameover BGM looping forever instead of playing once |

### Skill marketplace search results

| Skill | Source | Fit |
|---|---|---|
| `grill-me` v2.1.0 | tkersey/dotfiles (LobeHub/SkillsMP) | Best — exhaustive lane-status matrix, decision packet |
| `llm-council` | openclaw-skills (LobeHub) | Heavy — multi-agent council, more overhead |
| `create-plan` | davila7/claude-code-templates | Adjacent — plan creation, not ambiguity interrogation |
| `codex-plan-mode` | astomodynamics (LobeHub) | Adjacent — 5-phase workflow, not interrogation |

## Code Analysis

- `bgm_repeat_flag(key: &str) -> bool` — `crates/game/src/survivor/bgm.rs:75`. Trivial wrapper. Exists for testability.
- `play_bgm_file(audio: &mut AudioManager, path: &str, key: &str, _variant_count: usize)` — `bgm.rs:79`. The `_variant_count` parameter is kept in signature for API stability; `bgm_playlist_advances` is still called from `BgmSystem::run` with the real count.
- `bgm_playlist_advances(key, variant_count)` — `bgm.rs:71`. Returns `bgm_repeats(key) && variant_count > 1`. Used exclusively for the `is_finished` gate in the steady-state branch of `run`. NOT the repeat flag.
- The single-track looping case (variant_count == 1) now works correctly: `bgm_repeat_flag` returns true for title/ingame/boss regardless of variant count, so a single-file BGM still loops.

## Files Changed

### Source code
- `crates/game/src/survivor/bgm.rs` — added `bgm_repeat_flag`, fixed `play_bgm_file` repeat arg, renamed `variant_count` → `_variant_count`

### Tests
- `crates/game/src/survivor/bgm.rs` — added `bgm_repeat_flag_is_false_for_one_shot_keys` test

### Skills installed (outside repo)
- `~/.claude/skills/grill-me/SKILL.md` — v2.1.0, sourced from tkersey/dotfiles

## User Feedback & Preferences

- Asked to search marketplace after local skill list came up empty — preference for checking external sources before giving up.
- Chose `grill-me` over `llm-council` without hesitation after seeing the comparison — prefers lean interrogation over heavy council overhead.
- Said "commit 해줘" directly without providing a message — trusts Claude to write commit messages.
- Triggered `/handoff` at end of session as standard practice.

## Where We're Going

1. **Commit pre-existing worktree changes** — the untracked MP3s (`rustsurvivors *.mp3`), deleted WAVs, and modified docs/assets need a deliberate commit once the asset pipeline is confirmed complete.
2. **Manual visual/audio QA** — per `docs/manual_qa_checklist.md`: verify StageClear and GameOver screens now play their cue exactly once, then silence.
3. **Expand Korean/English localization coverage** — listed in CLAUDE.md next work.
4. **Replace placeholder assets with licensed final art/audio** — MP3s currently in worktree are placeholders; license documentation in `docs/ASSET_LICENSES.md`.
5. **macOS release hardening** — run `bash scripts/package_macos.sh` and `bash scripts/verify_macos_package.sh` after asset changes land.

## Risks & Blockers

- Untracked `"rustsurvivors *.mp3"` files have spaces in names — ensure shell quoting is correct when staging them.
- `docs/ENGINE_CHANGE_REQUESTS.md` contains pending engine-side codec policy changes that are not yet actioned; the rodio/mp3 coupling remains a temporary coupling per prior session notes.

## Open Questions

- Are the untracked MP3s the final licensed audio files, or still placeholders? Determines whether they should be committed now or after license review.

## Session Closed
**Closed at:** 2026-05-31
**Commit:** 5d7c224
**Session status:** Handed off to next session

## Quick Start for Next Session

```bash
# Reference docs
cat CLAUDE.md
cat docs/audio_assets.md
cat docs/manual_qa_checklist.md

# Key files
crates/game/src/survivor/bgm.rs   # BGM system — just fixed
docs/ASSET_LICENSES.md            # License status of audio assets
docs/ENGINE_CHANGE_REQUESTS.md    # Pending engine-side codec changes

# Verify current state
cargo test -p game --lib --locked -- --test-threads=1 bgm
# Expected: 11 passed, 0 failed

# Review uncommitted asset changes
git status -s

# Next action
# Manual QA: run the game, reach StageClear, confirm BGM plays once then stops
cargo run -p game --bin survivor
```
