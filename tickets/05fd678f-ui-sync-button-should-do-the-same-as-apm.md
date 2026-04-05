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

- [ ] Clicking Sync in the UI (or pressing Shift+S) closes tickets whose branches have been merged into main
- [ ] Clicking Sync in the UI closes `implemented` tickets whose branch no longer exists
- [ ] After a successful sync that closes tickets, those tickets no longer appear in the supervisor board (or appear in `closed` state if "Show closed" is checked)
- [ ] A sync that closes no tickets still succeeds and refreshes the ticket list
- [ ] The response from `POST /api/sync` includes a `closed` count so the caller knows what happened

### Out of scope

- Interactive confirmation prompts (the server always auto-closes, mirroring `apm sync --auto-close`)
- Exposing the list of closed ticket IDs in the API response (count is enough)
- The `--no-aggressive` / aggressive sync config option — the server honours `config.sync.aggressive` automatically since it calls the same `sync::apply` path
- Any changes to the UI beyond consuming the refreshed ticket list (no toast notifications, no closed-ticket list)

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