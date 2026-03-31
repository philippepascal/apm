+++
id = "7777cf5c"
title = "apm-ui: priority queue panel (bottom of left column, apm next ordering)"
state = "in_design"
priority = 0
effort = 3
risk = 2
author = "apm"
agent = "philippepascal"
branch = "ticket/7777cf5c-apm-ui-priority-queue-panel-bottom-of-le"
created_at = "2026-03-31T06:12:28.610477Z"
updated_at = "2026-03-31T19:07:38.204273Z"
+++

## Spec

### Problem

The bottom half of the `WorkerView` left column is a stub placeholder labelled "Queue" (put there by Step 7a, ticket 651f8a63). Supervisors currently have no browser-visible way to see which tickets are queued for dispatch or in what order the work engine will pick them up. They must leave the browser and run `apm next` from the CLI.

This ticket fills the placeholder with a `PriorityQueuePanel` showing all agent-actionable tickets ranked by the same scoring formula `apm next` uses: score = priority x priority_weight + effort x effort_weight + risk x risk_weight (weights from [workflow.prioritization] in apm.toml). The panel is read-only at this stage; drag-and-drop reordering is covered by ticket 7f61c54a (Step 11).

Two changes are required: (1) a GET /api/queue endpoint in apm-server returning all agent-actionable tickets sorted by descending score; (2) a PriorityQueuePanel React component wired into WorkerView.tsx replacing the stub placeholder.

### Acceptance criteria

- [ ] `GET /api/queue` returns HTTP 200 with `Content-Type: application/json`
- [ ] The response is a JSON array sorted by descending score using the formula `priority * priority_weight + effort * effort_weight + risk * risk_weight` with weights from `[workflow.prioritization]` in `apm.toml`
- [ ] Each element in the response contains: `rank` (1-based integer), `id`, `title`, `state`, `priority`, `effort`, `risk`, `score`
- [ ] Only tickets whose state is in `config.actionable_states_for("agent")` appear in the response
- [ ] `GET /api/queue` returns an empty JSON array when no agent-actionable tickets exist
- [ ] The handler offloads all blocking git work via `spawn_blocking` and does not block the tokio runtime
- [ ] `PriorityQueuePanel` renders a row for each ticket in the response, showing rank, ID, title, state badge, effort, risk, and score
- [ ] When the response array is empty, `PriorityQueuePanel` shows a centred "No tickets in queue." message
- [ ] While the initial fetch is in-flight, `PriorityQueuePanel` shows loading skeleton rows
- [ ] If the fetch fails, `PriorityQueuePanel` shows an inline error message
- [ ] `PriorityQueuePanel` automatically refetches `GET /api/queue` every 10 seconds via TanStack Query `refetchInterval`
- [ ] Clicking a queue row sets `selectedTicketId` in the Zustand store (the same global selection used by the swimlanes)
- [ ] The row for the currently selected ticket is visually highlighted
- [ ] `PriorityQueuePanel` is rendered in the bottom half of `WorkerView.tsx`, replacing the placeholder stub
- [ ] `npm run build` in `apm-ui/` exits 0 with no TypeScript errors
- [ ] `cargo test --workspace` passes

### Out of scope

- Drag-and-drop or keyboard reordering of the queue — covered by ticket 7f61c54a (Step 11)
- The top half of the left column (worker activity panel) — covered by ticket 651f8a63 (Step 7a)
- SSE/push-based live updates — polling every 10 seconds is sufficient at this stage
- Showing tickets in non-agent-actionable states (e.g. closed, specd, in_design)
- Keyboard navigation within the queue panel (arrow key focus, row-to-row) — deferred
- `DELETE /api/workers/:pid` or any worker control — Step 15 (ticket 6d46e15c)

### Approach

**Prerequisites:** Step 7a (ticket 651f8a63, worker activity panel) must be `implemented` before this ticket moves to `ready`. The `WorkerView.tsx` component and the bottom-half "Queue" stub it introduces are the integration points for this ticket.

---

### Open questions



### Amendment requests

- [ ] Add `config.actionable_states_for(actor: &str) -> Vec<String>` to apm-core Config (scan `[[workflow.states]]` for entries whose `actionable` array contains the given actor string). Both the queue handler and the dry-run handler depend on this method — it must be defined in apm-core before either handler is implemented. Include this step at the top of the Approach.

## History

| When | From | To | By |
|------|------|----|----|
| 2026-03-31T06:12Z | — | new | apm |
| 2026-03-31T06:38Z | new | in_design | philippepascal |
| 2026-03-31T06:42Z | in_design | specd | claude-0331-0638-c698 |
| 2026-03-31T18:15Z | specd | ammend | claude-0331-1200-a7b9 |
| 2026-03-31T19:07Z | ammend | in_design | philippepascal |
