+++
id = "996fef40"
title = "apm work non-daemon exits early when concurrency constraint temporarily blocks dispatch"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/996fef40-apm-work-non-daemon-exits-early-when-con"
created_at = "2026-05-07T02:20:24.657545Z"
updated_at = "2026-05-07T02:31:04.374458Z"
+++

## Spec

### Problem

In non-daemon mode, apm work exits early when a concurrency constraint (max_workers_on_default=1 or max_workers_per_epic) causes spawn_next_worker to return Ok(None) while workers are still running. When those workers finish and the slot reopens, no_more is never reset so the loop breaks instead of dispatching remaining tickets. Daemon mode handles this correctly with 'if daemon && reaped { no_more = false; }' at apm/src/cmd/work.rs:90-93. Non-daemon mode has no equivalent reset. Fix: extend the reset to both modes — 'if reaped { if daemon { next_poll = Instant::now(); } no_more = false; }'. Repro: set max_workers_on_default=1, have 3+ ready tickets targeting the default branch, run apm work without -d — only the first ticket gets worked.

### Acceptance criteria

- [ ] Running `apm work` with `max_workers_on_default=1` and 3+ ready tickets on the default branch dispatches and completes all tickets, not just the first
- [ ] Running `apm work` with `max_workers_per_epic=1` and 3+ ready tickets in the same epic dispatches and completes all tickets
- [ ] After the fix, `apm work` exits cleanly once all tickets are processed and no new ones remain (i.e. the break condition `!daemon && no_more && workers.is_empty()` still fires correctly)
- [ ] `apm work -d` (daemon mode) behaviour is unchanged: a reaped worker still resets `next_poll` to `Instant::now()` and clears `no_more`

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
| 2026-05-07T02:20Z | — | new | philippepascal |
| 2026-05-07T02:30Z | new | groomed | philippepascal |
| 2026-05-07T02:31Z | groomed | in_design | philippepascal |