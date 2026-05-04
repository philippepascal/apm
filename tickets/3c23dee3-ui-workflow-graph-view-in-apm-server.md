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

- [ ] `GET /api/workflow` returns a JSON object with a `states` array (each entry: `id`, `label`, `terminal`, `actionable`) and a `transitions` array (each entry: `from`, `to`, `label`, `trigger`) reflecting the project's live `WorkflowConfig`
- [ ] `GET /api/workflow` returns `{"states":[],"transitions":[]}` without error when the server is running in `InMemory` mode (no git root / no config file)
- [ ] A "Workflow" button appears in the `SupervisorView` header alongside the existing Sync and Clean buttons
- [ ] Clicking "Workflow" opens a modal that displays the workflow graph
- [ ] The graph renders every state returned by `/api/workflow` as a labelled node
- [ ] Terminal states are visually distinguished from non-terminal states (e.g. different border or opacity)
- [ ] Node fill or border colour matches the colour already used for that state in `stateColors.ts` (the `dot` palette entry)
- [ ] The graph renders every transition as a directed arrow from source node to target node
- [ ] Each transition arrow carries a label (the transition's `label` field, falling back to `"→ <to>"` when blank)
- [ ] Nodes are positioned using a layer-based layout computed at render time from the graph topology; no x/y coordinates are hard-coded in the component
- [ ] The graph is rendered as plain SVG — no new npm graph-layout or rendering library is added to `package.json`
- [ ] When `/api/workflow` returns an empty `states` array, the modal shows a "No workflow configured" message instead of a blank SVG

### Out of scope

- Interactive editing of the workflow graph (adding, removing, or relabelling states/transitions via the UI)
- Ticket-count badges or live ticket data overlaid on state nodes
- URL-based routing to the graph view (no React Router is in the stack)
- Pan, zoom, or drag interaction on the SVG canvas
- Export of the graph as an image or as TOML
- Displaying transition `completion`, `profile`, `on_failure`, or other advanced fields in the graph

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