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
| 2026-03-31T07:29Z | new | in_design | philippepascal |