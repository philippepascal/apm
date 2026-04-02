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

The work engine server endpoint (`POST /api/work/start`) has no way to start the engine in epic-exclusive mode, and the status endpoint does not report whether an epic filter is active. This means the UI cannot drive epic-scoped engine sessions.

The full design is in `docs/epics.md` (§ apm-server changes — Work engine — epic filter). `POST /api/work/start` gains an optional `"epic"` field; when set, `run_engine_loop` filters candidates to `frontmatter.epic == id` before the priority sort. `GET /api/work/status` includes `"epic": "<id>"` when exclusive mode is active, `null` otherwise.

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