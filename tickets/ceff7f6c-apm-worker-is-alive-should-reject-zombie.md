+++
id = "ceff7f6c"
title = "apm worker is_alive should reject zombie/defunct processes"
state = "in_progress"
priority = 0
effort = 2
risk = 2
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/ceff7f6c-apm-worker-is-alive-should-reject-zombie"
created_at = "2026-04-28T00:50:51.047540Z"
updated_at = "2026-04-28T06:05:09.939393Z"
+++

## Spec

### Problem

apm-core::worker::is_alive(pid) treats zombie/defunct processes as alive. It uses kill -0 to verify the PID is present in the process table, but kill -0 succeeds for zombies too — they remain in the table until reaped by their parent.\n\nReal incident: ticket ec5e9fe3 had a worker spawn, immediately exit, and become a zombie. ps -p 3227 -o stat returned Z. apm workers listed PID 3227 as the active worker, and the ticket was effectively unrecoverable through normal channels because APM thought a worker was still running.\n\nAffected call sites:\n- apm workers — false positives in the listing\n- epic_is_quiescent (apm-core/src/epic.rs) — falsely blocks apm refresh-epic and apm epic close when a worker has died as a zombie\n- apm-server/src/work.rs check_workers_alive — falsely reports an epic as occupied\n- apm-server/src/workers.rs — REST API reports zombie worker as alive\n\nThe fix is to extend is_alive so that, after confirming the PID exists, it also checks the process state via ps -p <pid> -o state= and returns false for any state beginning with Z.

### Acceptance criteria

- [x] `is_alive(pid)` returns `false` when the process state reported by `ps -p <pid> -o state=` begins with `Z`
- [x] `is_alive(pid)` returns `true` for the current process PID (existing behaviour preserved)
- [x] `is_alive(pid)` returns `false` for a PID that does not exist in the process table (existing behaviour preserved)
- [x] A unit test for the zombie-state parsing helper covers: state string `"Z"` → zombie, `"Z+"` → zombie, `"S"` → not zombie, `"R"` → not zombie, empty string → not zombie
- [x] `apm workers` does not list a zombie worker as alive
- [ ] `epic_is_quiescent` returns quiescent (no blockers) when the only remaining worker PID is a zombie

### Out of scope

- Reaping or cleaning up zombie processes (that is the parent process's responsibility)\n- Handling other unusual process states such as T (stopped/traced) or D (uninterruptible sleep) — only Z is treated as dead\n- Windows support (APM targets macOS and Linux)\n- Changes to the pid-file format or schema\n- UI changes to apm workers output beyond the alive/dead flag already computed by is_alive

### Approach

All changes are confined to `apm-core/src/worker.rs`. No other files need to change — all call sites use `is_alive` and benefit automatically.

**1. Add a private `state_is_zombie(state: &str) -> bool` pure helper**

Accepts the trimmed stdout of `ps -p <pid> -o state=` and returns `true` if the string starts with `'Z'`. Pure and trivially unit-testable without spawning any processes.

**2. Add a private `process_state(pid: u32) -> Option<String>` helper**

Shells out to `ps -p <pid> -o state=` and returns the trimmed stdout on success, `None` on any error (command not found, non-zero exit when the PID has just vanished, etc.).

**3. Update `is_alive` to reject zombies**

Keep the `kill -0` fast-path for the common case (PID not in process table at all), then add the zombie check. If `process_state` returns `None` (race: process exited between `kill -0` and `ps`), treat as dead.

Logic:
- Run `kill -0 <pid>` — if it fails, return `false` immediately.
- Call `process_state(pid)`:
  - `Some(s)` → return `!state_is_zombie(&s)`
  - `None` → return `false` (vanished between the two checks)

**4. Add unit tests for `state_is_zombie`**

Cover: `"Z"` → true, `"Z+"` → true, `"  Z  "` (with whitespace) → true, `"S"` → false, `"R"` → false, `""` → false.

**5. Add a zombie integration test for `is_alive`**

Spawn a child process (`true` or similar), record its PID, sleep 100 ms to let it exit and become a zombie (parent has not called `wait()`), assert `is_alive(pid)` returns `false`, then reap with `child.wait()`. This test exercises the full `kill -0` + `ps` path against a real zombie on CI.

The existing `is_alive_returns_true_for_current_process` test is unchanged and continues to pass.

**Constraints**

- `ps -p <pid> -o state=` is POSIX-compatible and works on macOS and Linux, both platforms APM targets. No `/proc` reading needed.
- The `kill -0` fast-path is preserved to avoid the `ps` overhead for the common "PID does not exist" case.
- No changes to public API surface; `is_alive` signature is unchanged.

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-04-28T00:50Z | — | new | philippepascal |
| 2026-04-28T00:51Z | new | groomed | philippepascal |
| 2026-04-28T01:02Z | groomed | in_design | philippepascal |
| 2026-04-28T01:05Z | in_design | specd | claude-0428-0102-e8b0 |
| 2026-04-28T06:00Z | specd | ready | philippepascal |
| 2026-04-28T06:05Z | ready | in_progress | philippepascal |