+++
id = "4a8d75c7"
title = "UI sync popup modal"
state = "in_progress"
priority = 0
effort = 3
risk = 2
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/4a8d75c7-ui-sync-popup-modal"
created_at = "2026-04-17T20:20:40.428309Z"
updated_at = "2026-04-17T21:51:10.739932Z"
depends_on = ["5473a0e6"]
+++

## Spec

### Problem

The Sync button in the supervisor header currently fires `POST /api/sync` immediately, shows a spinner while it runs, and surfaces errors only as a small inline red string that disappears on the next success. Users cannot see what sync actually did: how many tickets were closed, which ones, whether the git fetch succeeded, or how many branches are now visible.

The Clean button solves this for the clean operation by opening a modal with a log pane. Sync should follow the same pattern so that the outcome of every sync run is visible and the UI is consistent.

The current `sync_handler` returns a small structured JSON (`branches`, `closed`, `fetch_error`) rather than a human-readable log. The handler must be updated to build and return a log string, and the UI must display it in a modal instead of running silently.

### Acceptance criteria

- [x] Clicking the Sync button opens a modal dialog instead of immediately running sync
- [x] The modal contains a "Run" button that triggers the sync operation when clicked
- [x] After sync completes, the modal log pane shows how many ticket branches are visible
- [x] After sync completes, the modal log pane shows how many tickets were closed
- [x] If git fetch encountered an error, it is shown as a warning line in the log pane
- [x] The Run button shows a spinner and is disabled while sync is in progress
- [x] The modal has a "Close" button that dismisses it at any time
- [x] Pressing Escape dismisses the modal
- [x] The Shift+S keyboard shortcut opens the sync modal instead of triggering sync directly
- [x] The ticket list is refreshed automatically after a successful sync run
- [x] The inline `syncError` text and `syncMutation` in `SupervisorView` are removed

### Out of scope

- Adding options or parameters to the sync operation (no dry-run, no selective sync)
- Changing the underlying logic of what sync does (fetch, ref sync, push, close detection)
- Streaming log output in real time — log is returned as a single string after completion
- Per-ticket close details beyond the count (ticket IDs, titles of closed tickets)

### Approach

#### Backend — apm-server/src/handlers/maintenance.rs

Update `sync_handler` to build a `Vec<String>` log instead of returning a bare `fetch_error` field:

- If `fetch_all` fails: push `"warning: git fetch failed: <error>"`
- After `sync_non_checked_out_refs`: push `"synced non-checked-out refs"`
- After `push_default_branch`: push `"pushed default branch"` (or omit silently)
- After `detect`+`apply`: push `"closed N ticket(s)"` — if n==0 push `"no tickets to close"`
- At end: push `"N ticket branch(es) visible"`

Return `{ "log": log.join("\n"), "branches": branches, "closed": closed }` — drop the top-level `fetch_error` field (it now lives in the log string). The existing test `sync_in_memory_returns_not_implemented` continues to pass unchanged.

#### Store — apm-ui/src/store/useLayoutStore.ts

Add `syncOpen: boolean` (initial: false) and `setSyncOpen: (v: boolean) => void` alongside the existing `cleanOpen`/`setCleanOpen` pair.

#### New component — apm-ui/src/components/SyncModal.tsx

Mirror `CleanModal.tsx` exactly, with these differences:
- No options section — sync takes no parameters
- `SyncResponse` type: `{ log: string; branches: number; closed: number }`
- `mutationFn`: `POST /api/sync` with no body, returns `SyncResponse`
- `onSuccess`: `setLog(data.log)`, invalidate `['tickets']` and `['ticket']` query keys
- `onError`: `setLog(err.message)`
- Footer: "Close" button (following ticket 5473a0e6 convention) + "Run" button with spinner while pending
- Reset `log` state and call `mutation.reset()` when `open` transitions to false
- Escape key handler identical to `CleanModal`

#### SupervisorView — apm-ui/src/components/supervisor/SupervisorView.tsx

- Add `setSyncOpen` from `useLayoutStore`
- Remove: `syncError` state, `postSync` function, `syncMutation`
- Sync button `onClick`: `setSyncOpen(true)` — remove `disabled` and spinner JSX
- Keyboard handler (Shift+S): call `setSyncOpen(true)` instead of `syncMutation.mutate()`
- Remove the `syncError` error `<span>` from the header

#### WorkScreen — apm-ui/src/components/WorkScreen.tsx

Import `SyncModal` and `syncOpen`/`setSyncOpen` from the store. Mount `<SyncModal open={syncOpen} onOpenChange={setSyncOpen} />` in both render paths where `CleanModal` is currently mounted (lines ~171 and ~189).

### Backend — apm-server/src/handlers/maintenance.rs

Update `sync_handler` to build a `Vec<String>` log instead of returning a bare `fetch_error` field:

- If `fetch_all` fails: push `"warning: git fetch failed: <error>"`
- After `sync_non_checked_out_refs`: push `"synced non-checked-out refs"`
- After `push_default_branch`: push `"pushed default branch"` (or omit silently)
- After `detect`+`apply`: push `"closed N ticket(s)"` — if n==0 push `"no tickets to close"`
- At end: push `"N ticket branch(es) visible"`

Return `{ "log": log.join("\n"), "branches": branches, "closed": closed }` — drop the top-level `fetch_error` field (it now lives in the log string). The existing test `sync_in_memory_returns_not_implemented` continues to pass unchanged.

### Store — apm-ui/src/store/useLayoutStore.ts

Add `syncOpen: boolean` (initial: false) and `setSyncOpen: (v: boolean) => void` alongside the existing `cleanOpen`/`setCleanOpen` pair.

### New component — apm-ui/src/components/SyncModal.tsx

Mirror `CleanModal.tsx` exactly, with these differences:
- No options section — sync takes no parameters
- `SyncResponse` type: `{ log: string; branches: number; closed: number }`
- `mutationFn`: `POST /api/sync` with no body, returns `SyncResponse`
- `onSuccess`: `setLog(data.log)`, invalidate `['tickets']` and `['ticket']` query keys
- `onError`: `setLog(err.message)`
- Footer: "Close" button (following ticket 5473a0e6 convention) + "Run" button with spinner while pending
- Reset `log` state and call `mutation.reset()` when `open` transitions to false
- Escape key handler identical to `CleanModal`

### SupervisorView — apm-ui/src/components/supervisor/SupervisorView.tsx

- Add `setSyncOpen` from `useLayoutStore`
- Remove: `syncError` state, `postSync` function, `syncMutation`
- Sync button `onClick`: `setSyncOpen(true)` — remove `disabled` and spinner JSX
- Keyboard handler (Shift+S): call `setSyncOpen(true)` instead of `syncMutation.mutate()`
- Remove the `syncError` error `<span>` from the header

### WorkScreen — apm-ui/src/components/WorkScreen.tsx

Import `SyncModal` and `syncOpen`/`setSyncOpen` from the store. Mount `<SyncModal open={syncOpen} onOpenChange={setSyncOpen} />` in both render paths where `CleanModal` is currently mounted (lines ~171 and ~189).

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-04-17T20:20Z | — | new | philippepascal |
| 2026-04-17T20:23Z | new | groomed | apm |
| 2026-04-17T20:32Z | groomed | in_design | philippepascal |
| 2026-04-17T20:36Z | in_design | specd | claude-0417-2032-c010 |
| 2026-04-17T21:45Z | specd | ready | apm |
| 2026-04-17T21:51Z | ready | in_progress | philippepascal |