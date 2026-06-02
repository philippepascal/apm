+++
id = "dc2b08db"
title = "apm move should not change the current worktree checkout"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/dc2b08db-apm-move-should-not-change-the-current-w"
created_at = "2026-06-02T03:20:39.058642Z"
updated_at = "2026-06-02T06:07:54.104886Z"
+++

## Spec

### Problem

`apm move <ticket-id> <epic-id>` correctly reassigns the ticket to the new epic but leaves the main worktree's HEAD pointing at the ticket branch. The supervisor has to run `git checkout main` to recover their working state after every invocation.

The root cause is in `apm-core/src/ticket/ticket_util.rs::move_to_epic`, step 9. The implementation calls `git rebase --onto <newbase> <upstream> <branch>` with the three-argument form. Git's three-argument rebase checks out `<branch>` in the current worktree before replaying commits — this is what switches HEAD. Other ticket-mutating commands (`apm set`, `apm spec`, `apm state`) avoid this problem by using `commit_to_branch` / `try_worktree_commit`, which operate via temporary worktrees and never touch the calling worktree's HEAD.

The fix is to run the rebase inside a temporary worktree, exactly as `try_worktree_commit` does. After the rebase the local `refs/heads/<ticket_branch>` ref is updated to the rebased tip; the main worktree's HEAD is never touched. `commit_to_branch` (called immediately after) already operates safely without a checkout, so steps 10+ require no changes.

### Acceptance criteria

- [ ] Running `apm move <id> <epic-id>` from the main worktree with HEAD on `main` leaves HEAD on `main` when the command returns
- [ ] The ticket file on the ticket branch contains the updated `epic` field after the move, matching today's behaviour
- [ ] A history row (`move: main → epic/…`) is appended to the ticket branch after the move, with no regression in audit trail
- [ ] Uncommitted changes present in the main worktree before `apm move` are still present and uncommitted after the command returns
- [ ] An integration test in `apm/tests/integration.rs` creates a ticket, creates an epic, runs `apm move` with HEAD on `main`, and asserts HEAD is still `main` after the call

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
| 2026-06-02T06:07Z | groomed | in_design | philippepascal |