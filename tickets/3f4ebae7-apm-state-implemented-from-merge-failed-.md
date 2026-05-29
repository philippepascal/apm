+++
id = "3f4ebae7"
title = "apm state implemented from merge_failed should detect already-merged work and skip re-merging"
state = "ammend"
priority = 0
effort = 4
risk = 2
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/3f4ebae7-apm-state-implemented-from-merge-failed-"
created_at = "2026-05-29T23:03:15.303584Z"
updated_at = "2026-05-29T23:43:47.708330Z"
+++

## Spec

### Problem

PROBLEM: the default workflow.toml's merge_failed -> implemented transition has no completion strategy (defaults to None). So when a supervisor runs apm state ID implemented to recover from merge_failed, the transition only writes a state-change commit to the ticket branch — nothing flows to target_branch (the epic, under pr_or_epic_merge). The work the supervisor manually merged into the epic is in the epic; the state-transition commit is on the ticket branch; the epic-side view of the ticket frontmatter stays frozen at merge_failed. This drift is recurring and forces either (a) a manual re-merge of the ticket branch into the epic just to ship one frontmatter row, or (b) closing the ticket from any state and accepting the cosmetic drift. Observed in syn recently with ticket 25673007.

ROOT CAUSE: two related gaps:
1. merge_failed -> implemented has no completion strategy in the default workflow, so the supervisor's recovery transition is a state-only commit on the ticket branch — never touches target_branch.
2. Even if we add a merging completion strategy to that transition, a naive retry would conflict in any case where the supervisor already resolved the merge externally (which is the common pattern — supervisor pulls into the worktree, resolves conflicts, pushes the merged branch, then transitions). The merge would be redundant and would either succeed as a no-op or surface as a confusing 'nothing to commit' / fast-forward situation. The state transition logic needs to detect 'already merged' and skip the merge entirely.

FIX (direction; spec-writer to refine):
A) Update apm-core/src/default/workflow.toml: give merge_failed -> implemented the same completion = 'pr_or_epic_merge' (and on_failure = 'merge_failed') that in_progress -> implemented has. This makes the recovery path symmetric with the first attempt.
B) Update apm-core/src/state.rs: in the Merge and PrOrEpicMerge completion branches, BEFORE attempting the merge, run is_branch_merged_into(root, ticket_branch, target_branch_or_default) using the helper added in ticket 12f2c7fa. If it returns true, skip the merge attempt and just commit the state change. If false, perform the merge as today, with on_failure -> merge_failed if it fails.

This single mechanism (detect-or-merge) makes both recovery patterns work cleanly:
- Supervisor fixed externally then transition: the branch is already in target, detection returns true, no merge attempted, state transition lands cleanly. No drift.
- Supervisor fixed at source then retry: the original cause is gone, detection returns false, the merge retries and succeeds.
And both transitions (in_progress -> implemented first attempt, merge_failed -> implemented recovery) share the same code path with the same idempotent behavior. No more 'state-transition commit on ticket branch but work on target branch' drift.

OUT OF SCOPE: changes to pr completion (PR is a different mechanism — apm opens or updates a PR; the detect-or-merge pre-check doesn't apply); changes to other workflow states; changes to apm-server / apm-ui; retroactive fixes for tickets currently stuck in drifted merge_failed state (those need supervisor intervention to close or re-merge manually — this ticket only prevents FUTURE drift); changes to the merge-failure detection itself (existing merge_into_default flow, on_failure handling).

NON-GOAL: making merge_failed recoverable without supervisor action. The supervisor still has to either fix the merge cause or merge externally before transitioning — apm just stops doing redundant or wrong work when they do.

TESTS:
- merge_failed -> implemented when branch is already merged into target_branch: transition succeeds, state commit lands on ticket branch, no merge attempt, no error.
- merge_failed -> implemented when branch is not yet in target and the merge succeeds: transition succeeds, work lands in target via the merge.
- merge_failed -> implemented when branch is not in target and the merge still fails: on_failure fires, ticket stays in merge_failed, no spurious state changes.
- Existing in_progress -> implemented tests: still pass; add one that confirms the pre-check is short-circuited correctly when the branch is somehow already in target (edge case but should be idempotent).
- workflow.toml default and any test fixtures that snapshot the workflow may need updating to reflect the new completion + on_failure on the merge_failed transition.

### Acceptance criteria

- [ ] `apm-core/src/default/workflow.toml`'s `merge_failed → implemented` transition has `completion = "pr_or_epic_merge"` and `on_failure = "merge_failed"`.
- [ ] `apm state <id> implemented` from `merge_failed` when the ticket branch is already merged into `target_branch` succeeds: state commits to ticket branch as `implemented` and `merge_into_default` is not invoked.
- [ ] `apm state <id> implemented` from `merge_failed` when the branch is not yet merged and the merge succeeds: state lands as `implemented` and work is present in `target_branch`.
- [ ] `apm state <id> implemented` from `merge_failed` when the branch is not yet merged and the merge fails: ticket stays `merge_failed`, a history row is appended for the failed attempt, and no `implemented` state commit is written.
- [ ] `apm state <id> implemented` from `in_progress` when the branch is already merged into the target (edge case): succeeds without invoking `merge_into_default`.
- [ ] All pre-existing `in_progress → implemented` integration tests pass without modification.
- [ ] `apm validate` reports an `on_failure` error for any project where `merge_failed → implemented` has a merging completion but no `on_failure`; `apm validate --fix` repairs it.
- [ ] `cargo test --workspace` passes.

### Out of scope

- Changes to the `pr` completion strategy — PR-based workflows are unaffected; the detect-skip check only applies to the `merge` and `pr_or_epic_merge` (epic-merge) paths.
- Retroactive repair of tickets currently stuck in a drifted `merge_failed` state; those require supervisor intervention (`apm state <id> implemented` after this fix is deployed, or manual close).
- Auto-migration of existing project `workflow.toml` files to add `completion = "pr_or_epic_merge"` — only the default template is updated; existing projects must add `completion` manually (after which `apm validate --fix` can add `on_failure`).
- Changes to the merge-failure detection functions (`is_branch_merged_into`, `merged_into_main`, `content_merged_into_main`) — these are correct and untouched.
- Changes to `apm-server`, `apm-ui`, or `apm sync`.
- Making `merge_failed` recoverable without supervisor action — the supervisor still decides when and how to recover.

### Approach

#### 1. Update `apm-core/src/default/workflow.toml`

In the `merge_failed` state block, add `completion` and `on_failure` to the `→ implemented` transition:

```toml
[[workflow.states.transitions]]
to         = "implemented"
trigger    = "manual"
completion = "pr_or_epic_merge"
on_failure = "merge_failed"
outcome    = "needs_input"
```

This makes the recovery path symmetric with `in_progress → implemented`. The `apply_on_failure_fixes` path in `validate --fix` benefits automatically because `default_on_failure_map()` reads this file, and the existing validate rule already enforces `on_failure` on all `Merge`/`PrOrEpicMerge` transitions.

#### 2. Add detect-skip to `apm-core/src/state.rs`

Both `CompletionStrategy::Merge` and `CompletionStrategy::PrOrEpicMerge` arms in the `match completion` block (after the state commit) need an already-merged check before calling `git::merge_into_default`.

**`Merge` arm** (currently ~line 182): resolve `merge_target`, push branch, then wrap the `merge_into_default` call:

```rust
if !git::is_branch_merged_into(root, &branch, merge_target).unwrap_or(false) {
    let merge_result = git::merge_into_default(...);
    if let Err(merge_err) = merge_result {
        // existing on_failure handling — unchanged
    }
}
```

The push remains unconditional so the remote ref stays current regardless.

**`PrOrEpicMerge` arm** (currently ~line 227): inside the `if let Some(ref target) = t.frontmatter.target_branch` block, wrap `merge_into_default` similarly:

```rust
if let Some(ref target) = t.frontmatter.target_branch {
    if !git::is_branch_merged_into(root, &branch, target).unwrap_or(false) {
        let merge_result = git::merge_into_default(...);
        if let Err(merge_err) = merge_result {
            // existing on_failure handling — unchanged
        }
    }
} else {
    // PR path — unchanged
}
```

`is_branch_merged_into` returns `Ok(false)` on any git error, so `unwrap_or(false)` is safe: on ambiguous result we fall through to the normal merge attempt rather than silently skipping it.

No other files change in `apm-core`.

#### 3. Integration tests in `apm/tests/integration.rs`

Add a helper `setup_epic_with_ticket` or inline setup in each test that:
- Inits a repo with the default workflow
- Creates an epic branch
- Creates a ticket with `target_branch = "epic/X"` and sets it to `merge_failed` via `--force`

Then add four tests:

- **`merge_failed_to_implemented_already_merged`**: manually merge ticket branch into the target before calling `apm state implemented`; assert state = `implemented` on ticket branch, assert `merge_into_default` was not attempted (no extra merge commit on target since the last manual merge).
- **`merge_failed_to_implemented_not_yet_merged_succeeds`**: don't pre-merge; call `apm state implemented`; assert state = `implemented` and ticket branch tip is reachable from target.
- **`merge_failed_to_implemented_not_yet_merged_fails`**: set up a conflicting commit on target so the merge cannot auto-resolve; call `apm state implemented`; assert state stays `merge_failed` and the history table contains two `merge_failed` rows.
- **`in_progress_to_implemented_already_merged_skips`**: same setup but ticket starts `in_progress`; pre-merge; assert transition succeeds and no duplicate merge.

Existing tests in `setup_merge()` and `setup_on_failure_fix_project()` are unaffected — they patch `completion` independently and don't assert on the `merge_failed → implemented` transition config.

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-05-29T23:03Z | — | new | philippepascal |
| 2026-05-29T23:28Z | new | groomed | philippepascal |
| 2026-05-29T23:28Z | groomed | in_design | philippepascal |
| 2026-05-29T23:32Z | in_design | specd | claude |
| 2026-05-29T23:43Z | specd | ammend | philippepascal |
