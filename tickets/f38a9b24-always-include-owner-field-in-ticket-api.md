+++
id = "f38a9b24"
title = "Always include owner field in ticket API responses"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/f38a9b24-always-include-owner-field-in-ticket-api"
created_at = "2026-04-06T20:57:23.971981Z"
updated_at = "2026-04-06T21:42:42.121497Z"
+++

## Spec

### Problem

The GET /api/tickets and GET /api/tickets/:id endpoints omit the owner field entirely when it is None. This forces every client to distinguish between 'field absent' and 'field is null', which is error-prone and inconsistent with how other optional fields (like author) are handled. The owner field should always be present in API responses — set to the username string when assigned, or null when unassigned. This applies to both the list and detail endpoints.

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
| 2026-04-06T21:42Z | groomed | in_design | philippepascal |
