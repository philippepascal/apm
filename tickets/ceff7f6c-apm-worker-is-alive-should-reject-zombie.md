+++
id = "ceff7f6c"
title = "apm worker is_alive should reject zombie/defunct processes"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/ceff7f6c-apm-worker-is-alive-should-reject-zombie"
created_at = "2026-04-28T00:50:51.047540Z"
updated_at = "2026-04-28T01:02:27.065486Z"
+++

## Spec

### Problem

apm-core::worker::is_alive(pid) treats zombie/defunct processes as alive. It uses kill -0 to verify the PID is present in the process table, but kill -0 succeeds for zombies too — they remain in the table until reaped by their parent.\n\nReal incident: ticket ec5e9fe3 had a worker spawn, immediately exit, and become a zombie. ps -p 3227 -o stat returned Z. apm workers listed PID 3227 as the active worker, and the ticket was effectively unrecoverable through normal channels because APM thought a worker was still running.\n\nAffected call sites:\n- apm workers — false positives in the listing\n- epic_is_quiescent (apm-core/src/epic.rs) — falsely blocks apm refresh-epic and apm epic close when a worker has died as a zombie\n- apm-server/src/work.rs check_workers_alive — falsely reports an epic as occupied\n- apm-server/src/workers.rs — REST API reports zombie worker as alive\n\nThe fix is to extend is_alive so that, after confirming the PID exists, it also checks the process state via ps -p <pid> -o state= and returns false for any state beginning with Z.

### Acceptance criteria

- [ ] `is_alive(pid)` returns `false` when the process state reported by `ps -p <pid> -o state=` begins with `Z`
- [ ] `is_alive(pid)` returns `true` for the current process PID (existing behaviour preserved)
- [ ] `is_alive(pid)` returns `false` for a PID that does not exist in the process table (existing behaviour preserved)
- [ ] A unit test for the zombie-state parsing helper covers: state string `"Z"` → zombie, `"Z+"` → zombie, `"S"` → not zombie, `"R"` → not zombie, empty string → not zombie
- [ ] `apm workers` does not list a zombie worker as alive
- [ ] `epic_is_quiescent` returns quiescent (no blockers) when the only remaining worker PID is a zombie

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
| 2026-04-28T00:51Z | new | groomed | philippepascal |
| 2026-04-28T01:02Z | groomed | in_design | philippepascal |