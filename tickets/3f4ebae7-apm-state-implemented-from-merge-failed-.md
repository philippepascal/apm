+++
id = "3f4ebae7"
title = "apm state implemented from merge_failed should detect already-merged work and skip re-merging"
state = "in_design"
priority = 0
effort = 4
risk = 2
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/3f4ebae7-apm-state-implemented-from-merge-failed-"
created_at = "2026-05-29T23:03:15.303584Z"
updated_at = "2026-05-29T23:48:51.150125Z"
+++

## Spec

### Problem

`apm state <id> implemented` from `merge_failed` leaves the ticket in a drifted state: the default `workflow.toml` carries no `completion` strategy on the `merge_failed → implemented` transition. The transition writes the state-change commit (frontmatter now `implemented`) to the ticket branch only — nothing flows to `target_branch`. When a supervisor has already merged the work into the epic externally and then runs `apm state <id> implemented` to close the loop, the epic's view of the ticket stays frozen at `merge_failed`. This requires either re-merging the ticket branch manually just to propagate one frontmatter row, or accepting the cosmetic drift. Observed in syn with ticket 25673007.

The root cause is a single missing field: the `merge_failed → implemented` transition in `apm-core/src/default/workflow.toml` has no `completion = "pr_or_epic_merge"`. Adding it (plus `on_failure = "merge_failed"`) makes the recovery path symmetric with `in_progress → implemented`. Here is why the fix works: `state.rs` writes the new state-change commit to the ticket branch **before** the completion arm runs (line 166, before `match completion`). By the time `PrOrEpicMerge` calls `merge_into_default`, `target_branch` already holds all prior work commits — only the fresh state-row commit is missing. The merge fast-forwards (or trivially non-ff merges the one new commit) and succeeds without conflict. No detect-skip or `is_branch_merged_into` pre-check is needed; the existing merge machinery handles it correctly.

**Gating on existing projects.** This fix updates only the default workflow template shipped with `apm` (`apm-core/src/default/workflow.toml`). Projects (such as syn) that maintain their own `workflow.toml` receive no automatic behavior change — `state.rs` falls into `CompletionStrategy::None` until the project manually adds `completion = "pr_or_epic_merge"` to its `merge_failed → implemented` transition. `apm validate` cannot auto-add `completion` (only `on_failure` once `completion` is present), so this is a one-time manual edit per project. Until that edit is made, the recovery path for those projects continues to produce the same drift.

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

- [ ] Three concerns from the spec review, in priority order:

1) REFRAME THE PROBLEM AND ROOT CAUSE. The spec presents plan B (detect-skip via is_branch_merged_into) as load-bearing for the supervisor-fixed-externally flow. Trace the actual code timing: state.rs::transition appends a new ticket-branch commit at line 166 BEFORE the match completion block runs. That new state-change commit (merge_failed -> implemented in the frontmatter) is fresh and is NEVER in target by the time the completion arm runs. is_branch_merged_into therefore returns false. The detect-skip does NOT fire in the documented case. What actually fixes the documented case is plan A alone: adding completion = pr_or_epic_merge to merge_failed -> implemented in the default workflow makes the transition attempt a merge, and because target already contains all the work commits, the new state-row commit lands as a trivial fast-forward (or near-trivial non-ff). Update the Problem section, Root Cause section, and Fix section to state this clearly: plan A is the fix; plan B is defensive belt-and-suspenders for genuinely degenerate cases (cherry-pick equivalence, manual edits) and not the primary mechanism.

2) HANDLE THE DETECT-SKIP DRIFT GAP, OR DROP PLAN B. If plan B is kept, the case where it fires currently leaves the new state-row commit on the ticket branch and NEVER propagates it to target — target's view stays whatever it was last (possibly merge_failed). That is the same drift class that df03566b fixes for close. Two acceptable resolutions: (a) DROP PLAN B entirely; plan A handles the documented case correctly via the trivial merge. (b) KEEP PLAN B but add a commit_to_branch(root, target, rel_path, content, MSG) inside the skip branch so the new state-row reaches target via plumbing — same approach df03566b takes for the close path. Pick one; the spec author can decide which. Do not ship plan B without resolving the drift gap.

3) NOTE THE GATING ON EXISTING PROJECT WORKFLOWS. The fix only updates apm-core/src/default/workflow.toml. Existing projects (e.g. syn) ship their own workflow.toml which today lacks completion on merge_failed -> implemented. Without a manual edit those projects get NO behavior change from this ticket — state.rs's completion match falls into the None arm. The Out of scope section acknowledges no auto-migration; the Problem section should ALSO state plainly that this fix is gated on each project adding completion = pr_or_epic_merge to their workflow.toml. apm validate cannot auto-add completion (only on_failure once completion is present), so this is a one-time manual edit per project.

### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-05-29T23:03Z | — | new | philippepascal |
| 2026-05-29T23:28Z | new | groomed | philippepascal |
| 2026-05-29T23:28Z | groomed | in_design | philippepascal |
| 2026-05-29T23:32Z | in_design | specd | claude |
| 2026-05-29T23:43Z | specd | ammend | philippepascal |
| 2026-05-29T23:48Z | ammend | in_design | philippepascal |