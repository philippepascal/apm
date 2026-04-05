+++
id = "3966a671"
title = "UI: add button clean"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/3966a671-ui-add-button-clean"
created_at = "2026-04-04T18:42:57.517863Z"
updated_at = "2026-04-05T21:43:47.717581Z"
+++

## Spec

### Problem

The supervisor view toolbar has buttons for creating tickets, creating epics, and syncing with remote — but no way to trigger `apm clean` from the UI. Cleaning removes stale git worktrees for tickets that are closed or whose branches have been merged into main. Users must drop to the CLI and run `apm clean` manually after a batch of work completes, which is friction in an otherwise UI-driven workflow.

The desired behaviour is a "Clean" button in the supervisor toolbar that calls a new `POST /api/clean` server endpoint. The endpoint runs the same safe-clean logic as `apm clean` (no --force, no --remote, no --branches): it collects candidates via `apm_core::clean::candidates()` and removes each stale worktree via `clean::remove()`. The UI reflects the in-progress state with a spinner and surfaces errors inline, mirroring the existing Sync button pattern.

### Acceptance criteria

- [ ] A "Clean" button appears in the SupervisorView toolbar, positioned next to the "Sync" button
- [ ] Clicking "Clean" sends a `POST /api/clean` request to the server
- [ ] While the request is in progress, the "Clean" button displays a spinning loader icon (disabled, same pattern as Sync)
- [ ] On success, the ticket list is refreshed (TanStack Query invalidates `['tickets']` and `['ticket']` keys)
- [ ] On error, an inline error message appears in the toolbar (same pattern as `syncError` state)
- [ ] `POST /api/clean` returns `{ "removed": N }` where N is the number of worktrees removed
- [ ] `POST /api/clean` skips worktrees that have modified tracked files (dirty worktrees are not removed)
- [ ] `POST /api/clean` returns `501 Not Implemented` when the server has no git root (in-memory mode)

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
| 2026-04-04T18:42Z | — | new | philippepascal |
| 2026-04-05T21:41Z | new | groomed | apm |
| 2026-04-05T21:43Z | groomed | in_design | philippepascal |