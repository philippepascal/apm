+++
id = "2b7c4c97"
title = "apm-server: expose owner in ticket API and add owner query param"
state = "new"
priority = 0
effort = 0
risk = 0
author = "apm"
branch = "ticket/2b7c4c97-apm-server-expose-owner-in-ticket-api-an"
created_at = "2026-04-04T06:28:16.243562Z"
updated_at = "2026-04-04T06:28:16.243562Z"
epic = "8db73240"
target_branch = "epic/8db73240-user-mgmt"
+++

## Spec

### Problem

The `/api/tickets` endpoint returns `author` but not `owner` in its response. The `ListTicketsQuery` struct supports `?author=` but not `?owner=`. The UI cannot display or filter by ticket ownership until the server exposes the field and supports the query parameter.

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
| 2026-04-04T06:28Z | — | new | apm |