+++
id = "3bd3977c"
title = "Clean up rust build and clippy warnings across the workspace"
state = "specd"
priority = 0
effort = 3
risk = 2
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/3bd3977c-clean-up-rust-build-and-clippy-warnings-"
created_at = "2026-05-30T07:17:32.280247Z"
updated_at = "2026-05-30T17:13:09.520072Z"
+++

## Spec

### Problem

BUILD WARNINGS (cargo build --workspace + cargo test --workspace --no-run):
- apm-server/src/handlers/maintenance.rs:259 — unused variable id
- apm/tests/integration.rs:7395 — unused function push_remote_state_update

CLIPPY WARNINGS (cargo clippy --workspace --all-targets):
- 67 unique warnings spread across apm-core (18), apm-cli (14), apm-server (2-3), and test crates
- Per-crate breakdown:
  - apm-core lib: 18 (13 auto-fixable via clippy --fix)
  - apm-cli lib: 14 (10 auto-fixable)
  - apm-server: 2-3
  - test crates: assorted (mostly auto-fixable)

CATEGORIES seen (representative, not exhaustive):
- empty line after doc comment
- consecutive str::replace calls
- redundant closure
- Option.and_then(|x| Some(y)) → map
- map_or simplifications
- closure used to substitute value for Option::None
- loop variable only used to index a slice
- this impl can be derived
- this call to clone can be replaced with std::slice::from_ref
- items after a test module
- this function has too many arguments (one site, 8/7)

GOAL: get cargo build --workspace and cargo clippy --workspace --all-targets to zero warnings, by applying clippy's auto-fix suggestions where they exist and making targeted manual changes for the rest. cargo test --workspace must still pass after every batch.

NON-GOALS / CONSTRAINTS:
- Do not change behavior. Each clippy fix is a stylistic / idiomatic-rust change, not a logic change. If a 'fix' would alter behavior or readability for the worse, prefer #[allow(...)] with a one-line justification.
- Do not refactor a function signature to satisfy too-many-arguments — that warning is on a function whose 8 args reflect real per-call inputs; resolve with #[allow(clippy::too_many_arguments)] on the offending fn rather than a structural change.
- Do not delete the unused integration test helper push_remote_state_update by reflex — check whether it is intentionally kept for future use; if removed, do so in a separate commit with the test rationale stated.
- No project rule changes (no -D warnings, no deny lints, no clippy.toml additions). The cleanup is the deliverable; tightening enforcement is a separate concern.

APPROACH (direction; spec-writer to refine):
1. Run cargo clippy --fix --workspace --allow-dirty --allow-staged first to apply auto-fixes; commit that as one mechanical commit so the manual work that follows reviews cleanly.
2. Iterate manually on the remaining warnings, grouped by category, smallest crates first.
3. For each warning that cannot be cleanly fixed (e.g. the 8-arg function), apply targeted #[allow] with a one-line justification comment.
4. Resolve the two compiler warnings (unused id, unused fn) with the same judgement: delete if truly dead, keep with underscore-prefix or #[allow(dead_code)] if intentionally kept.

OUT OF SCOPE:
- Introducing new lints, RUSTFLAGS, or clippy.toml configuration.
- Refactoring code structure beyond what a single clippy suggestion demands.
- Touching apm-ui or non-rust code.
- The ts/js side of the workspace (tsc, eslint) — out of scope for this ticket.

TESTS:
- cargo test --workspace must pass on every commit (each mechanical batch).
- cargo build --workspace must emit zero warnings after the work is done.
- cargo clippy --workspace --all-targets must emit zero warnings after the work is done.

### Acceptance criteria

- [ ] `cargo build --workspace` emits zero warnings after all changes are committed
- [ ] `cargo clippy --workspace --all-targets` emits zero warnings after all changes are committed
- [ ] `cargo test --workspace` passes after the auto-fix commit (Phase 1)
- [ ] `cargo test --workspace` passes after all manual-fix commits (Phase 2)
- [ ] All four `too-many-arguments` sites (`apm-core/src/start.rs:574`, `apm/src/cmd/list.rs:6`, `apm/src/cmd/new.rs:6`, `apm/src/cmd/spec.rs:5`) are suppressed with `#[allow(clippy::too_many_arguments)]`, not restructured
- [ ] The `unused variable: id` at `apm-server/src/handlers/maintenance.rs:259` is resolved by renaming to `_id`
- [ ] `push_remote_state_update` at `apm/tests/integration.rs:7395` is either deleted (with rationale in commit message) or annotated with `#[allow(dead_code)]` and a comment explaining the intent

### Out of scope

- Introducing new lints, `RUSTFLAGS`, `deny` attributes, or `clippy.toml` configuration
- Refactoring code structure beyond what a single clippy suggestion demands
- Changing function signatures to reduce argument count
- Touching `apm-ui`, frontend assets, or any non-Rust file
- TypeScript/JavaScript tooling (tsc, eslint)
- Suppressing warnings via blanket `#![allow(...)]` at the crate level
- Deleting or modifying test logic beyond the mechanical `assert_eq!(x, true)` → `assert!(x)` auto-fix

### Approach

#### Phase 1 — Auto-fix batch

Run `cargo clippy --fix --workspace --allow-dirty --allow-staged` to apply all machine-safe suggestions in one pass. This covers approximately 33 warnings:

- `apm-core` lib (~13): consecutive `str::replace` chains collapsed to single call, redundant closures, `Option::and_then(|x| Some(y))` → `map`, `map_or` simplifications, `clone` → `std::slice::from_ref`, `field_reassign_with_default`, loop-index-only variables
- `apm-cli` lib (~10): same categories
- `apm-server` bin (~2): `literal with empty format string`, `manual char comparison`
- `apm/tests/validate_fix.rs` (~8): `assert_eq!(x, true)` → `assert!(x)`

Run `cargo test --workspace` to confirm no regressions. Commit: `Fix: apply clippy auto-fix suggestions across workspace`.

#### Phase 2 — Manual fixes

**`too-many-arguments`** — Add `#[allow(clippy::too_many_arguments)]` directly above each `pub fn run` or `pub fn spawn_next_worker` declaration. Do not change signatures. Add a one-line comment: `// Each argument maps to a distinct CLI flag.`
- `apm-core/src/start.rs:574` (`spawn_next_worker`, 8 args)
- `apm/src/cmd/list.rs:6` (`run`, 9 args)
- `apm/src/cmd/new.rs:6` (`run`, 11 args)
- `apm/src/cmd/spec.rs:5` (`run`, 11 args)

**`very complex type`** — Add `#[allow(clippy::type_complexity)]` at `apm/src/cmd/start.rs:41`.

**`items after a test module`** — In each file, move the non-test items that appear after `mod tests { ... }` to above the test module block:
- `apm-server/src/tls.rs:79`
- `apm-core/src/epic.rs:97`
- `apm-core/src/worktree.rs:163`
- `apm-core/src/logger.rs:59`
- `apm-core/src/ticket/ticket_fmt.rs:316`

**`unused variable: id`** — Rename `id` to `_id` at `apm-server/src/handlers/maintenance.rs:259`.

**`push_remote_state_update` unused** — Read `apm/tests/integration.rs` around line 7395 to check context. If the function has no pending callers or TODO comments, delete it and state the rationale in the commit message. If evidence suggests it was written for a planned test, keep it with `#[allow(dead_code)]` and a `// Reserved for <test name>` comment instead.

Run `cargo test --workspace` after this batch. Commit: `Fix: resolve remaining clippy and compiler warnings manually`.

#### Phase 3 — Verify zero warnings

Run all three checks and confirm clean output:
- `cargo build --workspace`
- `cargo clippy --workspace --all-targets`
- `cargo test --workspace`

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-05-30T07:17Z | — | new | philippepascal |
| 2026-05-30T17:08Z | new | groomed | philippepascal |
| 2026-05-30T17:09Z | groomed | in_design | philippepascal |
| 2026-05-30T17:13Z | in_design | specd | claude |
