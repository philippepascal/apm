+++
id = "bfcaacd0"
title = "e2e tests from 537c2e09 fail on main: setup_merge_dep_repo leaves merge_failed transition on pr_or_epic_merge"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/bfcaacd0-e2e-tests-from-537c2e09-fail-on-main-set"
created_at = "2026-08-28T18:18:15.540678Z"
updated_at = "2026-08-28T18:18:50.064653Z"
+++

## Spec

### Problem

`cargo test --workspace` fails on `main`: `sync_does_not_block_on_closed_branchless_dependency` and `new_still_fails_when_dependency_does_not_exist_anywhere` (`apm/tests/e2e.rs`, ~lines 1341-1454) both panic with:

```
config: config: workflow — inconsistent completion strategies: state.in_progress.transition(implemented) uses 'merge', state.merge_failed.transition(implemented) uses 'pr_or_epic_merge'; depends_on validation assumes one project-wide completion strategy
error: config has changed and validation is failing.
Mutating commands are blocked. Run apm validate to fix.
```

The shared test helper `setup_merge_dep_repo()` patches the generated `.apm/workflow.toml` so the `in_progress -> implemented` transition uses completion strategy `merge` instead of the default `pr_or_epic_merge` (needed so `depends_on` validation runs unconditionally for these tests). It does this with a single `String::replace` targeting only the two-space-indented line `completion  = "pr_or_epic_merge"`, which belongs to the `in_progress` transition. It misses the one-space-indented line `completion = "pr_or_epic_merge"` used by the `merge_failed -> implemented` transition, so after the patch the two transitions disagree on completion strategy.

This was harmless until `apm-core/src/validate.rs` gained Rule 4 (`validate_config`, ~line 536): "at most one distinct non-`none` completion strategy across all transitions." Once a repo's workflow has mixed strategies, `apm validate` fails and `apm`'s mutating commands (`new`, `sync`, `state`, ...) refuse to run. `setup_merge_dep_repo()`'s output now always fails this check, so both tests that use it fail deterministically, breaking `cargo test --workspace` for every contributor and CI run on `main`.

The fix pattern already exists in this same test file: `Env::new()` (`apm/tests/e2e.rs`, ~lines 130-143) chains two `.replace()` calls — one per indentation variant — to rewrite both the `in_progress` and `merge_failed` completion lines together when swapping in a different strategy. `setup_merge_dep_repo()` needs the equivalent second replace.

### Acceptance criteria

- [ ] `cargo test --workspace` passes with zero failures
- [ ] `cargo test --test e2e sync_does_not_block_on_closed_branchless_dependency` passes in isolation
- [ ] `cargo test --test e2e new_still_fails_when_dependency_does_not_exist_anywhere` passes in isolation
- [ ] In the `.apm/workflow.toml` produced by `setup_merge_dep_repo()`, both the `in_progress -> implemented` transition and the `merge_failed -> implemented` transition have `completion = "merge"`, and no transition in that file still reads `pr_or_epic_merge`
- [ ] `apm validate` run against a repo built by `setup_merge_dep_repo()` reports no "inconsistent completion strategies" error

### Out of scope

- Changing the default `.apm/workflow.toml` template (`apm-core/src/default/workflow.toml`) — it keeps `pr_or_epic_merge` on both transitions; only the test helper's runtime patch is fixed.
- Revisiting Rule 4 or the completion-strategy-consistency check in `apm-core/src/validate.rs` (~line 536) — that logic is working as intended and is not in question here.
- Other test helpers that already patch `workflow.toml` correctly, e.g. `Env::new()` in `apm/tests/e2e.rs` (~lines 130-143) and `setup_merge_strategy_remote`-style helpers in `apm/tests/integration.rs` (~lines 5469-5480) — these already chain both replacements and need no change.
- Any process fix for why 537c2e09 shipped without accounting for Rule 4 (c82f853f landed after 537c2e09 branched) — this ticket is a point fix to the test fixture only.

### Approach

How the implementation will work.

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-08-28T18:18Z | — | new | philippepascal |
| 2026-08-28T18:18Z | new | groomed | philippepascal |
| 2026-08-28T18:18Z | groomed | in_design | philippepascal |