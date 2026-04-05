+++
id = "05fd678f"
title = "UI sync button should do the same as apm sync"
state = "implemented"
priority = 0
effort = 2
risk = 2
author = "apm-ui"
owner = "philippepascal"
branch = "ticket/05fd678f-ui-sync-button-should-do-the-same-as-apm"
created_at = "2026-04-04T18:33:04.575258Z"
updated_at = "2026-04-05T22:17:54.460306Z"
+++

## Spec

### Problem

The UI sync button (Shift+S or the Sync button in SupervisorView) calls `POST /api/sync`, which currently only runs `git fetch` and `git sync_local_ticket_refs`. It does **not** run `sync::detect` or `sync::apply`, so it never detects or closes tickets whose branches have been merged into main.

The CLI `apm sync` command does the full job: fetch, sync refs, detect merge candidates (branches merged into main, or `implemented` tickets with a deleted branch), and apply closures. The server handler omits the detect-and-close step entirely.

The result: after a PR is merged, `apm sync` in the terminal will close the ticket, but clicking Sync in the UI has no effect on ticket state. Users who work primarily through the UI see stale tickets stuck in `implemented` or other non-terminal states indefinitely.

### Acceptance criteria

- [x] Clicking Sync in the UI (or pressing Shift+S) closes tickets whose branches have been merged into main
- [x] Clicking Sync in the UI closes `implemented` tickets whose branch no longer exists
- [x] After a successful sync that closes tickets, those tickets no longer appear in the supervisor board (or appear in `closed` state if "Show closed" is checked)
- [x] A sync that closes no tickets still succeeds and refreshes the ticket list
- [x] The response from `POST /api/sync` includes a `closed` count so the caller knows what happened

### Out of scope

- Interactive confirmation prompts (the server always auto-closes, mirroring `apm sync --auto-close`)
- Exposing the list of closed ticket IDs in the API response (count is enough)
- The `--no-aggressive` / aggressive sync config option — the server honours `config.sync.aggressive` automatically since it calls the same `sync::apply` path
- Any changes to the UI beyond consuming the refreshed ticket list (no toast notifications, no closed-ticket list)

### Approach

**File: `apm-server/src/main.rs`**

In `sync_handler`, after the existing `fetch_all` + `sync_local_ticket_refs` block, add:

```rust
let config = apm_core::config::Config::load(&root)?;
let candidates = apm_core::sync::detect(&root, &config)?;
let n_closed = candidates.close.len();
let aggressive = config.sync.aggressive;
if n_closed > 0 {
    apm_core::sync::apply(&root, &config, &candidates, "apm-ui", aggressive)?;
}
```

Include `closed` in the JSON response alongside the existing `branches` field:
```json
{ "branches": 12, "closed": 2 }
```

All of this runs inside the existing `spawn_blocking` closure since `detect` and `apply` are synchronous.

The UI side (`SupervisorView.tsx`) already invalidates the `tickets` query on sync success, so no UI changes are needed — the board will refresh automatically and show the newly-closed tickets correctly.

**Order of steps:**
1. Extend the `spawn_blocking` closure in `sync_handler` to call `sync::detect` then `sync::apply`
2. Return `closed` count in the response JSON
3. Add a regression test: create a git repo, merge a ticket branch into main, call the sync handler, assert the ticket is now `closed`

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-04-04T18:33Z | — | new | apm-ui |
| 2026-04-05T21:41Z | new | groomed | apm |
| 2026-04-05T21:41Z | groomed | in_design | philippepascal |
| 2026-04-05T21:43Z | in_design | specd | claude-0405-2141-spec7 |
| 2026-04-05T22:12Z | specd | ready | apm |
| 2026-04-05T22:13Z | ready | in_progress | philippepascal |
| 2026-04-05T22:17Z | in_progress | implemented | claude-0405-2213-4658 |
