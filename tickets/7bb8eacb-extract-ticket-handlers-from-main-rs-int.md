+++
id = "7bb8eacb"
title = "Extract ticket handlers from main.rs into handlers/tickets.rs"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/7bb8eacb-extract-ticket-handlers-from-main-rs-int"
created_at = "2026-04-12T09:03:09.497239Z"
updated_at = "2026-04-12T09:45:34.376132Z"
epic = "1e706443"
target_branch = "epic/1e706443-refactor-apm-server-code-organization"
depends_on = ["a6bc1326", "2973f8d1"]
+++

## Spec

### Problem

`apm-server/src/main.rs` contains ~500 lines of ticket CRUD handler functions that should be in their own module. These include:

- `list_tickets()` (~90 lines) — filtering, dependency computation, response building
- `get_ticket()` (~45 lines) — load ticket, compute deps and transitions
- `create_ticket()` — creates ticket via apm_core
- `patch_ticket()` — updates ticket fields
- `batch_update_tickets()` — bulk state/field updates
- `get_ticket_spec()`, `update_ticket_spec()` — spec section CRUD
- Various helper functions for ticket serialization

These handlers depend on the DTOs extracted by the prerequisite ticket (models.rs) and the business logic moved to apm_core by the other prerequisite. Extracting them into `handlers/tickets.rs` will reduce main.rs by ~500 lines and group all ticket-related HTTP logic in one place.

After extraction, main.rs should only reference the handler functions in its route definitions.

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
| 2026-04-12T09:03Z | — | new | philippepascal |
| 2026-04-12T09:09Z | new | groomed | apm |
| 2026-04-12T09:45Z | groomed | in_design | philippepascal |
