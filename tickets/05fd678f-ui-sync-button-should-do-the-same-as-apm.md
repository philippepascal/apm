+++
id = "05fd678f"
title = "UI sync button should do the same as apm sync"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "apm-ui"
owner = "philippepascal"
branch = "ticket/05fd678f-ui-sync-button-should-do-the-same-as-apm"
created_at = "2026-04-04T18:33:04.575258Z"
updated_at = "2026-04-05T21:41:31.174633Z"
+++

## Spec

### Problem

The UI sync button (Shift+S or the Sync button in SupervisorView) calls `POST /api/sync`, which currently only runs `git fetch` and `git sync_local_ticket_refs`. It does **not** run `sync::detect` or `sync::apply`, so it never detects or closes tickets whose branches have been merged into main.

The CLI `apm sync` command does the full job: fetch, sync refs, detect merge candidates (branches merged into main, or `implemented` tickets with a deleted branch), and apply closures. The server handler omits the detect-and-close step entirely.

The result: after a PR is merged, `apm sync` in the terminal will close the ticket, but clicking Sync in the UI has no effect on ticket state. Users who work primarily through the UI see stale tickets stuck in `implemented` or other non-terminal states indefinitely.

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
| 2026-04-04T18:33Z | — | new | apm-ui |
| 2026-04-05T21:41Z | new | groomed | apm |
| 2026-04-05T21:41Z | groomed | in_design | philippepascal |