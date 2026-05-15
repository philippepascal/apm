+++
id = "996fef40"
title = "apm work non-daemon exits early when concurrency constraint temporarily blocks dispatch"
state = "in_progress"
priority = 0
effort = 1
risk = 1
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/996fef40-apm-work-non-daemon-exits-early-when-con"
created_at = "2026-05-07T02:20:24.657545Z"
updated_at = "2026-05-15T02:01:27.877468Z"
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

- Changes to `apm work --dry-run` path
- Changes to `spawn_next_worker` itself
- Error-arm retry behaviour (`Err(e)` at work.rs:136)
- Daemon-mode shutdown or signal-handling logic
- Adding integration tests that span the full worker-spawn lifecycle

### Approach

#### Change `apm/src/cmd/work.rs` lines 89–93

Replace the daemon-only guard around the `no_more` reset with a guard that fires for both modes, keeping the `next_poll` reset inside the daemon branch:

```rust
// Before
// In daemon mode: a reaped worker opens a slot — check immediately.
if daemon && reaped {
    next_poll = Instant::now();
    no_more = false;
}

// After
// A reaped worker opens a slot — retry dispatch in both modes.
if reaped {
    if daemon {
        next_poll = Instant::now();
    }
    no_more = false;
}
```

This is the only code change required. No other call sites reference `no_more`.

#### Why the exit condition remains correct

After the fix, when there are truly no tickets left:

1. Last worker finishes → `reaped = true` → `no_more = false`
2. Next iteration: `spawn_next_worker` returns `Ok(None)` → `no_more = true`
3. Next iteration: `!daemon && no_more && workers.is_empty()` → `break`

The loop makes one extra `spawn_next_worker` call compared to the old (buggy) path, but exits correctly. No infinite-loop risk.

#### Tests

The two existing unit tests (`daemon_dry_run_is_error`, `sig_count_increments_correctly`) do not cover the scheduling loop and need no changes. The loop logic cannot be unit-tested without a real git repo; no new tests are required for this surgical fix.

### Open questions

**Q:** - Blocked: Edit tool and Bash (python3) both require user approval to write to the worktree path /Users/philippe/repos/apm/.apm--worktrees/ticket-996fef40-apm-work-non-daemon-exits-early-when-con/apm/src/cmd/work.rs. The fix is a 5-line change (lines 89-93 of apm/src/cmd/work.rs) replacing 'if daemon && reaped' with 'if reaped { if daemon { ... } }' per the Approach section. Needs write permission granted for the worktree path.

### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-05-07T02:20Z | — | new | philippepascal |
| 2026-05-07T02:30Z | new | groomed | philippepascal |
| 2026-05-07T02:31Z | groomed | in_design | philippepascal |
| 2026-05-07T02:32Z | in_design | specd | claude-0507-0231-85e0 |
| 2026-05-07T02:52Z | specd | ready | philippepascal |
| 2026-05-14T21:35Z | ready | in_progress | philippe |
| 2026-05-15T01:21Z | in_progress | ready | philippe |
| 2026-05-15T02:01Z | ready | in_progress | philippe |