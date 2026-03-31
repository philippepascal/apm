+++
id = "56499b61"
title = "apm-server + apm-ui: apm work engine start/stop controls"
state = "new"
priority = 0
effort = 0
risk = 0
author = "apm"
branch = "ticket/56499b61-apm-server-apm-ui-apm-work-engine-start-"
created_at = "2026-03-31T06:13:12.529756Z"
updated_at = "2026-03-31T06:13:12.529756Z"
+++

## Spec

### Problem

There is no way to start or stop the apm work daemon from the UI. Add POST /api/work/start and POST /api/work/stop endpoints. The top of the workerview panel shows a start/stop button with a status indicator (running / stopped / idle) and a keyboard shortcut. Full spec context: initial_specs/UIdraft_spec_starter.md Step 12. Requires Step 7a.

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
