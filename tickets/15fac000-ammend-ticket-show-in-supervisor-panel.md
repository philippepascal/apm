+++
id = "15fac000"
title = "ammend ticket show in supervisor panel"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
branch = "ticket/15fac000-ammend-ticket-show-in-supervisor-panel"
created_at = "2026-04-07T18:17:11.036755Z"
updated_at = "2026-04-07T18:22:06.257873Z"
+++

## Spec

### Problem

Tickets in the `ammend` state currently appear in the supervisor panel in the APM UI. This is wrong — `ammend` is a state where a spec-writer agent must act (it has `actionable = ["agent"]` in `workflow.toml`), so it belongs in the agent work queue, not in the supervisor's attention queue.

The root cause is that `apm-ui/src/lib/supervisorUtils.ts` hardcodes a `SUPERVISOR_STATES` array that explicitly names `'ammend'`. The supervisor panel uses this array to decide what to render, with no reference to `workflow.toml`. Any future workflow changes (new states, renamed states, changed `actionable` actors) require manual UI edits or risk the same bug recurring.

The desired behaviour is that the supervisor panel derives its visible-state list from the `actionable` property already present in `workflow.toml`: it should show states where the `supervisor` actor is listed as actionable. The `ammend` state (`actionable = ["agent"]`) is then excluded automatically, with no string matching on state names in the UI.

### Acceptance criteria

- [ ] make sure ammend is not hardcoded in the logic. it should only use existing worflow.toml properties

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
| 2026-04-07T18:17Z | — | new | philippepascal |
| 2026-04-07T18:17Z | new | groomed | apm |
| 2026-04-07T18:22Z | groomed | in_design | philippepascal |