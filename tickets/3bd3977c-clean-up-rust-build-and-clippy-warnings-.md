+++
id = "3bd3977c"
title = "Clean up rust build and clippy warnings across the workspace"
state = "new"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/3bd3977c-clean-up-rust-build-and-clippy-warnings-"
created_at = "2026-05-30T07:17:32.280247Z"
updated_at = "2026-05-30T07:17:32.280247Z"
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

Checkboxes; each one independently testable.

### Out of scope

Explicit list of what this ticket does not cover.

### Approach

How the implementation will work.

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-05-30T07:17Z | — | new | philippepascal |
