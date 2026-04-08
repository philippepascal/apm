+++
id = "5a4ad4bd"
title = "apm work improvement"
state = "closed"
priority = 0
effort = 2
risk = 2
author = "apm"
agent = "28354"
branch = "ticket/5a4ad4bd-apm-work-improvement"
created_at = "2026-03-30T19:21:34.679718Z"
updated_at = "2026-03-30T19:44:03.885151Z"
+++

## Spec

### Problem

The `apm work` dispatch loop sets a permanent `no_more` flag the first time `spawn_next_worker` returns `None` (no actionable ticket found). Once that flag is set, the loop stops trying to spawn new workers and merely waits for existing workers to drain. This means that if a running worker finishes and its ticket transitions unblock another ticket—making it newly actionable—the loop will never pick it up. The result: `apm work` under-utilises available worker slots after the initial burst, and newly-unblocked tickets are silently ignored until a completely new `apm work` invocation is run by the user.

### Acceptance criteria

- [x] When `apm work` has fewer running workers than `max_concurrent` and the current poll finds no actionable ticket, it sleeps for a poll interval and retries—it does not exit or permanently stop polling.
- [x] When a running worker finishes and the resulting state change makes a previously-blocked ticket actionable, `apm work` picks it up within one poll interval.
- [x] `apm work` exits only when all workers have drained and the latest poll found no actionable ticket.
- [x] Workers are spawned immediately when a ticket is ready, without waiting for the idle poll interval.
- [x] The idle poll interval (when under `max_concurrent` but no ticket is available) is at least 10 s and distinct from the 500 ms at-capacity reap sleep.

### Out of scope

- Making the poll interval user-configurable (a sensible constant is sufficient for now)
- Changes to `apm start`, `apm start --next`, or any other subcommand
- Changes to the worker spawning logic in `spawn_next_worker`
- Dry-run mode changes

### Approach

**File to change:** `apm/src/cmd/work.rs` only.

**Core change:** replace the permanent `no_more: bool` flag with a `last_poll_empty: bool` flag.  The key difference is that `last_poll_empty` is reset to `false` whenever a worker is reaped—because a finished worker may have transitioned a ticket in a way that unblocks new ones.

Add a constant at the top of the file:
```rust
const IDLE_POLL_INTERVAL: std::time::Duration = std::time::Duration::from_secs(30);
```

Updated loop:

1. **Reap finished workers** (same as today).  If `workers.len()` decreased (any worker was reaped), set `last_poll_empty = false`.
2. **Exit:** `workers.is_empty() && last_poll_empty` → break.
3. **Under capacity** (`workers.len() < max_concurrent`):
   - Call `spawn_next_worker`:
     - `Ok(Some(...))` → push worker, `last_poll_empty = false`
     - `Ok(None)` → `last_poll_empty = true`, sleep `IDLE_POLL_INTERVAL`
     - `Err(e)` → print warning, `last_poll_empty = true`, sleep `IDLE_POLL_INTERVAL`
4. **At capacity:** sleep 500 ms (unchanged).

No changes are needed to the summary, dry-run, or exit-code logic.

### Open questions



### Amendment requests



## History

| When | From | To | By |
|------|------|----|----|
| 2026-03-30T19:21Z | — | new | apm |
| 2026-03-30T19:23Z | new | in_design | philippepascal |
| 2026-03-30T19:28Z | in_design | specd | claude-0330-1930-b7f2 |
| 2026-03-30T19:30Z | specd | ready | apm |
| 2026-03-30T19:35Z | ready | in_progress | philippepascal |
| 2026-03-30T19:38Z | in_progress | implemented | claude-0330-1935-cea8 |
| 2026-03-30T19:44Z | implemented | closed | apm |