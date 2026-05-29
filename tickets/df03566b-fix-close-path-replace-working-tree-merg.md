+++
id = "df03566b"
title = "Fix close path: replace working-tree merge into default with commit_to_branch on target_branch"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/df03566b-fix-close-path-replace-working-tree-merg"
created_at = "2026-05-29T23:19:06.681786Z"
updated_at = "2026-05-29T23:32:58.321596Z"
+++

## Spec

### Problem

BUGS in ticket::close (apm-core/src/ticket/ticket_util.rs:330). The close transition currently runs:

    crate::git::merge_branch_into_default(root, &branch, &config.project.default_branch, &mut merge_warnings)

This is wrong on three coupled axes:

1. WRONG DESTINATION. It merges into config.project.default_branch (main), ignoring the ticket's per-ticket target_branch. For epic-scoped tickets the destination should be the epic branch, not main. Compare apm-core/src/state.rs:190 which correctly resolves target as t.frontmatter.target_branch.as_deref().unwrap_or(config.project.default_branch.as_str()).

2. WRONG API. merge_branch_into_default performs a git merge in a working tree (the main worktree if main is checked out there). That makes close fail or produce a confusing warning whenever the user has uncommitted changes in the main checkout — which is what triggered investigation (a dirty Cargo.lock in syn's main worktree caused 'merge into main failed' during apm sync close). A close should never depend on the cleanliness of an unrelated working tree.

3. WRONG OPERATION. The work the close commit propagates is just a one-row frontmatter change (state field plus a History row). Performing a full git merge with worktree checkout is overkill and unsafe; the same effect is achievable via plumbing.

Sync's detection passes (Cases 1/2/3/4) only offer tickets whose work is ALREADY in destination, so the merge is also unnecessary for the work content — by the time close runs, the only thing target_branch is missing is the close transition row in the ticket file.

NAIVE FIX IS NOT ENOUGH. Simply deleting the merge call would cause real drift downstream:
- Main-scoped tickets: main's tickets/<id>.md stays at 'implemented' forever; after branch cleanup, Case 2 (implemented + branch gone) re-offers an already-closed ticket as a close candidate every sync.
- Epic-scoped tickets: epic's view freezes at 'implemented'; when the epic later merges to main, main inherits the stale state and Case 2 re-fires.

PROPER FIX (direction; spec-writer to refine):
- Remove the merge_branch_into_default call from ticket::close.
- Replace it with a second commit_to_branch call that writes the closed ticket-file content to the ticket's effective target (target_branch if set, else config.project.default_branch). The first commit_to_branch (existing) writes to the ticket branch; the new one writes the same content to target. Both use plumbing — no worktree, no working-tree dirty-check issues, no merge conflict surface (target already has the implemented content; only the state field and a new History row differ).
- Apply the symmetric fix to apm-core/src/state.rs's transition path when new_state is the workflow's closed (or terminal) state: after the existing commit_to_branch on the ticket branch, add a commit_to_branch on target_branch with the same content. This unifies the three close entry points (apm close, apm sync close, apm state ID closed, apm validate --fix close) so they all converge on the same durable end state.

OUTCOME:
- Sync-triggered close (the user's reported case) succeeds without disturbing the main worktree even when the user has uncommitted changes in main.
- Epic-scoped close propagates the closed state into the epic immediately; main eventually inherits via the epic merge — same flow as any other epic ticket state change.
- Main-scoped close propagates the closed state to main immediately; Case 2 no longer re-detects already-closed tickets after branch cleanup.
- apm close, apm sync close, and apm state ID closed produce identical end states.

CALL SITES TO UPDATE:
- apm-core/src/ticket/ticket_util.rs ticket::close: replace merge with target_branch commit_to_branch.
- apm-core/src/state.rs transition (the path for ->closed/terminal): add target_branch commit_to_branch after the existing ticket-branch commit_to_branch.
- No changes needed to apm/src/cmd/close.rs, apm/src/cmd/state.rs, or apm/src/cmd/validate.rs — they call into the helpers above.

OUT OF SCOPE:
- Propagating non-terminal intermediate state changes (e.g. implemented -> ammend, in_progress -> blocked) to target_branch. The same mechanism could apply but is a separate decision; this ticket fixes the close path only.
- Changes to merge_into_default in state.rs's in_progress -> implemented path (it correctly uses target_branch and is the right place to do a real merge that brings code content into target).
- The pr completion strategy (PR mechanism is unaffected).
- Sync's detection passes (they are correct and already detect content in target).
- apm-server / apm-ui.

TESTS to consider:
- Sync close of a main-scoped ticket with a dirty Cargo.lock in the main worktree: succeeds, no merge warning, main's tickets/<id>.md reads state=closed.
- Sync close of an epic-scoped ticket whose target_branch is the epic: the epic's tickets/<id>.md reads state=closed; main is untouched.
- apm close and apm state ID closed and apm sync close all produce byte-identical end states across the ticket branch and target_branch.
- The 'already closed' guard in ticket::close still fires correctly when the supervisor tries to close a ticket that has already been closed via state.rs.
- After branch cleanup of a main-scoped closed ticket, sync no longer re-offers the ticket via Case 2 (main's view now reads closed which is terminal).

NON-GOAL: changing what 'close' means at the workflow level. The transition validity rules are unchanged; only the side-effect on target_branch changes.

### Acceptance criteria

- [ ] `apm close <id>` on a main-scoped ticket (no `target_branch`) writes state=closed to both the ticket branch and `main`; no working-tree merge is performed.
- [ ] `apm close <id>` on an epic-scoped ticket writes state=closed to both the ticket branch and `target_branch`; `main` is not touched.
- [ ] `apm state <id> closed` writes state=closed to both the ticket branch and the effective target branch (same resolution: `target_branch` if set, else `default_branch`).
- [ ] `apm sync` auto-close of a main-scoped ticket succeeds even when the main worktree has uncommitted changes (no merge error, no dirty-worktree error).
- [ ] After a successful close of a main-scoped ticket and deletion of the ticket branch, `apm sync` no longer re-offers the ticket as a close candidate (Case 2 does not re-fire).
- [ ] Calling `apm close <id>` on an already-closed ticket returns an "already closed" error without making additional commits to any branch.
- [ ] When the target-branch commit fails during close, the failure is reported as a warning (non-fatal); the ticket-branch commit has already succeeded.

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
| 2026-05-29T23:19Z | — | new | philippepascal |
| 2026-05-29T23:28Z | new | groomed | philippepascal |
| 2026-05-29T23:32Z | groomed | in_design | philippepascal |