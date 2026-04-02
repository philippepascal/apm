+++
id = "90ebf40b"
title = "apm-server: expose author field in ticket API responses"
state = "groomed"
priority = 0
effort = 0
risk = 0
author = "apm"
branch = "ticket/90ebf40b-apm-server-expose-author-field-in-ticket"
created_at = "2026-04-02T20:54:08.576527Z"
updated_at = "2026-04-02T23:22:42.287398Z"
epic = "8db73240"
target_branch = "epic/8db73240-user-mgmt"
depends_on = ["610be42e"]
+++

## Spec

### Problem

The server's ticket API responses do not include the `author` field. The UI cannot implement author filtering or display ticket ownership without it. See `initial_specs/DESIGN-users.md` points 1 and 8.

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
| 2026-04-02T20:54Z | — | new | apm |
| 2026-04-02T23:22Z | new | groomed | apm |
