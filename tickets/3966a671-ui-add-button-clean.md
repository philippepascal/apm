+++
id = "3966a671"
title = "UI: add button clean"
state = "in_progress"
priority = 0
effort = 2
risk = 2
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/3966a671-ui-add-button-clean"
created_at = "2026-04-04T18:42:57.517863Z"
updated_at = "2026-04-05T22:18:11.315013Z"
+++

## Spec

### Problem

The supervisor view toolbar has buttons for creating tickets, creating epics, and syncing with remote — but no way to trigger `apm clean` from the UI. Cleaning removes stale git worktrees for tickets that are closed or whose branches have been merged into main. Users must drop to the CLI and run `apm clean` manually after a batch of work completes, which is friction in an otherwise UI-driven workflow.

The desired behaviour is a "Clean" button in the supervisor toolbar that calls a new `POST /api/clean` server endpoint. The endpoint runs the same safe-clean logic as `apm clean` (no --force, no --remote, no --branches): it collects candidates via `apm_core::clean::candidates()` and removes each stale worktree via `clean::remove()`. The UI reflects the in-progress state with a spinner and surfaces errors inline, mirroring the existing Sync button pattern.

### Acceptance criteria

- [x] A "Clean" button appears in the SupervisorView toolbar, positioned next to the "Sync" button
- [x] Clicking "Clean" sends a `POST /api/clean` request to the server
- [x] While the request is in progress, the "Clean" button displays a spinning loader icon (disabled, same pattern as Sync)
- [x] On success, the ticket list is refreshed (TanStack Query invalidates `['tickets']` and `['ticket']` keys)
- [x] On error, an inline error message appears in the toolbar (same pattern as `syncError` state)
- [x] `POST /api/clean` returns `{ "removed": N }` where N is the number of worktrees removed
- [x] `POST /api/clean` skips worktrees that have modified tracked files (dirty worktrees are not removed)
- [ ] `POST /api/clean` returns `501 Not Implemented` when the server has no git root (in-memory mode)

### Out of scope

- `--force` clean (removing unmerged branches or dirty worktrees) — not exposed via the UI
- `--remote` / `--older-than` remote branch deletion
- `--branches` local branch deletion alongside worktrees
- Dry-run preview before cleaning
- Keyboard shortcut for the clean action
- Confirmation dialog before cleaning

### Approach

Server changes in apm-server/src/main.rs:

1. Add async fn clean_handler modelled on sync_handler:
   - Guard: return 501 Not Implemented if state.git_root() is None
   - In spawn_blocking: load Config, call apm_core::clean::candidates (force=false, untracked=false, dry_run=false), then call apm_core::clean::remove for each non-dirty candidate (force=false, branches=false), count removals
   - Return JSON with "removed" count

2. Register route("/api/clean", post(clean_handler)) in both router blocks (authenticated and unauthenticated), same pattern as /api/sync

UI changes in apm-ui/src/components/supervisor/SupervisorView.tsx:

3. Add postClean async function calling POST /api/clean, returning removed count

4. Add cleanMutation via useMutation mirroring syncMutation:
   onSuccess: invalidate tickets and ticket query keys, clear cleanError
   onError: set cleanError state

5. Add cleanError string state (same pattern as syncError)

6. Add Clean button in toolbar next to Sync button:
   - Icon: Trash2 from lucide-react (add to import)
   - Label: Clean, title: Remove stale worktrees
   - Disabled with spinner while cleanMutation.isPending
   - cleanError shown as red text inline

Order: server handler + route first, then UI fetch + mutation + button, then cargo test --workspace (no new tests needed; clean logic lives in apm-core which already has its own tests)

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-04-04T18:42Z | — | new | philippepascal |
| 2026-04-05T21:41Z | new | groomed | apm |
| 2026-04-05T21:43Z | groomed | in_design | philippepascal |
| 2026-04-05T21:46Z | in_design | specd | claude-0405-2143-s7w2 |
| 2026-04-05T22:12Z | specd | ready | apm |
| 2026-04-05T22:18Z | ready | in_progress | philippepascal |