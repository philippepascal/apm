+++
id = "a512c619"
title = "apm work --daemon: graceful shutdown with double-Ctrl+C escape hatch"
state = "closed"
priority = 85
effort = 2
risk = 2
author = "claude-0331-1200-a7b9"
agent = "6159"
branch = "ticket/a512c619-apm-work-daemon-graceful-shutdown-with-d"
created_at = "2026-03-31T18:35:38.898908Z"
updated_at = "2026-04-01T04:55:14.649551Z"
+++

## Spec

### Problem

When `apm work --daemon` receives a SIGINT (Ctrl+C), it currently sets an `interrupted` flag and breaks out of the dispatch loop on the next iteration, leaving any spawned workers running as orphaned independent processes. The daemon exits without waiting for those workers to finish.

This means the operator has no way to tell the daemon "stop accepting new work and wait for current agents to land cleanly". Pressing Ctrl+C produces an abrupt exit every time, with no chance to drain the queue gracefully.

The desired behaviour follows the standard two-stage shutdown pattern used by process supervisors: a first Ctrl+C requests graceful shutdown (stop dispatching, wait for running workers); a second Ctrl+C acts as an escape hatch that forces an immediate exit when the operator cannot wait any longer.

### Acceptance criteria

- [x] First Ctrl+C while workers are running prints a message stating how many workers are still running and that a second Ctrl+C will force-exit
- [x] After the first Ctrl+C the daemon stops dispatching new workers
- [x] After the first Ctrl+C the daemon continues reaping workers until all have finished, then exits cleanly
- [x] A second Ctrl+C at any point during the drain phase exits immediately and prints a message that workers may still be running
- [x] When the first Ctrl+C is received and no workers are running the daemon exits immediately without waiting
- [x] The non-daemon (one-shot) mode is unaffected: Ctrl+C behaviour there remains unchanged

### Out of scope

- Sending SIGTERM or any other signal to workers — they are independent processes and continue running regardless of how the daemon exits
- A configurable drain timeout — the operator already has the double-Ctrl+C escape hatch for cases where waiting is not an option
- Changes to non-daemon (`apm work` without `--daemon`) shutdown behaviour
- Changes to `apm start` or any other command

### Approach

All changes are in `apm/src/cmd/work.rs`.

**Signal counter**

Replace the `Arc<AtomicBool>` with an `Arc<AtomicUsize>` that counts how many times SIGINT has been received:

```rust
let sig_count = Arc::new(AtomicUsize::new(0));
let sig_count_clone = Arc::clone(&sig_count);
let _ = ctrlc::set_handler(move || {
    sig_count_clone.fetch_add(1, Ordering::Relaxed);
});
```

**Modified loop logic (daemon mode only)**

At the top of each loop iteration, read `sig_count`:

1. `sigs == 0` — normal operation, no change.
2. `sigs == 1` (first Ctrl+C):
   - If `workers.is_empty()`: log "Daemon stopped." and break immediately.
   - Otherwise, on the *first* iteration where `sigs` transitions to 1, print once: `"Graceful shutdown: waiting for N worker(s) to finish (Ctrl+C again to exit immediately)"`.
   - Stop dispatching (skip the `spawn_next_worker` call).
   - Continue the reap loop until `workers.is_empty()`, then break with `"All workers finished; exiting."`.
3. `sigs >= 2` (second Ctrl+C): log `"Forced exit; N worker(s) may still be running"` and break immediately.

Use a `bool` flag `drain_announced` to ensure the "waiting for N workers" message is only printed once.

**Non-daemon path**

No changes. The existing `if !daemon` early-exit guard at the top of the loop already keeps the paths separate. The `interrupted` variable can be removed if it becomes unused after the refactor, but only if nothing else references it.

**Order of steps**

1. Replace `AtomicBool` / `interrupted` with `AtomicUsize` / `sig_count` throughout `work.rs`.
2. Update the loop body as described above.
3. Verify `cargo test --workspace` passes (no existing tests cover this path, but compilation must succeed).
4. Add an integration test in `apm/tests/integration.rs` that spawns `apm work --daemon`, sends SIGINT once, and asserts the process does not exit before workers finish; then sends SIGINT again and asserts immediate exit. *(If process-spawning in integration tests is impractical given the current test harness, a unit test exercising the signal-count logic directly is acceptable.)*

### Open questions



### Amendment requests



## History

| When | From | To | By |
|------|------|----|----|
| 2026-03-31T18:35Z | — | new | claude-0331-1200-a7b9 |
| 2026-03-31T19:02Z | new | in_design | philippepascal |
| 2026-03-31T19:07Z | in_design | specd | claude-0331-1415-spec1 |
| 2026-03-31T19:45Z | specd | ready | apm |
| 2026-03-31T20:52Z | ready | in_progress | philippepascal |
| 2026-03-31T20:56Z | in_progress | implemented | claude-0331-2100-w4k9 |
| 2026-03-31T21:30Z | implemented | accepted | apm-sync |
| 2026-04-01T04:55Z | accepted | closed | apm-sync |