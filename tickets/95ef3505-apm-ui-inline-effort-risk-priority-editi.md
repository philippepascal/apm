+++
id = "95ef3505"
title = "apm-ui: inline effort/risk/priority editing in ticket detail"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "apm"
agent = "80361"
branch = "ticket/95ef3505-apm-ui-inline-effort-risk-priority-editi"
created_at = "2026-03-31T06:13:16.584261Z"
updated_at = "2026-03-31T07:14:43.852798Z"
+++

## Spec

### Problem

effort, risk, and priority fields in the ticket detail panel are read-only. Users need click-to-edit inline controls for these fields, backed by PATCH /api/tickets/:id, without opening the full markdown editor. Full spec context: initial_specs/UIdraft_spec_starter.md Step 13. Requires Step 9.

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
| 2026-03-31T07:14Z | new | in_design | philippepascal |