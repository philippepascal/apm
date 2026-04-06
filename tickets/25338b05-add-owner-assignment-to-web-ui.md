+++
id = "25338b05"
title = "Add owner assignment to web UI"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/25338b05-add-owner-assignment-to-web-ui"
created_at = "2026-04-06T20:57:16.722499Z"
updated_at = "2026-04-06T23:22:20.980324Z"
depends_on = ["f38a9b24", "87fb645e"]
+++

## Spec

### Problem

The CLI supports assigning ticket owners via `apm assign <id> <username>` (and clearing with `apm assign <id> -`), and `apm list --owner` supports filtering by owner. The web UI partially surfaces the owner concept — `SupervisorView` has an owner filter dropdown and `TicketCard` shows the owner name on the card — but there is no way to view, set, or clear the owner field from the ticket detail panel.

The backend gap compounds the problem: `PATCH /api/tickets/:id` accepts only `effort`, `risk`, and `priority` in its request body. Even if the UI wanted to update ownership, there is no endpoint to call. The underlying `set_field("owner", ...)` function in `apm-core` already handles both assignment and clearing (via the sentinel value `"-"`), so the backend wire-up is straightforward.

The result is that owner assignment is effectively CLI-only — any team member using the web dashboard cannot manage ticket ownership without dropping to a terminal. This ticket adds: (1) a visible owner field in the ticket detail panel, (2) inline editing to assign or reassign an owner (with suggestions drawn from existing owners in the system), (3) a way to clear the owner, and (4) the backend PATCH support required to persist the change.

### Acceptance criteria

- [ ] `PATCH /api/tickets/:id` accepts an `owner` field and persists it to the ticket frontmatter in git
- [ ] `PATCH /api/tickets/:id` with `owner` set to an empty string or `"-"` clears the owner (sets it to None)
- [ ] `PATCH /api/tickets/:id` that omits the `owner` field leaves the existing owner unchanged
- [ ] `PATCH /api/tickets/:id` returns the updated ticket including the new owner value in the response body
- [ ] The ticket detail panel displays the owner field; shows the username when assigned
- [ ] The ticket detail panel shows a placeholder (e.g. "Unassigned") when no owner is set
- [ ] Clicking the owner field in the detail panel activates an inline edit input
- [ ] The inline input offers autocomplete suggestions drawn from the distinct owners already present in the ticket list
- [ ] Submitting the inline input with a non-empty value assigns that owner and updates the display without a page reload
- [ ] Submitting the inline input with an empty value clears the owner and updates the display without a page reload
- [ ] Pressing Escape while editing the owner field cancels the edit and reverts to the previous display
- [ ] After assigning or clearing an owner via the web UI, refreshing the page shows the persisted value

### Out of scope

- User account management or validation that the entered owner matches a known system user
- Permission enforcement (restricting who may change the owner)
- Bulk owner assignment via the web UI
- Notifications or webhooks triggered by owner changes
- Worker view (WorkerView) — owner display there is not changed by this ticket
- Any changes to the CLI assign command or apm-proxy

### Approach

How the implementation will work.

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