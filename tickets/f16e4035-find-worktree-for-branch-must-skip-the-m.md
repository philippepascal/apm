+++
id = "f16e4035"
title = "find_worktree_for_branch must skip the main worktree"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/f16e4035-find-worktree-for-branch-must-skip-the-m"
created_at = "2026-05-28T07:31:38.018076Z"
updated_at = "2026-05-28T07:38:00.796245Z"
+++

## Spec

### Problem

find_worktree_for_branch in apm-core/src/worktree.rs returns the first git worktree list entry matching the branch, including the main worktree. If the main repo has a ticket branch checked out (e.g. because apm new opened an editor on it), the server dispatch cycle can spawn a worker with cwd=main repo instead of a fresh worktree. Fix: in find_worktree_for_branch, skip any worktree entry whose canonical path matches the main worktree root, and also skip entries whose branch name does not match the ticket/* pattern. list_ticket_worktrees already does this correctly — find_worktree_for_branch should apply the same guard. The ticket-branch name filter is the more robust check: a valid worker worktree must be on a ticket/* branch, so anything else should never be returned as a target.

### Acceptance criteria

- [ ] When the main worktree has a ticket branch checked out, `find_worktree_for_branch` returns `None` for that branch
- [ ] When a dedicated worktree exists for a ticket branch, `find_worktree_for_branch` returns its path
- [ ] `find_worktree_for_branch` never returns a path for a branch that does not start with `ticket/`
- [ ] `ensure_worktree` creates a new dedicated worktree when the main worktree holds the branch (i.e. does not reuse the main worktree path)
- [ ] A unit test in `worktree.rs` covers the main-worktree-has-the-branch scenario and asserts `None` is returned

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
| 2026-05-28T07:31Z | — | new | philippepascal |
| 2026-05-28T07:37Z | new | groomed | philippepascal |
| 2026-05-28T07:38Z | groomed | in_design | philippepascal |