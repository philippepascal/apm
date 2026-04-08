+++
id = "7da9673f"
title = "apm work --daemon: keep dispatching workers as tickets become actionable"
state = "closed"
priority = 0
effort = 3
risk = 2
author = "philippepascal"
agent = "8702"
branch = "ticket/7da9673f-apm-work-daemon-keep-dispatching-workers"
created_at = "2026-03-30T17:27:51.137680Z"
updated_at = "2026-03-30T19:54:55.060754Z"
+++

## Spec

### Problem

Currently `apm work` exits as soon as there are no more actionable tickets — either because all slots are filled or the queue is empty. If the supervisor is away and a worker finishes (freeing a slot) or a ticket transitions to `ready` (new work becomes available), nothing picks it up. The supervisor has to manually re-run `apm work`.

`apm work --daemon` would keep the process alive, polling at a configurable interval, and dispatch new workers as soon as slots open up or actionable tickets appear. This enables fully unattended operation: start the daemon, walk away, come back to completed work.

The daemon should be interruptible with Ctrl-C and should log each dispatch cycle clearly so the supervisor can see what happened while away.

### Acceptance criteria

- [x] `apm work --daemon` continues running after `apm next` returns null (queue exhausted)
- [x] When a worker finishes and a slot opens, the daemon immediately re-checks for actionable tickets without waiting for the poll interval
- [x] When the poll interval elapses with no workers finishing, the daemon re-checks for actionable tickets
- [x] `apm work --daemon --interval <N>` sets the poll interval to N seconds; default is 30
- [x] Each dispatch cycle logs a timestamped line: ticket dispatched, worker finished, or no tickets found with seconds until next check
- [x] Ctrl-C stops the daemon; workers already running continue to completion as independent processes
- [x] `apm work` without `--daemon` retains existing behaviour: exits when the queue is exhausted and all workers finish
- [x] `apm work --daemon --dry-run` exits immediately with an error message

### Out of scope

- Daemonizing (fork/detach, systemd/launchd integration, PID files for the daemon process itself)
- Auto-restarting crashed workers
- Persistent dispatch state across daemon restarts
- SIGHUP config reload
- Configuring the default poll interval in `apm.toml` (CLI flag only)
- Stopping or reaping workers that are already running when the daemon exits

### Approach

**Files changed:** `apm/src/main.rs`, `apm/src/cmd/work.rs`, `apm/Cargo.toml`

1. **`apm/Cargo.toml`** — add `ctrlc = "3"` dependency for SIGINT handling.

2. **`apm/src/main.rs`** — add two flags to the `Work` subcommand variant:
   ```
   --daemon / -d   bool   keep dispatching in a continuous loop
   --interval <N>  u64    poll interval in seconds (default 30, only meaningful with --daemon)
   ```
   Pass both to `cmd::work::run()`.

3. **`apm/src/cmd/work.rs`** — modify `run()` signature to accept `daemon: bool` and `interval_secs: u64`.

   **Guard:** if `daemon && dry_run`, print error and return `Err(...)` immediately.

   **Signal flag:** before entering the loop, register a ctrlc handler that sets an `Arc<AtomicBool>` (`interrupted`).

   **Loop changes** — the existing loop already has the right skeleton. Two changes:
   - Replace the `no_more` early-exit condition with daemon-aware logic:
     - Non-daemon: keep `no_more` flag; when set and workers empty, break as today.
     - Daemon: never set `no_more`; instead, when `spawn_next_worker()` returns `None`, record `next_poll = Instant::now() + Duration::from_secs(interval_secs)` and log the "no tickets found, next check in Ns" message.
   - On each iteration, check `interrupted.load(Relaxed)` and break if true.

   **Slot-open fast-path:** track whether at least one worker was reaped in the current iteration (`reaped` bool). If `reaped`, reset `next_poll = Instant::now()` so the next dispatch attempt happens immediately rather than waiting out the interval.

   **Loop sleep:** the existing 500ms sleep when at capacity is fine. When in daemon mode with no active workers and waiting for the next poll, keep sleeping 500ms per tick and check `interrupted` and `Instant::now() >= next_poll` each tick.

   **Timestamped logging** — add a small `log(msg: &str)` helper that prints `[HH:MM:SS] {msg}` using `chrono::Local::now()` (already a workspace dep). Call it at:
   - Worker dispatched: `[HH:MM:SS] Dispatched worker for ticket #<id> "<title>"`
   - Worker reaped: `[HH:MM:SS] Worker for ticket #<id> finished`
   - No tickets: `[HH:MM:SS] No actionable tickets; next check in <N>s`
   - Daemon stopping: `[HH:MM:SS] Daemon interrupted, stopping`

   **Non-daemon path** is unchanged except for the log helper calls (optional — can keep existing println! there to minimise diff).

4. **Order of steps:**
   1. Add `ctrlc` dep and compile-check.
   2. Add CLI flags and thread them through.
   3. Add guard for `--daemon --dry-run`.
   4. Add `log()` helper.
   5. Add signal handler + `interrupted` flag.
   6. Refactor loop: extract `reaped` tracking, add `next_poll` / daemon branch.
   7. Write tests (see below).

5. **Tests** — add to `apm/src/cmd/work.rs` or `apm-core/tests/`:
   - A unit test that the `--daemon --dry-run` combination returns an error.
   - An integration test (temp git repo, fake tickets) is not needed for the polling loop itself since it involves timing; rely on manual verification. Document in the PR test plan.

### Open questions



### Amendment requests



## History

| When | From | To | By |
|------|------|----|----|
| 2026-03-30T17:27Z | — | new | philippepascal |
| 2026-03-30T17:29Z | new | in_design | philippepascal |
| 2026-03-30T17:33Z | in_design | specd | claude-0330-1730-b4e1 |
| 2026-03-30T19:19Z | specd | ready | apm |
| 2026-03-30T19:24Z | ready | in_progress | philippepascal |
| 2026-03-30T19:28Z | in_progress | implemented | claude-0330-1924-w7k2 |
| 2026-03-30T19:48Z | implemented | accepted | apm-sync |
| 2026-03-30T19:54Z | accepted | closed | apm-sync |