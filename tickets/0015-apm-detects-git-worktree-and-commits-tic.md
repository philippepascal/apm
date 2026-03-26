+++
id = 15
title = "apm detects git worktree and commits ticket changes to main worktree"
state = "closed"
priority = 5
effort = 0
risk = 0
created = "2026-03-25"
updated = "2026-03-26"
+++

## Spec

### Problem

When APM runs from inside a git worktree (common when an agent operates in a
worktree checkout), ticket commits need to land on the correct branch without
disturbing the working tree.

### Acceptance criteria

- [x] `git::commit_to_branch` detects if the working directory is already on the target branch and commits directly
- [x] Otherwise uses a temporary git worktree to commit without disturbing the current working tree
- [x] Worktree is cleaned up after the commit regardless of success or failure

### Out of scope

- Pure git plumbing approach (tracked separately in #16)

### Approach

Implemented in `apm-core/src/git.rs` as part of the branch-per-ticket redesign (PR #5).
`commit_to_branch` checks `current_branch()` against the target branch; if matching,
commits directly; otherwise creates a temp worktree under `$TMPDIR/apm-<pid>-<nanos>-<branch>`,
commits there, removes the worktree, and pushes.

## History

| Date | Actor | Transition | Note |
|------|-------|------------|------|
| 2026-03-25 | manual | new → new | Created as placeholder |
| 2026-03-26 | manual | new → closed | Implemented in git.rs via PR #5 |
