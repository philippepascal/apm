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
| 2026-04-06T20:57Z | — | new | philippepascal |
| 2026-04-06T21:22Z | new | groomed | apm |
| 2026-04-06T23:13Z | groomed | in_design | philippepascal |
| 2026-04-06T23:21Z | in_design | groomed | apm |
| 2026-04-06T23:22Z | groomed | in_design | philippepascal |