+++
id = "3c23dee3"
title = "UI: workflow graph view in apm-server"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/3c23dee3-ui-workflow-graph-view-in-apm-server"
created_at = "2026-05-04T17:40:27.172621Z"
updated_at = "2026-05-04T17:41:10.625488Z"
+++

## Spec

### Problem

APM's ticket lifecycle is a user-configurable directed graph of states and transitions defined in `.apm/workflow.toml`. Currently, the only way to inspect this graph is to read the raw TOML file. The `apm-server` web UI offers no visual representation, which makes it hard to understand the overall lifecycle at a glance and to onboard new collaborators without pointing them at config files.

The desired behaviour is a diagram that shows every state as a labelled node and every permitted transition as a directed, labelled arrow — rendered inside the existing `apm-server` React UI without requiring a page reload or leaving the board view. Because the workflow is user-defined, the graph must be derived from the live server configuration rather than hard-coded.

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
| 2026-05-04T17:40Z | — | new | philippepascal |
| 2026-05-04T17:40Z | new | groomed | philippepascal |
| 2026-05-04T17:41Z | groomed | in_design | philippepascal |