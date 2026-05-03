+++
id = "b302b360"
title = "Add brief delay between fetch and merge in apm start to reduce fetch-race window"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/b302b360-add-brief-delay-between-fetch-and-merge-"
created_at = "2026-05-03T08:07:25.634157Z"
updated_at = "2026-05-03T19:02:14.377866Z"
+++

## Spec

### Problem

When apm start fetches origin/main before merging into the ticket branch, a narrow race window exists: if a previous ticket was merged to origin within ~30 seconds before apm start fires, the fetch may retrieve a stale snapshot and the merge silently operates on old content. Observed in 6095305a (f06272f1 merged at 12:21:51, apm start at 12:22:14 — 23-second window). The stale merge succeeded, the worker built on the old start.rs base, and the subsequent apm state implemented merge conflicted with f06272f1's changes. A short deterministic sleep (e.g. 1-2 seconds) between fetch and merge gives the remote propagation window time to settle, reducing the probability of this race without requiring retries or polling.

### Acceptance criteria

- [ ] When `apm start` runs with `--aggressive`, a sleep of at least 1 second occurs after the fetch block completes and before `merge_ref` is called
- [ ] When `apm start` runs without `--aggressive` (no fetch), no sleep is introduced
- [ ] The sleep duration is expressed as a named constant (not an inline magic number) in `start.rs`
- [ ] `apm start --aggressive` still succeeds end-to-end after the delay is added

### Out of scope

- Retry logic or polling to verify the fetched ref is current\n- Making the sleep duration configurable at runtime or via apm config\n- Fixing the root cause of remote-propagation latency\n- Non-aggressive mode (no fetch runs, so no delay is needed)

### Approach

One file changes: `apm-core/src/start.rs`.

Add a named constant near the top of the file (with the other constants or just before the `start()` function):

```rust
/// Delay inserted between `git fetch` and `git merge` in aggressive mode to let
/// remote-propagation settle and reduce the fetch-race window.
const POST_FETCH_SETTLE_MS: u64 = 1_000;
```

Inside the `if aggressive { ... }` block (lines 307-314), append the sleep as the last statement so it only fires when a fetch actually ran:

```rust
if aggressive {
    if let Err(e) = git::fetch_branch(root, &branch) {
        warnings.push(format!("warning: fetch failed: {e:#}"));
    }
    if let Err(e) = git::fetch_branch(root, default_branch) {
        warnings.push(format!("warning: fetch {} failed: {e:#}", default_branch));
    }
    std::thread::sleep(std::time::Duration::from_millis(POST_FETCH_SETTLE_MS));
}
```

No other files need to change. `std::thread` and `std::time` are in the standard library and can be referenced with their full paths inline — no new `use` imports required.

Order of steps:
1. Add the `POST_FETCH_SETTLE_MS` constant
2. Insert the `sleep` call as the last line inside the `if aggressive` block
3. Run `cargo test -p apm-core` to confirm no regressions

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-05-03T08:07Z | — | new | philippepascal |
| 2026-05-03T19:01Z | new | groomed | philippepascal |
| 2026-05-03T19:02Z | groomed | in_design | philippepascal |