+++
id = "ceff7f6c"
title = "apm worker is_alive should reject zombie/defunct processes"
state = "new"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/ceff7f6c-apm-worker-is-alive-should-reject-zombie"
created_at = "2026-04-28T00:50:51.047540Z"
updated_at = "2026-04-28T00:50:51.047540Z"
+++

## Spec

### Problem

`apm-core::worker::is_alive(pid)` (the function used by `apm workers`, `epic_is_quiescent`, and the dispatcher's epic-concurrency cap) treats zombie/defunct processes as alive. It only verifies that the PID is present in the process table; it does not check the process state.

Real incident: ticket ec5e9fe3 had a worker spawn, immediately exit, and become a zombie (`<defunct>`). `ps -p 3227 -o stat` returned `Z`. `apm workers` listed PID 3227 as the active worker, and the ticket was effectively unrecoverable through normal channels because APM thought a worker was still running.

Affected call sites:
- `apm workers` — false positives in the listing
- `epic_is_quiescent` (added by ticket 2973e208 in epic 5ea30227) — falsely blocks `apm refresh-epic` and `apm epic close` when a worker has died as a zombie
- The dispatcher's "is this epic at capacity?" check (`Config::blocked_epics` consumers in `start.rs::run_next`) — falsely caps an epic that is actually idle

Fix direction: in `worker::is_alive`, after confirming the PID exists, check process state and return false if the state is `Z` (zombie) or otherwise indicates the process has exited. On macOS, `ps -p <pid> -o state=` returns a single character (`Z` for zombie); on Linux, the same flag works, or read `/proc/<pid>/stat` field 3. A small cross-platform helper that shells out to `ps -p <pid> -o state=` is the simplest portable approach.

### Acceptance criteria

Checkboxes; each one independently testable.

### Out of scope

Explicit list of what this ticket does not cover.

### Approach

How the implementation will work.

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-04-28T00:50Z | — | new | philippepascal |
