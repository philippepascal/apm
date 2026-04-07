+++
id = "7f213c43"
title = "when moving a ticket to ready the ui shows it as crashed"
state = "ready"
priority = 0
effort = 2
risk = 2
author = "philippepascal"
branch = "ticket/7f213c43-when-moving-a-ticket-to-ready-the-ui-sho"
created_at = "2026-04-07T00:18:08.759817Z"
updated_at = "2026-04-07T04:53:25.826395Z"
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

Modify the /api/workers endpoint in apm-server/src/workers.rs to filter out tickets whose current state does not involve an active worker. Only tickets in states that have a worker phase (in_design, in_progress) or that just completed one (worker_end/terminal states) should appear in the workers list.

Specifically, add a worker_states set containing states where a worker is expected to be active (in_design, in_progress). Exclude tickets from the workers response if their state is not in worker_states and not in ended_states. This way, tickets in ready, specd, groomed, etc. are excluded entirely rather than showing as crashed.

File to modify: apm-server/src/workers.rs — update the filtering logic around determine_status() to skip tickets that have no worker association in their current state.

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-04-07T00:18Z | — | new | philippepascal |
| 2026-04-07T01:17Z | new | groomed | apm |
| 2026-04-07T02:59Z | groomed | in_design | philippepascal |
| 2026-04-07T04:52Z | in_design | specd | claude-0406-fix-stuck |
| 2026-04-07T04:53Z | specd | ready | apm |
