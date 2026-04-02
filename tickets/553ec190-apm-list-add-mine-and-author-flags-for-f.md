+++
id = "553ec190"
title = "apm list: add --mine and --author flags for filtering by collaborator"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "apm"
agent = "75161"
branch = "ticket/553ec190-apm-list-add-mine-and-author-flags-for-f"
created_at = "2026-04-02T20:54:04.874772Z"
updated_at = "2026-04-02T23:39:38.498948Z"
epic = "8db73240"
target_branch = "epic/8db73240-user-mgmt"
depends_on = ["610be42e"]
+++

## Spec

### Problem

There is no way to filter `apm list` output by ticket author. A developer working on a shared project has to scan all tickets to find their own. `apm list --mine` and `apm list --author <username>` are the intended daily-driver filters. See `initial_specs/DESIGN-users.md` point 7.

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
| 2026-04-02T23:39Z | groomed | in_design | philippepascal |