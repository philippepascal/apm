+++
id = "3f4ebae7"
title = "apm state implemented from merge_failed should detect already-merged work and skip re-merging"
state = "new"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/3f4ebae7-apm-state-implemented-from-merge-failed-"
created_at = "2026-05-29T23:03:15.303584Z"
updated_at = "2026-05-29T23:03:15.303584Z"
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
| 2026-05-29T23:03Z | — | new | philippepascal |
