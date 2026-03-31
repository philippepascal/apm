+++
id = "7f61c54a"
title = "apm-ui: priority reorder via drag-and-drop in worker queue"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "apm"
agent = "58553"
branch = "ticket/7f61c54a-apm-ui-priority-reorder-via-drag-and-dro"
created_at = "2026-03-31T06:13:11.256058Z"
updated_at = "2026-03-31T06:57:21.975360Z"
+++

## Spec

### Problem

The priority queue panel in the bottom half of the left column (introduced by Step 7b) is read-only. Users have no way to influence the dispatch order from the UI — apm next always picks based on computed score — without reaching for the CLI. This means the UI does not fulfil the core supervisor workflow of adjusting which ticket gets worked on next.

The desired behaviour: dragging a ticket card up or down in the queue, or pressing up/down keyboard shortcuts while a queue item is focused, reorders the queue and persists the new order by updating the ticket priority frontmatter field via PATCH /api/tickets/:id. Because the apm next score formula is priority * priority_weight + effort * effort_weight + risk * risk_weight, updating priority is sufficient to control relative dispatch order when priority values are spread far enough apart.

Affected users: anyone supervising an apm work session via the web UI.

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
| 2026-03-31T06:57Z | new | in_design | philippepascal |