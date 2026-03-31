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


### Out of scope

Explicit list of what this ticket does not cover.

### Approach

How the implementation will work.

### Open questions



### Amendment requests



## History

| When | From | To | By |
|------|------|----|----|
| 2026-03-31T18:35Z | — | new | claude-0331-1200-a7b9 |
| 2026-03-31T19:02Z | new | in_design | philippepascal |