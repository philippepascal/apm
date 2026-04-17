+++
id = "3d73a43b"
title = "apm clean fail on epics with work tree"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/3d73a43b-apm-clean-fail-on-epics-with-work-tree"
created_at = "2026-04-17T18:28:11.666627Z"
updated_at = "2026-04-17T18:31:37.086826Z"
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

- [ ] At least an error message explaining the user what needs to be done, but better if this can be done automatically

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
| 2026-04-17T18:28Z | — | new | philippepascal |
| 2026-04-17T18:31Z | new | groomed | apm |
| 2026-04-17T18:31Z | groomed | in_design | philippepascal |