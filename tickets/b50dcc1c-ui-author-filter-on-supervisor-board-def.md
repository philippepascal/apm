+++
id = "b50dcc1c"
title = "UI: author filter on supervisor board, default to current user from /api/me"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "apm"
agent = "philippepascal"
branch = "ticket/b50dcc1c-ui-author-filter-on-supervisor-board-def"
created_at = "2026-04-02T20:54:34.590380Z"
updated_at = "2026-04-03T00:10:33.348507Z"
epic = "8db73240"
target_branch = "epic/8db73240-user-mgmt"
depends_on = ["90ebf40b", "e2e3d958", "70d58b2d"]
+++

## Spec

### Problem

The supervisor board shows all tickets from all authors with no filtering. In a multi-collaborator project, or when automated agents have created tickets (`author = "apm"`), the board is noisy. There is no way to default to the current user's tickets or filter by author. See `initial_specs/DESIGN-users.md` point 8.

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
| 2026-04-02T23:23Z | new | groomed | apm |
| 2026-04-03T00:10Z | groomed | in_design | philippepascal |
