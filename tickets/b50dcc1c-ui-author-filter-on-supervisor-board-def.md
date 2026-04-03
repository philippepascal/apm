+++
id = "b50dcc1c"
title = "UI: author filter on supervisor board, default to current user from /api/me"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "apm"
agent = "43395"
branch = "ticket/b50dcc1c-ui-author-filter-on-supervisor-board-def"
created_at = "2026-04-02T20:54:34.590380Z"
updated_at = "2026-04-03T00:10:33.348507Z"
epic = "8db73240"
target_branch = "epic/8db73240-user-mgmt"
depends_on = ["90ebf40b", "e2e3d958", "70d58b2d"]
+++

## Spec

### Problem

The supervisor board shows all tickets from all authors with no filtering. In a multi-collaborator project, or when automated agents have created tickets (`author = "apm"`), the board is noisy. A developer cannot focus on their own work without manually scanning through unrelated tickets.

DESIGN-users.md point 8 specifies the desired behaviour: on load, the board defaults to showing only tickets where `author` matches the current user (fetched from `GET /api/me`), with an explicit control to reveal all authors. This default is also useful for solo developers: it filters out the noise of agent-authored side notes and automated tickets.

The `author` field is already present in ticket frontmatter and will be guaranteed present in API responses by ticket #90ebf40b. The `GET /api/me` endpoint is established by #90ebf40b (localhost case) and extended by #e2e3d958 (session-authenticated case). This ticket is purely a UI change: add the author filter control to the supervisor board and wire the default to `/api/me`.

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