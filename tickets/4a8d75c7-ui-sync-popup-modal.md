+++
id = "4a8d75c7"
title = "UI sync popup modal"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/4a8d75c7-ui-sync-popup-modal"
created_at = "2026-04-17T20:20:40.428309Z"
updated_at = "2026-04-17T20:32:04.884518Z"
depends_on = ["5473a0e6"]
+++

## Spec

### Problem

The Sync button in the supervisor header currently fires `POST /api/sync` immediately, shows a spinner while it runs, and surfaces errors only as a small inline red string that disappears on the next success. Users cannot see what sync actually did: how many tickets were closed, which ones, whether the git fetch succeeded, or how many branches are now visible.

The Clean button solves this for the clean operation by opening a modal with a log pane. Sync should follow the same pattern so that the outcome of every sync run is visible and the UI is consistent.

The current `sync_handler` returns a small structured JSON (`branches`, `closed`, `fetch_error`) rather than a human-readable log. The handler must be updated to build and return a log string, and the UI must display it in a modal instead of running silently.

### Acceptance criteria

- [ ] Clicking the Sync button opens a modal dialog instead of immediately running sync
- [ ] The modal contains a "Run" button that triggers the sync operation when clicked
- [ ] After sync completes, the modal log pane shows how many ticket branches are visible
- [ ] After sync completes, the modal log pane shows how many tickets were closed
- [ ] If git fetch encountered an error, it is shown as a warning line in the log pane
- [ ] The Run button shows a spinner and is disabled while sync is in progress
- [ ] The modal has a "Close" button that dismisses it at any time
- [ ] Pressing Escape dismisses the modal
- [ ] The Shift+S keyboard shortcut opens the sync modal instead of triggering sync directly
- [ ] The ticket list is refreshed automatically after a successful sync run
- [ ] The inline `syncError` text and `syncMutation` in `SupervisorView` are removed

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
| 2026-04-17T20:20Z | — | new | philippepascal |
| 2026-04-17T20:23Z | new | groomed | apm |
| 2026-04-17T20:32Z | groomed | in_design | philippepascal |