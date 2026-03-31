+++
id = "7f61c54a"
title = "apm-ui: priority reorder via drag-and-drop in worker queue"
state = "new"
priority = 0
effort = 0
risk = 0
author = "apm"
branch = "ticket/7f61c54a-apm-ui-priority-reorder-via-drag-and-dro"
created_at = "2026-03-31T06:13:11.256058Z"
updated_at = "2026-03-31T06:13:11.256058Z"
+++

## Spec

### Problem

The priority queue in the left column is currently read-only. Users need to reorder tickets to influence what apm next dispatches next. Add drag-and-drop (and up/down keyboard shortcuts) that call PATCH /api/tickets/:id {"priority":N} to persist the new order. Full spec context: initial_specs/UIdraft_spec_starter.md Step 11. Requires Step 7b.

### Acceptance criteria


### Out of scope

Explicit list of what this ticket does not cover.

### Approach

How the implementation will work.

### Open questions



### Amendment requests



## History

| When | From | To | By |
|------|------|----|----|
| 2026-03-31T06:13Z | — | new | apm |
