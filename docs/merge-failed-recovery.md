# `merge_failed` recovery scenarios

When a worker reaches `implemented` under a merging completion strategy
(`merge` or `pr_or_epic_merge` with `target_branch` set), APM tries to merge
the ticket branch into the target. If that merge can't complete, the
transition's `on_failure` handler moves the ticket to `merge_failed` and
records the error in the ticket body. This doc enumerates what can cause
that, and what the supervisor does to get the ticket moving again.

The `merge_failed` state is actionable by the supervisor; valid transitions
out are `→ implemented` (after the cause is resolved) or `→ in_progress`
(to return the ticket to a worker for fresh work).

## What can fail

The merge runs in
`apm-core/src/git_util.rs::merge_into_default`, which fetches `target_branch`
and then runs `git merge --no-ff <ticket-branch>` in a worktree that has
`target_branch` checked out. Four distinct things can go wrong from there.

### A. True merge conflict

The ticket branch and `target_branch` touched overlapping lines and git
can't auto-resolve. `git merge --no-ff` exits non-zero with conflict
markers; APM aborts the merge and writes the error into the ticket body's
"Merge notes" section.

This is the most common cause when an epic has multiple in-flight workers
or when the epic has moved past the point the ticket was based on.

### B. Dirty target worktree

`target_branch` is checked out somewhere — usually the main repo's working
tree (for main-targeted tickets) or the epic worktree (for epic-targeted
tickets) — and that working tree has uncommitted changes that would be
overwritten by the merge. `git merge` refuses to start.

This is what surfaced when a stray edit to `Cargo.lock` in the supervisor's
main checkout blocked an `apm sync` close.

### C. Push rejection

The local merge succeeded — `target_branch` advanced — but the subsequent
`git push origin <target_branch>` was rejected. Common reasons: a pre-push
hook script missing or returning non-zero, a non-fast-forward push because
remote moved during the operation, or an authentication failure.

In this case, the work is in `target_branch` *locally* but not on origin.

### D. Worktree provisioning failure

`ensure_worktree` can't materialize a worktree for `target_branch`. Usually
a stale `.apm/worktrees/<name>` directory, a permissions issue, or a path
collision. Rare in practice.

### E. (Not actually `merge_failed`) — pre-merge leak detection

Before any merge attempts, `state.rs` runs `check_leaked_files`: if the
target worktree has uncommitted edits to files this ticket modified, the
transition bails before invoking `merge_into_default`. The ticket stays
in `in_progress`; it does **not** reach `merge_failed`. Listed here only
to clarify that "the merge bailed" is not necessarily `merge_failed`.

## Recovery per cause

The general pattern is: **resolve the cause, then re-run the same
`apm state <id> implemented` transition that originally failed.** What
"resolve the cause" looks like differs.

### Recovering from A (true conflict)

1. Check out `target_branch` in its working tree (e.g. the epic worktree
   for an epic ticket, or the main repo's checkout for a main ticket).
2. `git merge ticket/<id>-<slug>`.
3. Resolve the conflicts in the conflicted files.
4. `git add` the resolved files and `git commit` the merge.
5. `git push` `target_branch`.
6. `apm state <id> implemented`.

The retry transitions cleanly because `target_branch` now already contains
the ticket branch's work; only a fresh state-row commit needs to land,
which is a trivial fast-forward.

### Recovering from B (dirty target worktree)

1. In the worktree that has `target_branch` checked out, either commit or
   stash the unrelated changes (so `git status` is clean enough to allow
   `git merge` to proceed).
2. `apm state <id> implemented`.

The merge runs from scratch and completes normally.

### Recovering from C (push rejection)

The local merge already advanced `target_branch`; only the push failed.

1. Fix the rejection cause. Examples:
   - Pre-push hook script is missing on this branch — install it (or make
     the hook defensive when its script is absent).
   - Non-fast-forward — `git fetch origin <target_branch>` then rebase or
     re-merge.
   - Auth — refresh credentials.
2. `apm state <id> implemented`.

The retry's `merge_into_default` is a local no-op (target already has the
merge); the push retries and succeeds.

### Recovering from D (worktree provisioning failure)

1. Clean up the offending path (`rm -rf .apm/worktrees/<name>` if stale,
   or fix permissions, etc.).
2. `apm state <id> implemented`.

The retry rebuilds the worktree from scratch.

## Edge case: the ticket was "fixed" by means other than merging

If the supervisor brought the work into `target_branch` by cherry-picking
individual commits, copying files manually, or applying a patch — anything
other than a clean `git merge` of the ticket branch — then the ticket
branch is not a strict ancestor of `target_branch`, and `merge_into_default`
will try to merge "new" history on top of an already-applied set of
changes. The most likely outcome is a non-trivial merge with duplicate or
overlapping content; in some cases conflicts.

This case is uncommon but does happen. The recommended path is to redo the
recovery as a proper merge (A above): drop the cherry-picked commits from
`target_branch` (revert or reset before push), merge the ticket branch
normally, and proceed. If that's not possible, the alternative is to close
the ticket as abandoned (see below) and accept the surfaced drift — the
work is in `target_branch` regardless of what the ticket's history says.

## When the ticket should not be recovered

Sometimes the right move is to give up on the ticket rather than retry the
merge — for example, when the work is fundamentally superseded, when the
conflict can't be resolved without rewriting the ticket, or when the
supervisor decides the change isn't worth shipping.

Two paths:

- **`apm state <id> in_progress`** — return the ticket to a worker for
  fresh work (e.g. amendments, a rebase, or a different approach). The
  worker picks up from where it was.

- **`apm state <id> closed`** (or `apm close <id>`) — abandon. The ticket
  is closed without merging the work; any uncommitted-to-target work on
  the ticket branch is lost at branch cleanup. This is the supervisor's
  escape hatch when no clean recovery is possible.

## Related references

- Completion strategies and what triggers a merge attempt:
  `docs/strategy-and-dependencies.md`.
- The merge implementation: `apm-core/src/git_util.rs::merge_into_default`.
- The transition that drives this:
  `apm-core/src/state.rs::transition` (the `Merge` and `PrOrEpicMerge`
  match arms).
- The `merge_failed` state and its outgoing transitions are configured in
  the project's `.apm/workflow.toml`. Existing projects whose
  `merge_failed → implemented` transition has no `completion` field will
  only commit the state change to the ticket branch on retry — the merge
  itself won't be re-attempted. Set
  `completion = "pr_or_epic_merge"` (matching `in_progress → implemented`)
  and `on_failure = "merge_failed"` to make the retry path symmetric.
