+++
id = "3d73a43b"
title = "apm clean fail on epics with work tree"
state = "specd"
priority = 0
effort = 2
risk = 1
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/3d73a43b-apm-clean-fail-on-epics-with-work-tree"
created_at = "2026-04-17T18:28:11.666627Z"
updated_at = "2026-04-17T18:34:04.417239Z"
+++

## Spec

### Problem

When `apm clean --epics` is run, it attempts to delete each epic's local git branch directly via `git branch -d`. If a worktree is checked out on that branch (e.g. an epic worktree at `apm--worktrees/epic-<id>-<slug>`), git refuses the deletion with:

```
error: cannot delete branch 'epic/<id>-<slug>' used by worktree at '<path>'
```

The root cause is that `run_epic_clean()` in `apm/src/cmd/epic.rs` skips the worktree-removal step that the regular ticket cleaning flow already performs. In `apm-core/src/clean.rs`, `remove()` calls `worktree::remove_worktree()` before attempting branch deletion. The epic path has no equivalent guard.

The result is a partially-completed clean: some epics are deleted while others fail silently (the error is printed but the loop continues), leaving orphaned branch entries in `.apm/epics.toml` and dangling worktrees on disk.

### Acceptance criteria

- [ ] `apm clean --epics` successfully deletes an epic whose branch has an active worktree (worktree is removed first, then branch is deleted)
- [ ] `apm clean --epics` removes the worktree directory from disk before attempting branch deletion
- [ ] `apm clean --epics` succeeds for all eligible epics in a single run when some have worktrees and some do not
- [ ] If a worktree removal fails, the error is reported and that epic is skipped (branch deletion is not attempted), leaving `.apm/epics.toml` intact for that entry
- [ ] Epics without an associated worktree continue to be cleaned without any change in behaviour

### Out of scope

- Cleaning worktrees for regular ticket branches (already handled correctly in `apm-core/src/clean.rs`)
- Remote branch deletion behaviour (unchanged)
- Changing when an epic is considered eligible for cleaning (state-machine logic untouched)
- Cleaning epics that are currently active / in-progress (not targeted by `apm clean --epics`)

### Approach

**File to change**: `apm/src/cmd/epic.rs` — `run_epic_clean()` function (around lines 340–390).

**Pattern to follow**: Mirror what `apm-core/src/clean.rs` `remove()` does — call `worktree::find_worktree_for_branch()` then `worktree::remove_worktree()` before branch deletion.

**Steps**:

1. In `run_epic_clean()`, for each epic branch scheduled for deletion, before the `git branch -d` call:
   - Call `apm_core::worktree::find_worktree_for_branch(root, &branch_name)` to check for an active worktree.
   - If a worktree path is returned, call `apm_core::worktree::remove_worktree(root, &worktree_path)`.
   - If `remove_worktree` returns an error, print a clear message (e.g. `"skipping epic/<id>: could not remove worktree at <path>: <err>"`) and `continue` to the next epic — do **not** attempt the branch deletion or `.apm/epics.toml` removal for that entry.

2. No changes needed to `git_util::delete_local_branch`, `worktree.rs`, or `apm-core/src/clean.rs`.

3. The `worktree::find_worktree_for_branch` function parses `git worktree list --porcelain` output; it already works for epic branches since they follow the same `<worktrees_base>/epic-<id>-<slug>` path convention confirmed in the error output.

**Key constraint**: The worktree must be removed *before* the branch deletion attempt; git will refuse the deletion otherwise. The existing `remove_worktree` implementation already passes `--force` to `git worktree remove`, which is sufficient.

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-04-17T18:28Z | — | new | philippepascal |
| 2026-04-17T18:31Z | new | groomed | apm |
| 2026-04-17T18:31Z | groomed | in_design | philippepascal |
| 2026-04-17T18:34Z | in_design | specd | claude-0417-1831-18d8 |
