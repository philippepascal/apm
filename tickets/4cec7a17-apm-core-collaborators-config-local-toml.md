+++
id = "4cec7a17"
title = "apm-core: collaborators config, local.toml, and identity resolution"
state = "groomed"
priority = 0
effort = 0
risk = 0
author = "apm"
branch = "ticket/4cec7a17-apm-core-collaborators-config-local-toml"
created_at = "2026-04-02T20:53:47.546444Z"
updated_at = "2026-04-02T23:21:28.551594Z"
epic = "8db73240"
target_branch = "epic/8db73240-user-mgmt"
+++

## Spec

### Problem

There is no concept of collaborator identity in apm-core. The `author` field is currently set to agent names, and there is no local identity file or collaborators list. Resolving who created a ticket requires a real username, not an ephemeral agent string. See `initial_specs/DESIGN-users.md` points 1–3.

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
| 2026-04-02T23:21Z | new | groomed | apm |
