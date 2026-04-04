+++
id = "ffaad988"
title = "apm start and apm state: set and clear owner on transitions"
state = "new"
priority = 0
effort = 0
risk = 0
author = "apm"
branch = "ticket/ffaad988-apm-start-and-apm-state-set-and-clear-ow"
created_at = "2026-04-04T06:28:06.049762Z"
updated_at = "2026-04-04T06:28:06.049762Z"
epic = "8db73240"
target_branch = "epic/8db73240-user-mgmt"
+++

## Spec

### Problem

Once an `owner` field exists on tickets, it needs to be set and cleared at the right moments. Today `apm start` sets the `agent` name in the ticket's History section but nothing in frontmatter. When a ticket moves back to `ready` or `groomed` (e.g. after being blocked and unblocked by a supervisor), the previous owner's name lingers — there is no mechanism to release ownership so another user or worker can pick it up. The field must follow the ticket lifecycle: set on claim (`apm start`, `apm state in_design`), preserved through active states (`in_progress`, `in_design`), and cleared when the ticket returns to a pool state (`ready`, `groomed`, `ammend`).

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