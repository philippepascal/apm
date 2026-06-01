+++
id = "1469175e"
title = "apm refresh-epic --merge: push after local merge so downstream sees the refresh"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/1469175e-apm-refresh-epic-merge-push-after-local-"
created_at = "2026-05-31T03:26:11.802159Z"
updated_at = "2026-06-01T02:53:29.892877Z"
+++

## Spec

### Problem

`apm refresh-epic --merge` merges the default branch into the epic worktree locally but does not push to origin. The dispatch path in `apm start` calls `remote_branch_tip`, which prefers `origin/<epic-branch>` when that ref exists. Any ticket dispatched after a local-only merge therefore receives the pre-merge epic content. The refresh is silently ineffective for all downstream workers until the supervisor pushes manually.

This asymmetry was confirmed in practice on the syn project: `apm refresh-epic <id> --merge` completed successfully, but a subsequent `apm start` on a ticket in that epic dispatched from the stale `origin/<epic-branch>` tip. The `--pr` path (lines 203–225 of `apm/src/cmd/epic.rs`) already calls `push_branch_tracking` before opening the PR; the `--merge` path has no equivalent step.

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
| 2026-05-31T03:26Z | — | new | philippepascal |
| 2026-06-01T02:52Z | new | groomed | philippepascal |
| 2026-06-01T02:53Z | groomed | in_design | philippepascal |