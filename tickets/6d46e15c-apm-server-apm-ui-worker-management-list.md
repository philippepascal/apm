+++
id = "6d46e15c"
title = "apm-server + apm-ui: worker management (list, stop, reassign)"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "apm"
agent = "66058"
branch = "ticket/6d46e15c-apm-server-apm-ui-worker-management-list"
created_at = "2026-03-31T06:13:21.657306Z"
updated_at = "2026-03-31T07:29:56.040384Z"
+++

## Spec

### Problem

The worker activity panel (added in Step 7a, ticket 651f8a63) is read-only: supervisors can see which workers are running but cannot stop a misbehaving worker or reassign a stalled ticket from the browser. Two controls are missing:

1. Stop worker - no way to SIGTERM an individual worker process from the UI. The CLI has apm workers --kill but the HTTP API has no DELETE endpoint.
2. Reassign ticket - no way to call the equivalent of apm take from the UI. A ticket stuck in_progress under a crashed or gone worker requires CLI access to reassign.

Additionally, the GET /api/workers response (specced in Step 7a) does not include the ticket branch, which is useful context when stopping a worker or deciding whether to reassign.

Current state (after Steps 7a and 8): WorkerActivityPanel shows a table of live/crashed workers with pid, ticket_id, title, state, agent, elapsed, and status. TicketDetail shows transition buttons. Neither has stop or reassign controls.

Desired state: WorkerActivityPanel gains a Stop button per row (live workers only). TicketDetail gains a Reassign to me button. The backend gains DELETE /api/workers/:pid and POST /api/tickets/:id/take.

### Acceptance criteria

- [ ] GET /api/workers response includes a `branch` field (the ticket's branch name) for each worker entry
- [ ] DELETE /api/workers/:pid returns 204 when the given PID is found in a worktree pid file, is alive, and is successfully sent SIGTERM
- [ ] DELETE /api/workers/:pid removes the `.apm-worker.pid` file from the worktree after sending SIGTERM
- [ ] DELETE /api/workers/:pid returns 404 when no `.apm-worker.pid` file in any worktree contains the given PID
- [ ] DELETE /api/workers/:pid returns 409 when the PID is found in a pid file but the process is no longer alive (stale pid file)
- [ ] POST /api/tickets/:id/take reassigns the ticket's agent field to the server's resolved agent name (APM_AGENT_NAME env var, falling back to USER) and returns 200 with the updated ticket JSON
- [ ] POST /api/tickets/:id/take returns 404 when the ticket id does not exist
- [ ] WorkerActivityPanel shows a "Stop" button for each worker row where status is "running"
- [ ] Clicking "Stop" in WorkerActivityPanel calls DELETE /api/workers/:pid; on success the worker list refreshes and the row disappears or shows "crashed"
- [ ] Clicking "Stop" disables the button while the DELETE request is in-flight
- [ ] On a DELETE failure, WorkerActivityPanel shows an inline error message near the row
- [ ] TicketDetail shows a "Reassign to me" button when a ticket is selected
- [ ] Clicking "Reassign to me" calls POST /api/tickets/:id/take and, on success, updates the agent badge in the detail panel
- [ ] On a take failure, TicketDetail shows an inline error message near the button
- [ ] npm run build in apm-ui/ exits 0 with no TypeScript errors
- [ ] cargo test --workspace passes

### Out of scope

- Killing a worker by ticket id from the API (the CLI uses ticket id, but the HTTP endpoint uses PID directly — no ticket id → pid lookup needed in this ticket)
- Sending signals other than SIGTERM (SIGKILL fallback, graceful drain) — not needed at this scale
- The log tail viewer — covered by ticket e9ba2503 (Step 14)
- Authentication or authorisation for the stop/take actions — no auth layer exists yet
- The priority queue in the bottom half of the left column — covered by ticket 7777cf5c (Step 7b)
- Streaming or SSE updates for the worker list — polling every 5 seconds (established in Step 7a) is sufficient
- Reassigning a ticket to a specific named agent other than the server's own identity — the UI has no agent picker at this step

### Approach

How the implementation will work.

### Open questions



### Amendment requests



## History

| When | From | To | By |
|------|------|----|----|
| 2026-03-31T06:13Z | — | new | apm |
| 2026-03-31T07:29Z | new | in_design | philippepascal |