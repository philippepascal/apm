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

- [ ] POST /api/work/start with an empty body starts the engine and returns a status response (no regression)
- [ ] POST /api/work/start with body {"epic": "ab12cd34"} starts the engine without error
- [ ] When the engine is started with {"epic": "ab12cd34"}, spawn_next_worker only considers tickets whose frontmatter.epic == "ab12cd34"
- [ ] When the engine is started without an epic field, spawn_next_worker considers all actionable tickets (open mode, no regression)
- [ ] GET /api/work/status returns {"status": "idle", "epic": "ab12cd34"} when the engine was started with that epic filter
- [ ] GET /api/work/status returns {"status": "idle", "epic": null} when the engine was started without an epic filter
- [ ] GET /api/work/status returns {"status": "stopped"} (no epic key) when no engine is running

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