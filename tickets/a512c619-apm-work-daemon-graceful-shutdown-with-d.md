+++
id = "a512c619"
title = "apm work --daemon: graceful shutdown with double-Ctrl+C escape hatch"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "claude-0331-1200-a7b9"
agent = "4585"
branch = "ticket/a512c619-apm-work-daemon-graceful-shutdown-with-d"
created_at = "2026-03-31T18:35:38.898908Z"
updated_at = "2026-03-31T19:02:23.764652Z"
+++

## Spec

### Problem

When `apm work --daemon` receives a SIGINT (Ctrl+C), it currently sets an `interrupted` flag and breaks out of the dispatch loop on the next iteration, leaving any spawned workers running as orphaned independent processes. The daemon exits without waiting for those workers to finish.

This means the operator has no way to tell the daemon "stop accepting new work and wait for current agents to land cleanly". Pressing Ctrl+C produces an abrupt exit every time, with no chance to drain the queue gracefully.

The desired behaviour follows the standard two-stage shutdown pattern used by process supervisors: a first Ctrl+C requests graceful shutdown (stop dispatching, wait for running workers); a second Ctrl+C acts as an escape hatch that forces an immediate exit when the operator cannot wait any longer.

### Acceptance criteria

- [ ] First Ctrl+C while workers are running prints a message stating how many workers are still running and that a second Ctrl+C will force-exit
- [ ] After the first Ctrl+C the daemon stops dispatching new workers
- [ ] After the first Ctrl+C the daemon continues reaping workers until all have finished, then exits cleanly
- [ ] A second Ctrl+C at any point during the drain phase exits immediately and prints a message that workers may still be running
- [ ] When the first Ctrl+C is received and no workers are running the daemon exits immediately without waiting
- [ ] The non-daemon (one-shot) mode is unaffected: Ctrl+C behaviour there remains unchanged

### Out of scope

- Sending SIGTERM or any other signal to workers — they are independent processes and continue running regardless of how the daemon exits
- A configurable drain timeout — the operator already has the double-Ctrl+C escape hatch for cases where waiting is not an option
- Changes to non-daemon (`apm work` without `--daemon`) shutdown behaviour
- Changes to `apm start` or any other command

### Approach

How the implementation will work.

### Open questions



### Amendment requests



## History

| When | From | To | By |
|------|------|----|----|
| 2026-03-31T18:35Z | — | new | claude-0331-1200-a7b9 |
| 2026-03-31T19:02Z | new | in_design | philippepascal |