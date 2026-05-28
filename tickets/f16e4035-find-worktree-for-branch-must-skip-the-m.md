+++
id = "f16e4035"
title = "find_worktree_for_branch must skip the main worktree"
state = "ready"
priority = 0
effort = 2
risk = 2
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/f16e4035-find-worktree-for-branch-must-skip-the-m"
created_at = "2026-05-28T07:31:38.018076Z"
updated_at = "2026-05-28T07:44:50.773243Z"
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

- Changes to `list_ticket_worktrees` — it already applies both guards correctly
- Changes to `ensure_worktree`, `add_worktree`, or any other worktree lifecycle functions beyond what `find_worktree_for_branch` calls
- Fixing any race condition where the main branch changes between the check and the worktree creation
- Server dispatch logic or callers outside `worktree.rs`

### Approach

All changes are in `apm-core/src/worktree.rs`.

#### Patch `find_worktree_for_branch`

Mirror the two guards already present in `list_ticket_worktrees`:

1. Canonicalize the main worktree root once before the loop, assigning to `main` (same variable name used in `list_ticket_worktrees`).

2. In the branch-match arm, before returning `current_path`, apply both guards:
   - Skip if `b` does not start with `"ticket/"` — a non-ticket branch in a worktree is never a valid return value.
   - Skip if `current_path.canonicalize()` equals `main` — the main worktree must never be returned as a dedicated worker worktree.

The only condition that allows a return is: branch matches, branch starts with `ticket/`, and the worktree path is not the main root.

#### Add a unit test

Add a test in the existing `#[cfg(test)] mod tests` block in `worktree.rs`. The test:

1. Creates a temp git repo with an initial commit on `main`.
2. Creates a `ticket/my-branch` local branch without adding a dedicated worktree.
3. Checks out `ticket/my-branch` in the main worktree via `git checkout`.
4. Calls `find_worktree_for_branch(repo, "ticket/my-branch")` and asserts the result is `None`.

This directly exercises the bug scenario: the main repo has the ticket branch checked out, so before the fix the function would return the main repo path instead of `None`.

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-05-28T07:31Z | — | new | philippepascal |
| 2026-05-28T07:37Z | new | groomed | philippepascal |
| 2026-05-28T07:38Z | groomed | in_design | philippepascal |
| 2026-05-28T07:39Z | in_design | specd | claude |
| 2026-05-28T07:44Z | specd | ready | philippepascal |
