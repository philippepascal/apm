+++
id = "867df156"
title = "Add epic filter to work engine server endpoint"
state = "in_design"
priority = 4
effort = 0
risk = 0
author = "claude-0401-2145-a8f3"
agent = "93815"
branch = "ticket/867df156-add-epic-filter-to-work-engine-server-en"
created_at = "2026-04-01T21:56:02.797958Z"
updated_at = "2026-04-02T00:52:44.574586Z"
+++

## Spec

### Problem

The work engine server (POST /api/work/start) accepts no request body and always starts the engine in open mode -- dispatching any actionable ticket regardless of epic membership. There is no way for the UI (or any API caller) to start the engine in epic-exclusive mode, where only tickets belonging to a specific epic are dispatched.

Correspondingly, GET /api/work/status returns only a status field and has no way to communicate whether an active engine is running in epic-exclusive mode, which makes the UI unable to reflect that constraint to the user.

The design for epic-scoped scheduling is specified in docs/epics.md (section: Work engine -- epic filter). This ticket implements that slice: the server-side plumbing connecting an optional "epic" field in the start request through to run_engine_loop and spawn_next_worker, and the corresponding status reporting.

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
| 2026-04-01T21:56Z | — | new | claude-0401-2145-a8f3 |
| 2026-04-01T22:00Z | new | groomed | claude-0401-2145-a8f3 |
| 2026-04-02T00:52Z | groomed | in_design | philippepascal |