+++
id = "6903a8d1"
title = "worker status bug"
state = "in_progress"
priority = 0
effort = 1
risk = 1
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/6903a8d1-worker-status-bug"
created_at = "2026-04-04T16:07:08.053019Z"
updated_at = "2026-04-04T18:17:28.650565Z"
+++

## Spec

### Problem

The `apm workers` CLI command (`apm/src/cmd/workers.rs`) always displays "crashed" for any dead worker PID, regardless of whether the ticket has already reached a normal worker-completion state. A worker that finishes cleanly and transitions the ticket to `specd` or `implemented` (states with `worker_end = true`) is indistinguishable from one that actually crashed mid-run.

Ticket fa2dce31 already fixed the server-side equivalent in `apm-server/src/workers.rs` by building an `ended_states` set from `workflow.states` (union of `terminal` and `worker_end` states) and using it in `determine_status()`. The `StateConfig` struct already carries the `worker_end: bool` field and `.apm/workflow.toml` already marks `specd` and `implemented` with `worker_end = true`. Only the CLI command was not updated.

### Acceptance criteria

- [ ] `apm workers` shows "crashed" for a dead worker whose ticket is in a state where neither `worker_end` nor `terminal` is true
- [ ] `apm workers` shows the ticket's actual state name (not "crashed") for a dead worker whose ticket is in a state with `worker_end = true`
- [ ] `apm workers` shows the ticket's actual state name (not "crashed") for a dead worker whose ticket is in a `terminal = true` state
- [ ] `apm workers` shows the ticket's actual state for a live worker (existing behaviour unchanged)

### Out of scope

- Server-side workers API (`apm-server/src/workers.rs`) — already fixed in ticket fa2dce31
- Adding a `worker_end` field to `StateConfig` — already done in fa2dce31
- Updating `.apm/workflow.toml` with `worker_end = true` — already done in fa2dce31
- Removing or cleaning up stale PID files after a worker completes
- Changes to the elapsed or PID columns for completed workers
- UI or server worker panel presentation

### Approach

**Single file change: `apm/src/cmd/workers.rs`**

1. After loading `config` (already done at the top of `list()`), build an `ended_states` set — same pattern as `collect_workers()` in `apm-server/src/workers.rs`:

```rust
let ended_states: std::collections::HashSet<&str> = config
    .workflow
    .states
    .iter()
    .filter(|s| s.terminal || s.worker_end)
    .map(|s| s.id.as_str())
    .collect();
```

2. Replace the dead-worker branch of the state column (currently always `"crashed"`) with an ended-states check:

```rust
let state = if alive {
    t.map(|t| t.frontmatter.state.as_str()).unwrap_or("—").to_string()
} else {
    let ticket_state = t.map(|t| t.frontmatter.state.as_str()).unwrap_or("");
    if ended_states.contains(ticket_state) {
        ticket_state.to_string()
    } else {
        "crashed".to_string()
    }
};
```

3. Add unit tests in `apm/src/cmd/workers.rs` (or inline in the same file) covering the three dead-worker cases: ended via `worker_end`, ended via `terminal`, and genuinely crashed. Mirror the test structure from `apm-server/src/workers.rs::determine_status_*` tests.

No changes needed to `apm-core/src/config.rs`, `.apm/workflow.toml`, or any other file.

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-04-04T16:07Z | — | new | philippepascal |
| 2026-04-04T16:39Z | new | groomed | apm |
| 2026-04-04T16:43Z | groomed | in_design | philippepascal |
| 2026-04-04T16:46Z | in_design | specd | claude-0404-1643-b7f2 |
| 2026-04-04T17:26Z | specd | ready | apm |
| 2026-04-04T17:35Z | ready | in_progress | philippepascal |
| 2026-04-04T18:17Z | in_progress | ready | apm |
| 2026-04-04T18:17Z | ready | in_progress | philippepascal |
