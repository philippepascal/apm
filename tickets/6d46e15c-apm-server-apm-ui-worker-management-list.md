+++
id = "6d46e15c"
title = "apm-server + apm-ui: worker management (list, stop, reassign)"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "apm"
agent = "66058"
branch = "ticket/6d46e15c-apm-server-apm-ui-worker-management-list"
created_at = "2026-03-31T06:13:21.657306Z"
updated_at = "2026-03-31T07:29:56.040384Z"
+++

## Spec

### Problem

The worker activity panel shows running workers but provides no controls. Extend GET /api/workers with PID and uptime, add DELETE /api/workers/:pid to stop a worker, and add a reassign action (apm take equivalent) on the ticket detail panel. Full spec context: initial_specs/UIdraft_spec_starter.md Step 15. Requires Step 7a and Step 8.

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
| 2026-03-31T07:29Z | new | in_design | philippepascal |