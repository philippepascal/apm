+++
id = "061d0ac1"
title = "Add missing git helpers to git_util.rs"
state = "groomed"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/061d0ac1-add-missing-git-helpers-to-git-util-rs"
created_at = "2026-04-12T17:29:22.472769Z"
updated_at = "2026-04-12T17:30:27.527161Z"
epic = "6062f74f"
target_branch = "epic/6062f74f-consolidate-git-operations-into-git-util"
+++

## Spec

### Problem

21 raw `Command::new("git")` calls exist across clean.rs, epic.rs, init.rs, start.rs, and worktree.rs — bypassing git_util.rs entirely. These modules construct git commands directly, duplicating patterns and leaking git implementation details.

git_util.rs should be the only module that knows how to talk to git. Several helpers are missing from its public API:

- `is_worktree_dirty(path) -> bool` (used in clean.rs ×2)
- `local_branch_exists(root, branch) -> bool` (used in clean.rs ×2, worktree.rs ×1)
- `delete_local_branch(root, branch)` (used in clean.rs)
- `prune_remote_tracking(root, branch)` (used in clean.rs)
- `stage_files(root, files)` (used in init.rs, epic.rs)
- `commit(root, message)` (used in init.rs, epic.rs)
- `git_config_get(root, key) -> Option<String>` (used in start.rs)
- `merge_ref(root, refname)` (used in start.rs — existing merge helpers are coupled to the "merge + push" workflow)
- `is_file_tracked(root, path) -> bool` (used in worktree.rs)

Additionally, some callers duplicate helpers that already exist: init.rs reimplements `has_commits`, epic.rs reimplements `fetch_branch` and `remove_worktree`.

This ticket adds the missing helpers. Subsequent tickets update each caller module.

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
| 2026-04-12T17:30Z | new | groomed | apm |
