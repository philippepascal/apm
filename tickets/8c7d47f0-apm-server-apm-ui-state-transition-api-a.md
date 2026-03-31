+++
id = "8c7d47f0"
title = "apm-server + apm-ui: state transition API and buttons"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "apm"
agent = "philippepascal"
branch = "ticket/8c7d47f0-apm-server-apm-ui-state-transition-api-a"
created_at = "2026-03-31T06:12:47.638355Z"
updated_at = "2026-03-31T06:42:17.687543Z"
+++

## Spec

### Problem

There is no way to transition ticket state from the UI. Add POST /api/tickets/:id/transition {"to":"<state>"} backed by the apm-core state machine. The ticket detail panel gains buttons for all valid transitions from the current state, including close and keep-at-current-state, matching CLI behaviour. Full spec context: initial_specs/UIdraft_spec_starter.md Step 8. Requires Step 6.

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
| 2026-03-31T06:12Z | — | new | apm |
| 2026-03-31T06:42Z | new | in_design | philippepascal |
