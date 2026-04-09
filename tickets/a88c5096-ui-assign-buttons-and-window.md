+++
id = "a88c5096"
title = "UI: assign buttons and window"
state = "closed"
priority = 0
effort = 3
risk = 2
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/a88c5096-ui-assign-buttons-and-window"
created_at = "2026-04-09T05:16:55.238687Z"
updated_at = "2026-04-09T06:30:36.703357Z"
+++

## Spec

### Problem

The ticket detail panel has a "Reassign to me" button that only assigns a ticket to the current user. There is no way in the UI to assign a ticket to another collaborator. This is limiting in a team setting where a user may want to hand off a ticket or triage work to others.

Additionally, the current button calls `POST /api/tickets/{id}/take`, an endpoint that does not exist on the server, so the feature is currently broken end-to-end.

The desired behaviour: clicking an "Assign" button opens a small picker listing all project collaborators plus an "Unassigned" option. Selecting an entry assigns or clears the owner and dismisses the picker. The existing `PATCH /api/tickets/:id` endpoint already accepts an `owner` field, so the main work is a new collaborators API endpoint and a frontend picker component.

### Acceptance criteria

- [x] The "Reassign to me" button is replaced by an "Assign" button in the `TransitionButtons` component
- [x] Clicking "Assign" opens a picker listing all project collaborators
- [x] The picker includes an "Unassigned" entry at the top to clear the owner
- [x] The current user is always present in the picker list
- [x] Selecting a collaborator calls `PATCH /api/tickets/:id` with `{ owner: username }` and dismisses the picker
- [x] Selecting "Unassigned" calls `PATCH /api/tickets/:id` with `{ owner: "" }` and dismisses the picker
- [x] After a successful assignment the ticket detail panel reflects the updated owner without a full page reload
- [x] While the assignment request is in-flight, the Assign button is disabled
- [x] If the assignment request fails, an error message is shown near the button
- [x] Pressing Escape or clicking outside the picker dismisses it without making any change
- [x] `GET /api/collaborators` returns the project collaborator list (from GitHub API or config.toml fallback)

### Out of scope

- Creating a `POST /api/tickets/:id/take` endpoint (the existing `PATCH /api/tickets/:id` owner field is sufficient)
- Inline owner editing in the ticket header (`InlineOwnerField` already handles that; this ticket only changes the `TransitionButtons` area)
- Batch assignment across multiple tickets
- Role-based restrictions on who can assign to whom
- Assignee validation on the server beyond what apm-core already enforces

### Approach

**Server — new `GET /api/collaborators` endpoint** (`apm-server/src/main.rs`)

1. Add `collaborators_handler`: calls `apm_core::config::resolve_collaborators()` on the loaded `AppState` config and returns a JSON array of strings, e.g. `["alice", "bob"]`. No new structs needed — `Json(Vec<String>)` serializes directly.
2. Register in `build_app()` under the protected router: `GET /api/collaborators -> collaborators_handler`.

**Frontend — new `AssignPicker` component** (`apm-ui/src/components/AssignPicker.tsx`)

Props: `{ ticketId: string, onDone: () => void }`

- On mount, fire `GET /api/collaborators` and `GET /api/me` in parallel via `useQuery`. Merge into a deduplicated sorted list; prepend an "Unassigned" sentinel (empty string).
- Render a small absolutely-positioned box (`role="listbox"`) listing each name as a clickable row.
- On row click: call `PATCH /api/tickets/:id` with `{ owner: selected }` via `useMutation`, then call `onDone()` on success.
- On Escape keydown or mousedown outside (both via `useEffect`): call `onDone()` without mutating.
- While mutation is pending: show `Loader2` spinner, disable all rows.
- On mutation error: show inline error, keep picker open for retry or manual dismiss.
- Styling: picker box `bg-gray-900 border border-gray-600 rounded shadow-lg p-1 min-w-48 z-50`; rows `px-3 py-1 text-sm rounded hover:bg-gray-700 cursor-pointer text-gray-200`; "Unassigned" row `italic text-gray-400`.

**Frontend — `TransitionButtons` changes** (`apm-ui/src/components/TicketDetail.tsx`)

1. Remove `reassignMutation` (called the non-existent `/take` endpoint) and `reassignError` state.
2. Add `showAssignPicker` boolean state (default `false`).
3. Replace the "Reassign to me" button with a `<div className="relative">` containing an "Assign" button that sets `showAssignPicker = true`, and conditionally renders `<AssignPicker>` beneath it.
4. In `AssignPicker`'s `onDone` callback: set `showAssignPicker = false`, then `queryClient.invalidateQueries` for `['ticket', ticket.id]` and `['tickets']`.
5. Remove `reassignMutation.isPending` from the `anyPending` expression and the `reassignError` display.

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-04-09T05:16Z | — | new | philippepascal |
| 2026-04-09T05:17Z | new | groomed | apm |
| 2026-04-09T05:29Z | groomed | in_design | philippepascal |
| 2026-04-09T05:36Z | in_design | specd | claude-0409-0529-5d68 |
| 2026-04-09T05:52Z | specd | ready | apm |
| 2026-04-09T05:55Z | ready | in_progress | philippepascal |
| 2026-04-09T05:59Z | in_progress | implemented | claude-0409-0555-5ff8 |
| 2026-04-09T06:30Z | implemented | closed | philippepascal |
