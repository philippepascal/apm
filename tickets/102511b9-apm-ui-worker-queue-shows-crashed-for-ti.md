+++
id = "102511b9"
title = "apm-ui: worker queue shows 'crashed' for tickets in terminal states"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
agent = "78978"
branch = "ticket/102511b9-apm-ui-worker-queue-shows-crashed-for-ti"
created_at = "2026-04-01T06:10:23.311626Z"
updated_at = "2026-04-01T06:13:02.387897Z"
+++

## Spec

### Problem

The worker queue UI fetches from /api/workers and renders each entry status as green running or red crashed. The handler in apm-server/src/workers.rs (line 53) sets status purely based on whether the OS process is alive: if is_alive(pid) { running } else { crashed }.

When an agent finishes its work on a ticket that reaches a terminal state (specd, implemented, closed, etc.), its process exits normally. The PID file persists in the worktree, so the entry keeps appearing in the queue but with the alarming red crashed badge. This is misleading: the worker completed normally, it did not crash.

The fix must consult the workflow config terminal flag (already defined on StateConfig in apm-core/src/config.rs) to distinguish the two cases. A dead process whose ticket is in a terminal state should be shown as ended with neutral gray styling. Only a dead process whose ticket is still in a non-terminal state should be shown as crashed.

### Acceptance criteria

- [ ] A worker whose process has exited and whose ticket is in a terminal state shows an ended badge with gray/neutral styling, not a red crashed badge
- [ ] A worker whose process has exited and whose ticket is NOT in a terminal state still shows a red crashed badge
- [ ] A worker whose process is still running shows a green running badge regardless of ticket state
- [ ] The set of terminal states is read from the workflow config (StateConfig.terminal == true) and not hardcoded in the server or client
- [ ] Adding or removing a terminal state in apm.toml is reflected in worker status without a code change

### Out of scope

Explicit list of what this ticket does not cover.

### Approach

How the implementation will work.

### Open questions



### Amendment requests



## History

| When | From | To | By |
|------|------|----|----|
| 2026-04-01T06:10Z | — | new | philippepascal |
| 2026-04-01T06:13Z | new | in_design | philippepascal |