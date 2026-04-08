+++
id = "102511b9"
title = "apm-ui: worker queue shows 'crashed' for tickets in terminal states"
state = "closed"
priority = 0
effort = 2
risk = 1
author = "philippepascal"
agent = "3031"
branch = "ticket/102511b9-apm-ui-worker-queue-shows-crashed-for-ti"
created_at = "2026-04-01T06:10:23.311626Z"
updated_at = "2026-04-01T07:47:26.526540Z"
+++

## Spec

### Problem

The worker queue UI fetches from /api/workers and renders each entry status as green running or red crashed. The handler in apm-server/src/workers.rs (line 53) sets status purely based on whether the OS process is alive: if is_alive(pid) { running } else { crashed }.

When an agent finishes its work on a ticket that reaches a terminal state (specd, implemented, closed, etc.), its process exits normally. The PID file persists in the worktree, so the entry keeps appearing in the queue but with the alarming red crashed badge. This is misleading: the worker completed normally, it did not crash.

The fix must consult the workflow config terminal flag (already defined on StateConfig in apm-core/src/config.rs) to distinguish the two cases. A dead process whose ticket is in a terminal state should be shown as ended with neutral gray styling. Only a dead process whose ticket is still in a non-terminal state should be shown as crashed.

### Acceptance criteria

- [x] A worker whose process has exited and whose ticket is in a terminal state shows an ended badge with gray/neutral styling, not a red crashed badge
- [x] A worker whose process has exited and whose ticket is NOT in a terminal state still shows a red crashed badge
- [x] A worker whose process is still running shows a green running badge regardless of ticket state
- [x] The set of terminal states is read from the workflow config (StateConfig.terminal == true) and not hardcoded in the server or client
- [x] Adding or removing a terminal state in apm.toml is reflected in worker status without a code change

### Out of scope

- Removing stale PID files from worktrees after a worker ends
- Adding a fourth status value beyond running, crashed, and ended
- Changing the polling interval or the overall worker queue UI layout
- Surfacing workflow config terminal state definitions via a separate API endpoint

### Approach

All changes are confined to two files.

**apm-server/src/workers.rs**

1. Inside collect_workers, after loading tickets with load_all_from_git, also load the config:
   let config = apm_core::config::Config::load(root)?;

2. Build a HashSet of terminal state IDs:
   let terminal_states: std::collections::HashSet<_> = config.workflow.states.iter()
       .filter(|s| s.terminal)
       .map(|s| s.id.as_str())
       .collect();

3. Replace the single-line status assignment (line 53) with:
   let status = if apm_core::worker::is_alive(pid) {
       "running"
   } else if terminal_states.contains(state.as_str()) {
       "ended"
   } else {
       "crashed"
   };
   Note: state is already resolved from the ticket lookup at this point in the function.

4. Add a unit test that constructs an in-memory worker list with state set to a known terminal state and asserts status == "ended". The existing test infrastructure in the mod tests block can be extended.

**apm-ui/src/components/WorkerActivityPanel.tsx**

1. Extend the WorkerInfo interface on line 10:
   status: 'running' | 'crashed' | 'ended'

2. Add an else-if branch in the status cell renderer after the running branch and before the existing else (crashed):
   } else if (w.status === 'ended') {
     <span className="inline-flex items-center px-1.5 py-0.5 rounded bg-gray-100 text-gray-500">
       ended
     </span>
   }

No schema changes, no new files, no changes to AppState or routing.

### Open questions



### Amendment requests



## History

| When | From | To | By |
|------|------|----|----|
| 2026-04-01T06:10Z | — | new | philippepascal |
| 2026-04-01T06:13Z | new | in_design | philippepascal |
| 2026-04-01T06:15Z | in_design | specd | claude-0401-0613-69f0 |
| 2026-04-01T06:25Z | specd | ready | apm |
| 2026-04-01T07:19Z | ready | in_progress | philippepascal |
| 2026-04-01T07:23Z | in_progress | implemented | claude-0401-0719-d458 |
| 2026-04-01T07:46Z | implemented | accepted | apm-sync |
| 2026-04-01T07:47Z | accepted | closed | apm-sync |