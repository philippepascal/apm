+++
id = "fabfef3d"
title = "apm-ui: apm work dry-run preview before engine start"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "apm"
agent = "17766"
branch = "ticket/fabfef3d-apm-ui-apm-work-dry-run-preview-before-e"
created_at = "2026-03-31T06:13:13.767038Z"
updated_at = "2026-03-31T07:05:53.084511Z"
+++

## Spec

### Problem

When the apm work engine is stopped, users have no way to preview which tickets would be dispatched if they started it. They must either start the engine and watch what it does, or run `apm work --dry-run` on the command line. Neither gives visibility directly from the UI before committing to a start.

The CLI already implements the dry-run logic in `apm/src/cmd/work.rs:run_dry()`: load all tickets from git, filter to actionable+startable states, sort by score (priority_weight × priority + effort_weight × effort + risk_weight × risk), and take up to `max_concurrent`. This ticket exposes that same logic through a `GET /api/work/dry-run` HTTP endpoint and renders the result in a preview panel in the workerview column, visible whenever the engine is stopped.

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
| 2026-03-31T07:05Z | new | in_design | philippepascal |