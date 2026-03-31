+++
id = "7777cf5c"
title = "apm-ui: priority queue panel (bottom of left column, apm next ordering)"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "apm"
agent = "7369"
branch = "ticket/7777cf5c-apm-ui-priority-queue-panel-bottom-of-le"
created_at = "2026-03-31T06:12:28.610477Z"
updated_at = "2026-03-31T06:38:21.959118Z"
+++

## Spec

### Problem

The bottom half of the `WorkerView` left column is a stub placeholder labelled "Queue" (put there by Step 7a, ticket 651f8a63). Supervisors currently have no browser-visible way to see which tickets are queued for dispatch or in what order the work engine will pick them up. They must leave the browser and run `apm next` from the CLI.

This ticket fills the placeholder with a `PriorityQueuePanel` showing all agent-actionable tickets ranked by the same scoring formula `apm next` uses: score = priority x priority_weight + effort x effort_weight + risk x risk_weight (weights from [workflow.prioritization] in apm.toml). The panel is read-only at this stage; drag-and-drop reordering is covered by ticket 7f61c54a (Step 11).

Two changes are required: (1) a GET /api/queue endpoint in apm-server returning all agent-actionable tickets sorted by descending score; (2) a PriorityQueuePanel React component wired into WorkerView.tsx replacing the stub placeholder.

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
| 2026-03-31T06:12Z | — | new | apm |
| 2026-03-31T06:38Z | new | in_design | philippepascal |