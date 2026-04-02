+++
id = "610be42e"
title = "apm-core: write author from identity on ticket creation, remove agent field"
state = "new"
priority = 0
effort = 0
risk = 0
author = "apm"
branch = "ticket/610be42e-apm-core-write-author-from-identity-on-t"
created_at = "2026-04-02T20:53:55.085303Z"
updated_at = "2026-04-02T20:53:55.085303Z"
epic = "8db73240"
target_branch = "epic/8db73240-user-mgmt"
depends_on = ["4cec7a17"]
+++

## Spec

### Problem

New tickets write `author` from the agent name rather than resolving a real collaborator identity. The identity resolution function (git host plugin → local.toml → "unassigned") does not yet exist in apm-core, so `apm new` cannot populate `author` correctly. See `initial_specs/DESIGN-users.md` points 1 and 3.

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