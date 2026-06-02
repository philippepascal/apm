+++
id = "dc2b08db"
title = "apm move should not change the current worktree checkout"
state = "groomed"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/dc2b08db-apm-move-should-not-change-the-current-w"
created_at = "2026-06-02T03:20:39.058642Z"
updated_at = "2026-06-02T06:07:09.088705Z"
+++

## Spec

### Problem

BUG: apm move <ticket-id> <epic-id> performs the move operation correctly (ticket is reassigned to the new epic, the move output confirms success), but as a side effect it switches the main worktree's HEAD to the ticket's branch. The user has to manually run git checkout main (or whatever they were on) to return to their working state.

REPRODUCTION (observed in the syn project):
- Supervisor on main branch in the main worktree
- Runs: apm move fe3c4c67 87375105 (move ticket fe3c4c67 into epic 87375105)
- Output: 'fe3c4c67: moved into epic 87375105' — the move succeeded
- Prompt now shows: syn git:(ticket/fe3c4c67-syn-core-make-https-client-helper-public) — the main worktree has been switched to the ticket branch
- Supervisor runs git checkout main to recover their working state

EXPECTED BEHAVIOUR: apm move should not touch the current worktree checkout. The supervisor's working directory should stay on whatever branch they were on (typically main). The move is a metadata-and-history operation; it should not be conflated with branch switching.

WHY THIS MATTERS: it surprises the supervisor every time. It risks losing in-flight changes if main had uncommitted work. It breaks the assumption that PM commands (move, set, show, list, etc.) leave the working tree alone. Worse, it can interact badly with git hooks (post-checkout) firing for an unintended checkout.

LIKELY CAUSE: apm move probably needs to commit the new epic assignment to the ticket branch (since branches are the source of truth for ticket frontmatter). The implementation likely does the equivalent of:
- git checkout ticket/<id>-...
- edit the ticket file (change the epic field)
- git commit
- ...but forgets to checkout back to the previous branch

The fix is to use the same pattern as other apm commands that mutate a ticket's branch without affecting the working tree: either use the ticket's worktree if one exists, or use git plumbing (commit-tree, update-ref) to write the change without a checkout. apm-core/src/git.rs::commit_to_branch (used by apm set, apm spec, apm state and others) is the right reference — those commands all mutate the ticket file on the ticket branch without disturbing the main worktree.

OUT OF SCOPE:
- Changes to what apm move does semantically (it should still move the ticket between epics; only the side effect of changing the working tree's HEAD is the bug)
- Behaviour when the move is run from inside the ticket's own worktree (separate concern)
- apm-server / apm-ui (this is a CLI-only bug)

ACCEPTANCE CRITERIA hints (for the spec-writer to refine):
- Running apm move <id> <epic-id> from the main worktree with HEAD on main leaves HEAD on main after the command returns
- The ticket file on the ticket branch is correctly updated to reflect the new epic, just as today
- A history row is committed to the ticket branch as today (no regression in audit trail)
- apm log / apm show on the ticket reflects the move
- If the main worktree had uncommitted changes before apm move, those changes are still present and uncommitted after the command (no silent stash / no checkout-induced loss)
- A test (preferably integration) that runs apm move from a tempdir with HEAD on main and asserts HEAD is still main after the command

REFERENCES:
- apm/src/cmd/move.rs (the implementation)
- apm-core/src/git.rs::commit_to_branch (the plumbing pattern that other ticket-mutating commands use without changing the working tree)
- Background: the supervisor hit this in the syn project running apm move fe3c4c67 87375105 and had to git checkout main to recover

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
| 2026-06-02T03:20Z | — | new | philippepascal |
| 2026-06-02T06:07Z | new | groomed | philippepascal |
