+++
id = "45e401f9"
title = "UI: in progress shows up in supervisor panel"
state = "in_design"
priority = 0
effort = 1
risk = 2
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/45e401f9-ui-in-progress-shows-up-in-supervisor-pa"
created_at = "2026-04-29T06:57:10.412779Z"
updated_at = "2026-04-29T21:27:15.607295Z"
+++

## Spec

### Problem

The supervisor panel is intended to show only tickets in states where the supervisor must act: `new`, `question`, `specd`, `blocked`, `merge_failed`, and `implemented`. The `in_progress` state has no `actionable = ["supervisor"]` in `workflow.toml` and should never appear there.

A previous ticket fixed `merge_failed` not showing up in the supervisor panel. As part of that fix, a catch-all block was added to `apm-server/src/handlers/tickets.rs` (lines 68–82). This block scans all non-terminal tickets and adds any state not already in `supervisor_states` to the list — regardless of whether the state has `actionable = ["supervisor"]` in the config.

The consequence: when any ticket is in `in_progress`, the catch-all appends `in_progress` to `supervisor_states`, causing the UI to render an unwanted swimlane column for it.

The catch-all is also redundant: `merge_failed` already carries `actionable = ["supervisor"]` in `.apm/workflow.toml`, so the normal config-driven path (lines 52–57) already includes it in `supervisor_states`. The catch-all provides no value for `merge_failed` and actively harms correct behaviour for `in_progress`.

### Acceptance criteria

- [ ] The supervisor panel does not show an `in_progress` swimlane column when one or more tickets exist in `in_progress` state
- [ ] The supervisor panel continues to show `merge_failed` tickets after the fix
- [ ] The supervisor panel continues to show all other expected supervisor-actionable states (`new`, `question`, `specd`, `blocked`, `implemented`) when tickets exist in those states

### Out of scope

- UI component changes (SupervisorView.tsx, Swimlane.tsx)\n- Adding automated tests for supervisor panel state filtering\n- Changes to workflow state definitions in workflow.toml

### Approach

Delete the catch-all block in `apm-server/src/handlers/tickets.rs` (lines 68–82 inclusive). That block is both incorrect — it adds states regardless of `actionable` — and unnecessary — `merge_failed` is already surfaced by the normal config-driven path at lines 52–57 because it has `actionable = ["supervisor"]` in `.apm/workflow.toml`.

After removing the block, inspect line 45: if `supervisor_states` is no longer mutated after that point, remove the `mut` qualifier from its binding to keep the code clean.

No UI changes are required. The supervisor panel (`SupervisorView.tsx`) already derives its columns entirely from `supervisor_states` returned by the server; fixing the server response is sufficient.

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-04-29T06:57Z | — | new | philippepascal |
| 2026-04-29T21:13Z | new | groomed | philippepascal |
| 2026-04-29T21:24Z | groomed | in_design | philippepascal |