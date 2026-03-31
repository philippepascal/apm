+++
id = "e1748434"
title = "apm-ui: 3-column resizable/hidable layout shell with Zustand"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "apm"
agent = "64729"
branch = "ticket/e1748434-apm-ui-3-column-resizable-hidable-layout"
created_at = "2026-03-31T06:11:50.266948Z"
updated_at = "2026-03-31T06:20:11.319397Z"
+++

## Spec

### Problem

The workscreen layout (3 resizable/hidable columns: workerview, supervisorview, ticket detail) needs to be established before any data is rendered into it. Zustand store holds selectedTicketId and column visibility flags. No data rendered yet — validate resize, hide, and keyboard focus between columns. Full spec context: initial_specs/UIdraft_spec_starter.md Step 4. Requires Step 3.

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
| 2026-03-31T06:11Z | — | new | apm |
| 2026-03-31T06:20Z | new | in_design | philippepascal |