+++
id = 16
title = "apm uses git plumbing to commit ticket changes without checkout"
state = "closed"
priority = 1
effort = 0
risk = 0
created = "2026-03-25"
updated = "2026-03-26"
+++

## Spec

### Problem

Committing to a branch without disturbing the working tree can be done via either
git worktrees or git plumbing commands (`hash-object`, `update-index`, `write-tree`,
`commit-tree`). The plumbing approach avoids any filesystem state and is more robust
in constrained environments.

### Acceptance criteria

- [x] APM can commit ticket changes to any branch without checking it out

### Out of scope

- Switching from the worktree approach to plumbing is a refinement; the worktree
  approach already satisfies the requirement

### Approach

The worktree-based `commit_to_branch` in `apm-core/src/git.rs` (implemented in PR #5)
satisfies the core requirement. A future ticket can pursue a pure-plumbing replacement
if the worktree approach proves problematic in practice.

## History

| Date | Actor | Transition | Note |
|------|-------|------------|------|
| 2026-03-25 | manual | new → new | Created as placeholder |
| 2026-03-26 | manual | new → closed | Requirement satisfied by worktree approach in git.rs |
