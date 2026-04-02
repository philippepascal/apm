+++
id = "ea172f4a"
title = "UI: add epic selector to engine controls"
state = "in_design"
priority = 2
effort = 0
risk = 0
author = "claude-0401-2145-a8f3"
agent = "87245"
branch = "ticket/ea172f4a-ui-add-epic-selector-to-engine-controls"
created_at = "2026-04-01T21:56:28.916880Z"
updated_at = "2026-04-02T00:58:02.594592Z"
+++

## Spec

### Problem

The engine controls panel has no way to start the engine in epic-exclusive mode, and when exclusive mode is active there is no visual indicator of which epic is running. Without this, the UI cannot drive focused epic sprints.

The full design is in `docs/epics.md` (§ apm-ui changes — Engine controls). Add an optional **Epic** selector dropdown (populated from `GET /api/epics`) before starting the engine. When exclusive mode is active, show a small label: `epic: <slug>` that links to the epic filter on the supervisor board.

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
| 2026-04-01T22:01Z | new | groomed | claude-0401-2145-a8f3 |
| 2026-04-02T00:58Z | groomed | in_design | philippepascal |