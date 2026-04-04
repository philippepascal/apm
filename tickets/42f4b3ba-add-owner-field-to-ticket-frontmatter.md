+++
id = "42f4b3ba"
title = "Add owner field to ticket frontmatter"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "apm"
branch = "ticket/42f4b3ba-add-owner-field-to-ticket-frontmatter"
created_at = "2026-04-04T06:28:01.284791Z"
updated_at = "2026-04-04T06:42:31.380933Z"
epic = "8db73240"
target_branch = "epic/8db73240-user-mgmt"
+++

## Spec

### Problem

The ticket frontmatter has `author` (who created it) and `supervisor` (who reviews it) but no field to track who is currently working on it. The UI has an "agent" filter dropdown that renders but does nothing because there is no corresponding field in the Frontmatter struct or API response. Without an ownership field, there is no way to answer "which tickets is Alice currently responsible for?" — you can only see who created them.

### Acceptance criteria

- [ ] `Frontmatter` has an `agent` field that round-trips through TOML parse/serialize
- [ ] `apm start <id>` sets `agent` on the ticket frontmatter to the running agent's name
- [ ] `apm state <id> in_design` sets `agent` on the ticket frontmatter to the running agent's name
- [ ] `apm take <id>` sets `agent` to the new agent's name (replacing the previous value) and records the correct previous agent name in the history row instead of "unknown"
- [ ] `apm set <id> agent <name>` sets the `agent` field; `apm set <id> agent -` clears it
- [ ] `apm list --agent <name>` filters to tickets whose `agent` field matches (analogous to existing `--author` filter)
- [ ] `GET /api/tickets?agent=<name>` returns only tickets whose `agent` matches
- [ ] `GET /api/tickets` includes `agent` in each ticket object when set (null/absent when not set)
- [ ] The UI agent filter dropdown is populated from the `agent` values in the ticket list and correctly filters the swimlane view

### Out of scope

- Clearing `agent` automatically when a ticket leaves an active state (e.g. transitions to `specd`, `blocked`, or `closed`) — ownership tracking is intentionally sticky
- UI changes to the TicketCard or TicketDetail components (the UI already reads `ticket.agent` and renders it; once the API returns the field the UI will work)
- Back-filling `agent` on existing tickets in git history
- Enforcing that only one agent can own a ticket at a time

### Approach

How the implementation will work.

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-04-04T06:28Z | — | new | apm |
| 2026-04-04T06:35Z | new | groomed | apm |
| 2026-04-04T06:42Z | groomed | in_design | philippepascal |