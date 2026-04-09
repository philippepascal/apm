+++
id = "95ef3505"
title = "apm-ui: inline effort/risk/priority editing in ticket detail"
state = "closed"
priority = 35
effort = 4
risk = 2
author = "apm"
agent = "65404"
branch = "ticket/95ef3505-apm-ui-inline-effort-risk-priority-editi"
created_at = "2026-03-31T06:13:16.584261Z"
updated_at = "2026-04-01T06:21:11.430803Z"
+++

## Spec

### Problem

In the ticket detail panel (Step 6), the `effort`, `risk`, and `priority` frontmatter fields are displayed as static text. Supervisors and spec-writers need to adjust these values frequently — particularly after reviewing a spec — without opening the full CodeMirror markdown editor introduced in Step 9.

Currently the only way to change these fields is via the CLI (`apm set <id> effort <n>`). The UI should provide click-to-edit inline controls directly in the detail panel header area so supervisors can update values with a single click and a keystroke.

The backend already exposes or will expose `PATCH /api/tickets/:id` (first introduced in Step 11 for priority reordering). This ticket extends that endpoint to accept `effort` and `risk` in addition to `priority`, and adds the corresponding inline UI controls for all three fields.

### Acceptance criteria

- [x] Clicking the `effort` value in the ticket detail panel activates an inline number input
- [x] Clicking the `risk` value in the ticket detail panel activates an inline number input
- [x] Clicking the `priority` value in the ticket detail panel activates an inline number input
- [x] Pressing Enter or blurring the input commits the change via `PATCH /api/tickets/:id`
- [x] Pressing Escape cancels the edit and restores the previous value without a network request
- [x] The UI reflects the updated value immediately after a successful PATCH (optimistic update via TanStack Query cache invalidation)
- [x] `PATCH /api/tickets/:id` body `{"effort":N}` updates the effort field in the ticket frontmatter and commits it to the ticket branch
- [x] `PATCH /api/tickets/:id` body `{"risk":N}` updates the risk field in the ticket frontmatter and commits it to the ticket branch
- [x] `PATCH /api/tickets/:id` body `{"priority":N}` updates the priority field in the ticket frontmatter and commits it to the ticket branch
- [x] Submitting a value outside the valid range (effort/risk: 1–10; priority: 0–255) shows an inline validation error and does not issue a PATCH request
- [x] If the PATCH request returns an error, the field reverts to its pre-edit value and a toast error is shown
- [x] Each inline control is keyboard-accessible: Tab focuses the field, Enter activates edit mode

### Out of scope

- Editing any other frontmatter fields (title, state, agent, author) inline — those are handled elsewhere
- The full markdown editor (CodeMirror) — that is Step 9
- The sync button (`POST /api/sync`) — that is the sibling ticket Step 13a
- Drag-and-drop priority reordering in the worker queue — that is Step 11
- Persisting the PATCH endpoint itself if Step 11 is not yet merged; this ticket may need to introduce `PATCH /api/tickets/:id` if it does not already exist

### Approach

**Backend — `apm-server`**

1. If `PATCH /api/tickets/:id` does not yet exist (Step 11 not merged), add it. The handler accepts a JSON body with any subset of `{"effort": N, "risk": N, "priority": N}` (partial update — unknown keys are ignored or rejected with 400).
2. For each provided field, call `ticket::set_field(&mut fm, field, value)` from `apm-core` (already handles effort, risk, priority with range validation returning an error for 0–255; note effort/risk semantic range 1–10 is enforced client-side only, server accepts 0–255).
3. Serialize the updated frontmatter back into the ticket file and commit it to the ticket branch using the same git-commit helper used by `PUT /api/tickets/:id/body`.
4. Return the updated ticket JSON (same shape as `GET /api/tickets/:id`).

**Frontend — `apm-ui`**

5. Create an `InlineNumberField` component (e.g. `src/components/InlineNumberField.tsx`) that:
   - Renders a styled span showing the current value by default
   - On click (or Enter when focused), switches to a `<input type="number">` pre-filled with the current value
   - On blur or Enter: calls the `onCommit(newValue)` callback; reverts on Escape
   - Accepts `min`/`max` props for client-side range validation; shows an inline error badge if violated (no network call)
6. In the ticket detail panel (`TicketDetail` or equivalent component), replace the static `effort`, `risk`, and `priority` display with `InlineNumberField` instances wired to a TanStack Query mutation:
   ```ts
   useMutation({
     mutationFn: (patch) => fetch(`/api/tickets/${id}`, { method: 'PATCH', body: JSON.stringify(patch) }),
     onSuccess: () => queryClient.invalidateQueries(['ticket', id]),
     onError: () => { revert(); toast.error('Update failed') },
   })
   ```
7. Use optimistic updates (via `onMutate` / `onError` rollback) so the value updates instantly in the UI.
8. Fields and their valid ranges:
   - `effort`: min=1, max=10
   - `risk`: min=1, max=10
   - `priority`: min=0, max=255

**Files changed**

- `apm-server/src/routes/tickets.rs` — add/extend PATCH handler
- `apm-ui/src/components/InlineNumberField.tsx` — new reusable component
- `apm-ui/src/components/TicketDetail.tsx` — wire in the three fields

### Open questions



### Amendment requests



## History

| When | From | To | By |
|------|------|----|----|
| 2026-03-31T06:13Z | — | new | apm |
| 2026-03-31T07:14Z | new | in_design | philippepascal |
| 2026-03-31T07:18Z | in_design | specd | claude-0331-0800-b7e2 |
| 2026-03-31T19:44Z | specd | ready | apm |
| 2026-04-01T05:45Z | ready | in_progress | philippepascal |
| 2026-04-01T05:53Z | in_progress | implemented | claude-0401-0545-1398 |
| 2026-04-01T06:00Z | implemented | accepted | apm |
| 2026-04-01T06:21Z | accepted | closed | apm-sync |