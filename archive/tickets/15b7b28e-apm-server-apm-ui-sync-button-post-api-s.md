+++
id = "15b7b28e"
title = "apm-server + apm-ui: sync button (POST /api/sync)"
state = "closed"
priority = 38
effort = 3
risk = 2
author = "apm"
agent = "89531"
branch = "ticket/15b7b28e-apm-server-apm-ui-sync-button-post-api-s"
created_at = "2026-03-31T06:13:15.004948Z"
updated_at = "2026-04-01T06:20:52.825267Z"
+++

## Spec

### Problem

The UI will have no way to pull the latest ticket state from git branches once it is running. Without a sync mechanism, the browser shows stale data until the server process is restarted. A single button press should trigger the same operations as `apm sync --offline`: refresh local ticket-branch refs (and optionally fetch from remote), then return up-to-date ticket data to the frontend.

This ticket adds the `POST /api/sync` endpoint to `apm-server` and the corresponding sync button (with keyboard shortcut and loading state) to `apm-ui`. It does not auto-accept or auto-close tickets — those are destructive, confirmation-requiring operations that belong in a later ticket.

### Acceptance criteria

- [x] `POST /api/sync` returns HTTP 200 with a JSON body containing at least `{ "branches": <count> }`
- [x] `POST /api/sync` calls `git::sync_local_ticket_refs` so local branch refs are refreshed before the response is sent
- [x] `POST /api/sync` attempts `git::fetch_all`; if the fetch fails (e.g. no remote configured) the endpoint still returns 200 and includes a `"fetch_error"` field in the response body
- [x] After `POST /api/sync` completes, a subsequent `GET /api/tickets` returns ticket data that reflects the refreshed branch state
- [x] The UI renders a Sync button in the supervisorview header or an equivalent top-level location visible on the main screen
- [x] Clicking the Sync button disables it and shows a loading indicator while the request is in-flight
- [x] On success, the Sync button re-enables and TanStack Query invalidates all ticket queries so the swimlanes and detail panel refresh with fresh data
- [x] On failure (non-2xx or network error), the Sync button re-enables and an error message is shown to the user
- [x] A keyboard shortcut (`Shift+S`) triggers the same sync action as clicking the button

### Out of scope

- Auto-accepting merged tickets (that is a separate user-confirmation flow, not part of this ticket)
- Auto-closing tickets (same reason — requires user confirmation)
- Any conflict resolution or merge logic
- Inline effort/risk/priority editing (covered by ticket 13b)
- Streaming sync progress — the response is returned only after sync completes

### Approach

**Server side (`apm-server` crate — added in Steps 1-3):**

1. Add a `POST /api/sync` axum handler. Run sync work on `tokio::task::spawn_blocking` since the underlying calls are synchronous:
   - Call `git::fetch_all(root)` — capture any error as a string rather than returning HTTP 500.
   - Call `git::sync_local_ticket_refs(root)` — refresh local refs.
   - Call `git::ticket_branches(root)` — count visible branches for the response.
2. Return `200 OK` with `application/json`:
   - Success: `{ "branches": 12 }`
   - Fetch failed but sync succeeded: `{ "branches": 12, "fetch_error": "no remote origin" }`
3. No server-side ticket cache exists (per the Step 2 design, `GET /api/tickets` reads from git on every call), so the refreshed refs are automatically visible to subsequent requests with no extra work.

**UI side (`apm-ui/`):**

4. Add a `useMutation` (TanStack Query) that POSTs to `/api/sync`.
5. `onSuccess`: call `queryClient.invalidateQueries({ queryKey: ['tickets'] })` to refetch all ticket data.
6. `onError`: surface the error via a shadcn/ui `Toast` or inline alert near the button.
7. Render a shadcn/ui `<Button>` in the supervisorview header (or shared top bar). While mutation is pending: disabled + spinner icon. Otherwise: normal state with a tooltip showing the keyboard shortcut.
8. Register a global keydown handler for `Shift+S` (skip when an input/textarea/editor is focused) to fire the mutation. Use a `useEffect` with cleanup or a hotkey library already present in the project.

### Open questions



### Amendment requests

- [x] Change keyboard shortcut from lowercase `s` to `Shift+S` in both the Acceptance Criteria and the Approach — lowercase `s` is unsafe as a global shortcut (fires unexpectedly when returning focus from modals or other components)

## History

| When | From | To | By |
|------|------|----|----|
| 2026-03-31T06:13Z | — | new | apm |
| 2026-03-31T07:10Z | new | in_design | philippepascal |
| 2026-03-31T07:14Z | in_design | specd | claude-0331-spec-15b7 |
| 2026-03-31T18:14Z | specd | ammend | claude-0331-1200-a7b9 |
| 2026-03-31T19:08Z | ammend | in_design | philippepascal |
| 2026-03-31T19:12Z | in_design | specd | claude-0331-1430-b2c4 |
| 2026-03-31T19:44Z | specd | ready | apm |
| 2026-04-01T04:59Z | ready | in_progress | philippepascal |
| 2026-04-01T05:03Z | in_progress | implemented | claude-0401-0459-4ff0 |
| 2026-04-01T05:13Z | implemented | accepted | apm-sync |
| 2026-04-01T06:20Z | accepted | closed | apm-sync |