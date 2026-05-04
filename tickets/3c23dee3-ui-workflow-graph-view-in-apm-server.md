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

#### Backend — new `/api/workflow` endpoint

1. **`apm-server/src/handlers/workflow.rs`** (new file): define two response structs and the handler.

   ```rust
   #[derive(serde::Serialize)]
   pub struct StateNode {
       pub id: String,
       pub label: String,
       pub terminal: bool,
       pub actionable: Vec<String>,
   }

   #[derive(serde::Serialize)]
   pub struct TransitionEdge {
       pub from: String,
       pub to: String,
       pub label: String,
       pub trigger: String,
   }

   #[derive(serde::Serialize)]
   pub struct WorkflowGraphResponse {
       pub states: Vec<StateNode>,
       pub transitions: Vec<TransitionEdge>,
   }

   pub async fn workflow_handler(State(state): State<Arc<AppState>>) -> Json<WorkflowGraphResponse> {
       let Some(root) = state.git_root() else {
           return Json(WorkflowGraphResponse { states: vec![], transitions: vec![] });
       };
       let Ok(cfg) = apm_core::config::Config::load(root) else {
           return Json(WorkflowGraphResponse { states: vec![], transitions: vec![] });
       };
       let states = cfg.workflow.states.iter().map(|s| StateNode {
           id: s.id.clone(),
           label: s.label.clone(),
           terminal: s.terminal,
           actionable: s.actionable.clone(),
       }).collect();
       let transitions = cfg.workflow.states.iter().flat_map(|s| {
           s.transitions.iter().map(move |tr| TransitionEdge {
               from: s.id.clone(),
               to: tr.to.clone(),
               label: if tr.label.is_empty() {
                   format!("→ {}", tr.to)
               } else {
                   tr.label.clone()
               },
               trigger: tr.trigger.clone(),
           })
       }).collect();
       Json(WorkflowGraphResponse { states, transitions })
   }
   ```

2. **`apm-server/src/handlers/mod.rs`**: add `pub mod workflow;`.

3. **`apm-server/src/main.rs`**: add the route to the protected router:
   ```rust
   .route("/api/workflow", get(handlers::workflow::workflow_handler))
   ```

#### Frontend — layout utilities

4. **`apm-ui/src/lib/workflowLayout.ts`** (new file): pure-TS layer assignment and coordinate computation.

   - `assignLayers(states, transitions)` → `Map<id, layer>`:
     - Build in-degree map from transitions.
     - Longest-path: initialise all layers to 0; iterate topologically (Kahn's algorithm), setting `layer[to] = max(layer[to], layer[from] + 1)`.
     - Nodes with no incoming edges start at layer 0.
   - `computePositions(states, transitions, nodeW, nodeH, colGap, rowGap)` → `Map<id, {x, y}>`:
     - Group states by layer (preserving config order within each layer).
     - `x = layer * (nodeW + colGap)`.
     - `y = index_in_layer * (nodeH + rowGap)`, centred vertically per layer.
   - Returns a flat `{x, y, layer}` record per state id.

#### Frontend — components

5. **`apm-ui/src/components/WorkflowGraph.tsx`** (new file):
   - Fetches `/api/workflow` with `useQuery(['workflow'], ...)` (stale for 60 s — rarely changes).
   - Calls `computePositions` with constants `NODE_W=120, NODE_H=40, COL_GAP=80, ROW_GAP=20`.
   - Computes SVG `viewBox` from the bounding box of all positions.
   - Renders `<defs>` with a single `<marker id="arrowhead">` (filled triangle, ~8 px).
   - For each state: `<g>` containing a `<rect>` (rounded, stroke from `getStateColors(id).dot` converted to a border, double stroke or dashed for terminal), and a `<text>` label centred inside.
   - For each transition: `<path>` with a cubic bezier from the right edge of the source rect to the left edge of the target rect (control points offset by `colGap / 2`); self-loops (same layer, adjacent) curve upward. `marker-end="url(#arrowhead)"`. A small `<text>` positioned at the midpoint of the bezier for the label.
   - Empty state: when `states.length === 0`, renders `<p>No workflow configured</p>` instead of the SVG.

   **Colour mapping**: `stateColors.ts` exports `getStateColors(state)` which returns `{ dot: 'bg-<colour>-500', ... }`. The SVG needs actual hex colours, not Tailwind class strings. Add a small helper alongside the component (or in `stateColors.ts`) that maps the Tailwind dot class to an SVG-safe hex string (e.g. `'bg-blue-500' → '#3b82f6'`, covering the six palettes already in use: red, amber, blue, purple, green, gray).

6. **`apm-ui/src/components/WorkflowGraphModal.tsx`** (new file):
   - A full-screen overlay (`fixed inset-0 bg-black/70 z-50 flex items-center justify-center`) with a close button.
   - Renders `<WorkflowGraph />` inside a scrollable white card (`max-w-[90vw] max-h-[90vh] overflow-auto`).
   - Accepts `open: boolean` and `onClose: () => void` props.

7. **`apm-ui/src/store/useLayoutStore.ts`**: add `workflowOpen: boolean` (default `false`) and `setWorkflowOpen: (v: boolean) => void`.

8. **`apm-ui/src/components/supervisor/SupervisorView.tsx`**:
   - Import `WorkflowGraphModal` and `useLayoutStore.workflowOpen / setWorkflowOpen`.
   - Add a `<button>` in the header toolbar (alongside Sync / Clean) labelled "Workflow" with an appropriate icon (e.g. `GitBranch` from `lucide-react`).
   - Render `<WorkflowGraphModal open={workflowOpen} onClose={() => setWorkflowOpen(false)} />` inside the component tree.

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-05-04T17:40Z | — | new | philippepascal |
| 2026-05-04T17:40Z | new | groomed | philippepascal |
| 2026-05-04T17:41Z | groomed | in_design | philippepascal |