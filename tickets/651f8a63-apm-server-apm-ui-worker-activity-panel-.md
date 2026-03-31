+++
id = "651f8a63"
title = "apm-server + apm-ui: worker activity panel (running workers, top of left column)"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "apm"
agent = "philippepascal"
branch = "ticket/651f8a63-apm-server-apm-ui-worker-activity-panel-"
created_at = "2026-03-31T06:12:27.354130Z"
updated_at = "2026-03-31T06:33:51.720971Z"
+++

## Spec

### Problem

The top half of the left column shows running worker processes and which ticket each holds. Add GET /api/workers listing PID, agent name, ticket id, and state. The panel polls on a short interval or uses SSE. Full spec context: initial_specs/UIdraft_spec_starter.md Step 7. Requires Step 6.

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
| 2026-03-31T06:33Z | new | in_design | philippepascal |
