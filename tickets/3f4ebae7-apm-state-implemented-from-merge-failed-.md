+++
id = "3f4ebae7"
title = "apm state implemented from merge_failed should detect already-merged work and skip re-merging"
state = "specd"
priority = 0
effort = 2
risk = 2
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/3f4ebae7-apm-state-implemented-from-merge-failed-"
created_at = "2026-05-29T23:03:15.303584Z"
updated_at = "2026-05-29T23:55:53.642685Z"
+++

## Spec

### Problem

`apm state <id> implemented` from `merge_failed` leaves the ticket in a drifted state: the default `workflow.toml` carries no `completion` strategy on the `merge_failed → implemented` transition. The transition writes the state-change commit (frontmatter now `implemented`) to the ticket branch only — nothing flows to `target_branch`. When a supervisor has already merged the work into the epic externally and then runs `apm state <id> implemented` to close the loop, the epic's view of the ticket stays frozen at `merge_failed`. This requires either re-merging the ticket branch manually just to propagate one frontmatter row, or accepting the cosmetic drift. Observed in syn with ticket 25673007.

The root cause is a single missing field: the `merge_failed → implemented` transition in `apm-core/src/default/workflow.toml` has no `completion = "pr_or_epic_merge"`. Adding it (plus `on_failure = "merge_failed"`) makes the recovery path symmetric with `in_progress → implemented`. Here is why the fix works: `state.rs` writes the new state-change commit to the ticket branch **before** the completion arm runs (line 166, before `match completion`). By the time `PrOrEpicMerge` calls `merge_into_default`, `target_branch` already holds all prior work commits — only the fresh state-row commit is missing. The merge fast-forwards (or trivially non-ff merges the one new commit) and succeeds without conflict. No detect-skip or `is_branch_merged_into` pre-check is needed; the existing merge machinery handles it correctly.

**Gating on existing projects.** This fix updates only the default workflow template shipped with `apm` (`apm-core/src/default/workflow.toml`). Projects (such as syn) that maintain their own `workflow.toml` receive no automatic behavior change — `state.rs` falls into `CompletionStrategy::None` until the project manually adds `completion = "pr_or_epic_merge"` to its `merge_failed → implemented` transition. `apm validate` cannot auto-add `completion` (only `on_failure` once `completion` is present), so this is a one-time manual edit per project. Until that edit is made, the recovery path for those projects continues to produce the same drift.

### Acceptance criteria

- [ ] `apm-core/src/default/workflow.toml`'s `merge_failed → implemented` transition has `completion = "pr_or_epic_merge"` and `on_failure = "merge_failed"`.
- [ ] `apm state <id> implemented` from `merge_failed` when the merge succeeds: state commits to ticket branch as `implemented` and the state row is present in `target_branch`.
- [ ] `apm state <id> implemented` from `merge_failed` when the merge fails: ticket stays `merge_failed`, a history row is appended for the failed attempt, and no `implemented` state commit is written.
- [ ] All pre-existing `in_progress → implemented` integration tests pass without modification.
- [ ] `apm validate` reports an `on_failure` error for any project where `merge_failed → implemented` has a merging completion but no `on_failure`; `apm validate --fix` repairs it.
- [ ] `cargo test --workspace` passes.

### Out of scope

- Changes to the `pr` completion strategy — PR-based workflows are unaffected; only the epic-merge path is touched.
- Retroactive repair of tickets currently stuck in a drifted `merge_failed` state; those require supervisor intervention (`apm state <id> implemented` after this fix is deployed, or manual close).
- Auto-migration of existing project `workflow.toml` files to add `completion = "pr_or_epic_merge"` — only the default template is updated; existing projects must add `completion` manually (after which `apm validate --fix` can add `on_failure`).
- Changes to `apm-core/src/state.rs` or any git helper functions — the fix is purely a config change in `workflow.toml`.
- Changes to `apm-server`, `apm-ui`, or `apm sync`.
- Making `merge_failed` recoverable without supervisor action — the supervisor still decides when and how to recover.

### Approach

#### 1. Update `apm-core/src/default/workflow.toml`

In the `merge_failed` state block (lines 236–239), update the `→ implemented` transition to add `completion` and `on_failure`:

```toml
[[workflow.states.transitions]]
to         = "implemented"
trigger    = "manual"
completion = "pr_or_epic_merge"
on_failure = "merge_failed"
outcome    = "needs_input"
```

This is the only code change required. `state.rs` writes the new state-change commit to the ticket branch before the completion arm runs (line 166, before `match completion`). By the time `PrOrEpicMerge` calls `merge_into_default`, `target_branch` already holds all prior work commits — only the fresh state-row commit is missing. The merge fast-forwards cleanly. No changes to `state.rs` or any other file.

**Gating note**: existing projects that maintain their own `workflow.toml` (e.g. syn) must manually add `completion = "pr_or_epic_merge"` to their `merge_failed → implemented` transition. Once that edit is made, `apm validate --fix` can add `on_failure` automatically. Without the manual edit, those projects continue to fall into `CompletionStrategy::None` and see no behavior change from this ticket.

#### 2. Integration tests in `apm/tests/integration.rs`

Add two tests. Each sets up a temp repo with the default workflow, an epic branch, and a ticket with `target_branch` pointing to that epic, forced into `merge_failed` state.

- **`merge_failed_to_implemented_succeeds`**: no pre-existing conflict on the target; call `apm state <id> implemented`; assert state = `implemented` on ticket branch and ticket-branch tip is reachable from `target_branch`.
- **`merge_failed_to_implemented_fails`**: place a conflicting commit on `target_branch` before calling `apm state <id> implemented`; assert state stays `merge_failed` and the history table contains two `merge_failed` rows.

Existing `in_progress → implemented` tests are unaffected — they configure `completion` independently and do not assert on the `merge_failed → implemented` transition config.

### Open questions


### Amendment requests

- [x] Three concerns from the spec review, in priority order:

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
| 2026-05-29T23:55Z | in_design | specd | claude |
