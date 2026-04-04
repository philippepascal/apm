+++
id = "fa2dce31"
title = "Worker panel: show ended instead of crashed when ticket reached a worker-complete state"
state = "in_progress"
priority = 0
effort = 2
risk = 1
author = "apm"
branch = "ticket/fa2dce31-worker-panel-show-ended-instead-of-crash"
created_at = "2026-04-04T00:12:56.422580Z"
updated_at = "2026-04-04T07:30:11.839768Z"
+++

## Spec

### Problem

`determine_status()` in `apm-server/src/workers.rs` classifies a dead worker as "ended" only if the ticket's current state is in the workflow's `terminal` set (e.g., `closed`). Workers that complete successfully but leave the ticket in a non-terminal state — such as a spec-writer finishing in `specd`, or an implementer finishing in `implemented` — are reported as **"crashed"** in the worker panel.

This is misleading: the worker did its job; it is not crashed. The root cause is that `determine_status()` has no concept of "the worker's task is done" distinct from "the ticket's lifecycle is done."

The fix must be **config-based** — a new boolean field on state definitions in `workflow.toml` (e.g., `worker_end = true`) that marks states where a worker is expected to have exited. `determine_status()` should check this field alongside `terminal` to decide between "ended" and "crashed". No state names should be hardcoded.

### Acceptance criteria

- [x] A dead worker whose ticket is in a state with `worker_end = true` reports status "ended"
- [x] A dead worker whose ticket is in a state that is neither `terminal` nor `worker_end` reports status "crashed"
- [x] A dead worker whose ticket is in a `terminal` state reports status "ended" (existing behaviour preserved)
- [x] A live worker always reports status "running" regardless of ticket state (existing behaviour preserved)
- [x] `StateConfig` parses a `worker_end` boolean from TOML, defaulting to `false` when absent
- [x] The project's `.apm/workflow.toml` has `worker_end = true` on the `specd` and `implemented` states

### Out of scope

- UI presentation changes beyond the status string (colour coding, icons, etc.)
- Adding or renaming workflow states
- Changing the meaning of "running" status
- Filtering or hiding dead workers from the panel
- Any changes to how the pid file is written or read

### Approach

**1. Add `worker_end` to `StateConfig` — `apm-core/src/config.rs`**

Add a `worker_end: bool` field (with `#[serde(default)]`) to `StateConfig`, parallel to the existing `terminal: bool` field. No other changes to config loading are needed.

**2. Update `collect_workers()` — `apm-server/src/workers.rs`**

Build a single `ended_states` set that is the union of `terminal` and `worker_end` states:

```rust
let ended_states: std::collections::HashSet<&str> = config
    .workflow.states.iter()
    .filter(|s| s.terminal || s.worker_end)
    .map(|s| s.id.as_str())
    .collect();
```

Pass `&ended_states` to `determine_status()` in place of the current `&terminal_states`.

**3. Rename parameter in `determine_status()`**

Rename the third parameter from `terminal_states` to `ended_states`. Logic is unchanged — the function remains a three-branch match on alive / ended / crashed.

**4. Update `.apm/workflow.toml`**

Add `worker_end = true` to the `specd` and `implemented` state blocks. These are the two states where a worker exits cleanly (spec-writer finishes at `specd`, implementer at `implemented`).

**5. Update tests**

In `apm-server/src/workers.rs`: update `determine_status_dead_terminal_shows_ended` to cover a `worker_end`-only state (e.g. `specd`) returning `"ended"`, and confirm a state in neither set still returns `"crashed"`.

In `apm-core/src/config.rs`: add unit tests asserting `worker_end` parses as `true` when set and defaults to `false` when absent.

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-04-04T00:12Z | — | new | apm |
| 2026-04-04T06:01Z | new | groomed | apm |
| 2026-04-04T06:34Z | groomed | in_design | philippepascal |
| 2026-04-04T06:36Z | in_design | specd | claude-0403-spec-fa2d |
| 2026-04-04T07:15Z | specd | ready | apm |
| 2026-04-04T07:30Z | ready | in_progress | philippepascal |