+++
id = 21
title = "commit_to_branch writes ticket files to working tree"
state = "implemented"
priority = 9
effort = 1
risk = 2
branch = "ticket/0021-commit-to-branch-writes-ticket-files-to-"
created = "2026-03-26"
updated = "2026-03-26"
+++

## Spec

### Problem

`commit_to_branch` always writes the file content to the current working
directory as a "local cache" before performing the worktree commit (lines 81–85
of `apm-core/src/git.rs`):

```rust
// Always update the local cache first.
let local_path = root.join(rel_path);
std::fs::write(&local_path, content)?;
```

This means every `apm state`, `apm set`, and `apm new` call modifies files on
disk in whatever branch is currently checked out (usually `main`). The commit
correctly lands on the ticket branch via the worktree, but the working tree is
left dirty with an uncommitted change — confusing and incorrect.

### Acceptance criteria

- [ ] `apm state`, `apm set`, `apm new` no longer modify any file in the current
  working tree when the current branch is not the target branch
- [ ] The file is still written inside the worktree before committing (this is
  how it reaches the target branch)
- [ ] The "already on target branch" fast-path (when `current_branch == branch`)
  still writes to the working tree and commits directly — that case is correct
- [ ] All existing tests continue to pass

### Out of scope

- Changing when/whether pushes happen (tracked in #22)

### Approach

Remove the unconditional local-cache write at the top of `commit_to_branch`.
The file write inside `try_worktree_commit` (line 139) already handles writing
into the worktree before committing, so deleting lines 81–85 is sufficient.
The direct-commit fast-path (`current_branch == branch`) uses `git add` on the
existing working-tree file, which is correct as-is.

## History

| Date | Actor | Transition | Note |
|------|-------|------------|------|
| 2026-03-26 | agent | new → ready | |
