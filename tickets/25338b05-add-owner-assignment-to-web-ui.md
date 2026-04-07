+++
id = "25338b05"
title = "Add owner assignment to web UI"
state = "in_progress"
priority = 0
effort = 4
risk = 2
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/25338b05-add-owner-assignment-to-web-ui"
created_at = "2026-04-06T20:57:16.722499Z"
updated_at = "2026-04-07T00:41:36.777824Z"
depends_on = ["f38a9b24", "87fb645e"]
+++

## Spec

### Problem

The CLI supports assigning ticket owners via `apm assign <id> <username>` (and clearing with `apm assign <id> -`), and `apm list --owner` supports filtering by owner. The web UI partially surfaces the owner concept — `SupervisorView` has an owner filter dropdown and `TicketCard` shows the owner name on the card — but there is no way to view, set, or clear the owner field from the ticket detail panel.

The backend gap compounds the problem: `PATCH /api/tickets/:id` accepts only `effort`, `risk`, and `priority` in its request body. Even if the UI wanted to update ownership, there is no endpoint to call. The underlying `set_field("owner", ...)` function in `apm-core` already handles both assignment and clearing (via the sentinel value `"-"`), so the backend wire-up is straightforward.

The result is that owner assignment is effectively CLI-only — any team member using the web dashboard cannot manage ticket ownership without dropping to a terminal. This ticket adds: (1) a visible owner field in the ticket detail panel, (2) inline editing to assign or reassign an owner (with suggestions drawn from existing owners in the system), (3) a way to clear the owner, and (4) the backend PATCH support required to persist the change.

### Acceptance criteria

- [x] `PATCH /api/tickets/:id` accepts an `owner` field and persists it to the ticket frontmatter in git
- [x] `PATCH /api/tickets/:id` with `owner` set to an empty string or `"-"` clears the owner (sets it to None)
- [x] `PATCH /api/tickets/:id` that omits the `owner` field leaves the existing owner unchanged
- [x] `PATCH /api/tickets/:id` returns the updated ticket including the new owner value in the response body
- [x] The ticket detail panel displays the owner field; shows the username when assigned
- [x] The ticket detail panel shows a placeholder (e.g. "Unassigned") when no owner is set
- [x] Clicking the owner field in the detail panel activates an inline edit input
- [x] The inline input offers autocomplete suggestions drawn from the distinct owners already present in the ticket list
- [x] Submitting the inline input with a non-empty value assigns that owner and updates the display without a page reload
- [x] Submitting the inline input with an empty value clears the owner and updates the display without a page reload
- [x] Pressing Escape while editing the owner field cancels the edit and reverts to the previous display
- [x] After assigning or clearing an owner via the web UI, refreshing the page shows the persisted value

### Out of scope

- User account management or validation that the entered owner matches a known system user
- Permission enforcement (restricting who may change the owner)
- Bulk owner assignment via the web UI
- Notifications or webhooks triggered by owner changes
- Worker view (WorkerView) — owner display there is not changed by this ticket
- Any changes to the CLI assign command or apm-proxy

### Approach

**1. Backend — `apm-server/src/main.rs`**

Add `owner: Option<String>` to `PatchTicketRequest` (currently defined around lines 137-141). In the `patch_ticket` handler, after the existing effort/risk/priority processing, check if `owner` is `Some(v)`: if `v` is empty, call `set_field("owner", "-")` (the clear sentinel); otherwise call `set_field("owner", v)`. `TicketDetailResponse` already serializes the `owner` field (`skip_serializing_if = "Option::is_none"`), so no response struct changes are needed.

Add three server-side tests following the existing `patch_ticket` pattern:
- set owner to a new value -> persisted and returned
- set owner to empty string -> field cleared in frontmatter
- omit owner field -> existing value unchanged

**2. Frontend — new `InlineOwnerField` component**

Create `apm-ui/src/components/InlineOwnerField.tsx`. Model it after `InlineNumberField.tsx`:
- Props: `value: string | undefined`, `suggestions: string[]`, `onCommit: (v: string) => void`
- Renders current value as text with a click-to-edit affordance; shows placeholder when `value` is undefined
- On click: renders `<input>` with a `<datalist>` populated from `suggestions`
- On Enter or blur: calls `onCommit(inputValue)` (empty string = clear)
- On Escape: reverts to display mode without calling `onCommit`

**3. Frontend — `TicketDetail.tsx`**

Add an owner row in the metadata section alongside effort/risk/priority. Wire `InlineOwnerField` to call `PATCH /api/tickets/:id` with `{ owner: v }` via the existing React Query mutation. On success, invalidate or update the ticket cache so the UI reflects the change immediately.

Accept a new `availableOwners: string[]` prop and pass it to `InlineOwnerField` as `suggestions`.

**4. Frontend — `SupervisorView.tsx`**

`SupervisorView` already computes distinct owners from the ticket list (lines ~108-112). Pass this array as the `availableOwners` prop to `TicketDetail` — no additional network request needed.

**Implementation order:** backend (struct + handler + tests) -> `InlineOwnerField` component -> wire into `TicketDetail` -> pass suggestions from `SupervisorView`.

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-04-06T20:57Z | — | new | philippepascal |
| 2026-04-06T21:22Z | new | groomed | apm |
| 2026-04-06T23:13Z | groomed | in_design | philippepascal |
| 2026-04-06T23:21Z | in_design | groomed | apm |
| 2026-04-06T23:22Z | groomed | in_design | philippepascal |
| 2026-04-06T23:32Z | in_design | specd | claude-0406-1735-b2e1 |
| 2026-04-07T00:15Z | specd | ready | apm |
| 2026-04-07T00:41Z | ready | in_progress | philippepascal |
