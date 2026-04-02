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

The engine controls panel in the UI has no way to start the engine in epic-exclusive mode, and when exclusive mode is active there is no visual indicator of which epic is running. Without this, the UI cannot drive focused epic sprints.

Currently `WorkEngineControls.tsx` exposes a plain Start/Stop toggle with no parameters. The desired behaviour is:

1. Before starting: show an optional **Epic** selector dropdown (populated from `GET /api/epics`) so the user can choose to restrict the engine to one epic.
2. While running in exclusive mode: display a small `epic: <slug>` label that links to the epic filter on the supervisor board.

This requires extending the server's work engine API to accept and remember an optional epic filter, implementing a minimal `GET /api/epics` route, and adding the `epic` optional field to `Frontmatter` so the engine loop can filter on it.

### Acceptance criteria

- [ ] `GET /api/epics` returns a JSON array of epic objects with at least `id`, `title`, and `branch` fields, derived from `epic/*` remote git branches
- [ ] `GET /api/epics` returns an empty array when no `epic/*` branches exist
- [ ] `POST /api/work/start` with body `{"epic": "ab12cd34"}` starts the engine in exclusive mode
- [ ] `POST /api/work/start` with no body (or body without `epic`) starts the engine in open mode, identical to current behaviour
- [ ] `GET /api/work/status` includes `"epic": "ab12cd34"` when the engine is running in exclusive mode
- [ ] `GET /api/work/status` includes `"epic": null` when the engine is running in open mode
- [ ] The engine controls panel shows an Epic dropdown when the engine is stopped
- [ ] The Epic dropdown is populated with epics from `GET /api/epics`; a blank/"All" option is present as the default (open mode)
- [ ] Clicking Start with an epic selected sends `{"epic": "<id>"}` in the start request body
- [ ] Clicking Start with no epic selected sends no epic in the start request body
- [ ] When the engine is running in exclusive mode, a label `epic: <slug>` is shown next to the status badge
- [ ] The `epic: <slug>` label is not shown when the engine is running in open mode
- [ ] When the engine is stopped or idle the Epic dropdown is visible; when running it is hidden (replaced by the label if applicable)

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