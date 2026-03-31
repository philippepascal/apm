+++
id = "56499b61"
title = "apm-server + apm-ui: apm work engine start/stop controls"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "apm"
agent = "45049"
branch = "ticket/56499b61-apm-server-apm-ui-apm-work-engine-start-"
created_at = "2026-03-31T06:13:12.529756Z"
updated_at = "2026-03-31T07:02:07.528911Z"
+++

## Spec

### Problem

The apm work engine — which dispatches Claude worker agents to actionable tickets — can only be started or stopped from the command line. There is no way to control it from the UI.

This ticket adds three server endpoints (`GET /api/work/status`, `POST /api/work/start`, `POST /api/work/stop`) and a control widget at the top of the workerview left column. The widget shows the current engine state (running / idle / stopped) and a button to toggle it, with a keyboard shortcut.

The work engine runs as a child process of the axum server, equivalent to `apm work --daemon`. The server tracks the child process handle in shared state. Start spawns it; stop sends SIGTERM. Status is derived by checking whether the child is alive and whether any worker PID files exist.

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
| 2026-03-31T07:02Z | new | in_design | philippepascal |