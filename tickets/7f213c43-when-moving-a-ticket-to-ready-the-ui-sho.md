+++
id = "7f213c43"
title = "when moving a ticket to ready the ui shows it as crashed"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
branch = "ticket/7f213c43-when-moving-a-ticket-to-ready-the-ui-sho"
created_at = "2026-04-07T00:18:08.759817Z"
updated_at = "2026-04-07T02:59:03.359894Z"
+++

## Spec

### Problem

When a ticket transitions to the ready state, the UI incorrectly shows it in the Workers panel with a crashed status. The ticket should only appear in the Priority Queue panel at this point — it has not been dispatched to a worker yet.

Root cause: the /api/workers endpoint in apm-server/src/workers.rs uses determine_status() which classifies any ticket with a dead worker process as crashed if its state is not in the ended_states set. The ready state is neither terminal nor worker_end, so tickets that were previously in_design (with a spec-writer worker) show as crashed when they advance to ready.

### Acceptance criteria

- [ ] A ticket in ready state does not appear in the Workers panel
- [ ] A ticket in specd state does not appear in the Workers panel
- [ ] A ticket in in_progress state with a live worker shows as running in the Workers panel
- [ ] A ticket in in_progress state with a dead worker shows as crashed in the Workers panel
- [ ] A ticket that completed its worker phase (implemented, closed, etc.) shows as ended, not crashed

### Out of scope

- Changing how workers are spawned or killed
- Modifying the Priority Queue panel logic
- Adding worker lifecycle events or logging
- Frontend-only filtering (the fix belongs in the backend API)

### Approach

How the implementation will work.

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-04-07T00:18Z | — | new | philippepascal |
| 2026-04-07T01:17Z | new | groomed | apm |
| 2026-04-07T02:59Z | groomed | in_design | philippepascal |