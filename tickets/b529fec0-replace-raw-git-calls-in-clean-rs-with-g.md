+++
id = "b529fec0"
title = "Replace raw git calls in clean.rs with git_util helpers"
state = "new"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/b529fec0-replace-raw-git-calls-in-clean-rs-with-g"
created_at = "2026-04-12T17:29:27.980274Z"
updated_at = "2026-04-12T17:29:27.980274Z"
epic = "6062f74f"
target_branch = "epic/6062f74f-consolidate-git-operations-into-git-util"
depends_on = ["061d0ac1"]
+++

## Spec

### Problem

clean.rs has 6 raw `Command::new("git")` calls that should use git_util helpers:

1. `git -C <path> status --porcelain` (line ~49) — check worktree dirtiness → `git_util::is_worktree_dirty()`
2. `git -C <path> status --porcelain` (line ~156) — same check, different call site → `git_util::is_worktree_dirty()`
3. `git rev-parse --verify refs/heads/{branch}` (line ~181) — check local branch exists → `git_util::local_branch_exists()`
4. `git rev-parse --verify refs/heads/{branch}` (line ~234) — same pattern → `git_util::local_branch_exists()`
5. `git branch -D {branch}` (line ~272) — delete local branch → `git_util::delete_local_branch()`
6. `git branch -dr origin/{branch}` (line ~300) — prune remote tracking ref → `git_util::prune_remote_tracking()`

After this ticket, clean.rs should have zero `Command::new("git")` or `use std::process::Command` — all git interaction goes through git_util.

Depends on the git_util helpers ticket landing first.

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
| 2026-04-12T17:29Z | — | new | philippepascal |