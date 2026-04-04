+++
id = "717f9b6b"
title = "UI filter error"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "apm-ui"
branch = "ticket/717f9b6b-ui-filter-error"
created_at = "2026-04-04T16:05:39.699619Z"
updated_at = "2026-04-04T16:40:07.217530Z"
+++

## Spec

### Problem

On every mount (including browser refresh), `SupervisorView` fires a `useEffect` that calls `/api/me` and auto-sets `authorFilter` to the current user's username. Because `authorFilter` is local React state it always initialises to `null` on refresh, then the effect overwrites it with the username.

When the detected username does not appear as the `author` field on any ticket — common when the supervisor oversees work authored by agents (`apm`, `apm-ui`, etc.) — the `columns` memo produces zero results and the panel renders the empty state ('No tickets match the current filters') even though tickets exist.

The user's workaround is to manually change the author filter select, which clears the auto-applied value and restores visibility. Desired behaviour: the supervisor panel should default to showing all tickets on load; any author filter the user sets manually should survive a page refresh.

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
| 2026-04-04T16:05Z | — | new | apm-ui |
| 2026-04-04T16:39Z | new | groomed | apm |
| 2026-04-04T16:40Z | groomed | in_design | philippepascal |