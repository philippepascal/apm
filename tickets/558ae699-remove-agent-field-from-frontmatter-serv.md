+++
id = "558ae699"
title = "Remove agent field from frontmatter, server API, and apm list/show output"
state = "groomed"
priority = 0
effort = 0
risk = 0
author = "apm"
branch = "ticket/558ae699-remove-agent-field-from-frontmatter-serv"
created_at = "2026-04-02T20:53:58.923882Z"
updated_at = "2026-04-02T23:22:33.350042Z"
epic = "8db73240"
target_branch = "epic/8db73240-user-mgmt"
depends_on = ["610be42e"]
+++

## Spec

### Problem

The `agent` field appears in ticket frontmatter (deserialized on read), server API responses, and `apm list`/`apm show` output. Workers are spawned once per ticket and the agent name has no durable value after implementation. The field adds noise and ties frontmatter to a specific agent naming convention. See `initial_specs/DESIGN-users.md` point 2.

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
| 2026-04-02T20:53Z | — | new | apm |
| 2026-04-02T23:22Z | new | groomed | apm |
