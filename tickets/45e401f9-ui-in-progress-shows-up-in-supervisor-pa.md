+++
id = "45e401f9"
title = "UI: in progress shows up in supervisor panel"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/45e401f9-ui-in-progress-shows-up-in-supervisor-pa"
created_at = "2026-04-29T06:57:10.412779Z"
updated_at = "2026-04-29T21:24:16.423560Z"
+++

## Spec

### Problem

The supervisor panel is intended to show only tickets in states where the supervisor must act: `new`, `question`, `specd`, `blocked`, `merge_failed`, and `implemented`. The `in_progress` state has no `actionable = ["supervisor"]` in `workflow.toml` and should never appear there.

A previous ticket fixed `merge_failed` not showing up in the supervisor panel. As part of that fix, a catch-all block was added to `apm-server/src/handlers/tickets.rs` (lines 68–82). This block scans all non-terminal tickets and adds any state not already in `supervisor_states` to the list — regardless of whether the state has `actionable = ["supervisor"]` in the config.

The consequence: when any ticket is in `in_progress`, the catch-all appends `in_progress` to `supervisor_states`, causing the UI to render an unwanted swimlane column for it.

The catch-all is also redundant: `merge_failed` already carries `actionable = ["supervisor"]` in `.apm/workflow.toml`, so the normal config-driven path (lines 52–57) already includes it in `supervisor_states`. The catch-all provides no value for `merge_failed` and actively harms correct behaviour for `in_progress`.

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
| 2026-04-29T06:57Z | — | new | philippepascal |
| 2026-04-29T21:13Z | new | groomed | philippepascal |
| 2026-04-29T21:24Z | groomed | in_design | philippepascal |