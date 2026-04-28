+++
id = "63f5e6d2"
title = "UI: epics filter fixes"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/63f5e6d2-ui-epics-filter-fixes"
created_at = "2026-04-28T19:28:59.918275Z"
updated_at = "2026-04-28T19:42:01.533924Z"
+++

## Spec

### Problem

The epics filter dropdown in `SupervisorView` has two independent bugs.

**No auto-refresh.** The `useQuery` for epics (`queryKey: ['epics']`) at `SupervisorView.tsx:60` has no `refetchInterval`. Every other data query in the UI — tickets, ticket detail, priority queue — polls every 10 seconds. Because the epics query never re-fires on its own, a new epic created outside the browser (via CLI or another session) won't appear in the filter dropdown until the page reloads or a ticket-creating mutation happens to invalidate the `['epics']` cache entry. Supervisors working in long-running sessions routinely miss newly created epics.

**Missing "No epic" option.** The dropdown only allows "All epics" or filtering by a specific epic ID. There is no way to show only tickets where `epic` is absent — a useful view for finding orphaned tickets that have never been assigned to an epic.

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
| 2026-04-28T19:28Z | — | new | philippepascal |
| 2026-04-28T19:33Z | new | groomed | philippepascal |
| 2026-04-28T19:42Z | groomed | in_design | philippepascal |