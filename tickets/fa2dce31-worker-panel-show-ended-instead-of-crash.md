+++
id = "fa2dce31"
title = "Worker panel: show ended instead of crashed when ticket reached a worker-complete state"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "apm"
branch = "ticket/fa2dce31-worker-panel-show-ended-instead-of-crash"
created_at = "2026-04-04T00:12:56.422580Z"
updated_at = "2026-04-04T06:34:36.838860Z"
+++

## Spec

### Problem

`determine_status()` in `apm-server/src/workers.rs` classifies a dead worker as "ended" only if the ticket's current state is in the workflow's `terminal` set (e.g., `closed`). Workers that complete successfully but leave the ticket in a non-terminal state — such as a spec-writer finishing in `specd`, or an implementer finishing in `implemented` — are reported as **"crashed"** in the worker panel.

This is misleading: the worker did its job; it is not crashed. The root cause is that `determine_status()` has no concept of "the worker's task is done" distinct from "the ticket's lifecycle is done."

The fix must be **config-based** — a new boolean field on state definitions in `workflow.toml` (e.g., `worker_end = true`) that marks states where a worker is expected to have exited. `determine_status()` should check this field alongside `terminal` to decide between "ended" and "crashed". No state names should be hardcoded.

### Acceptance criteria

- [ ] A dead worker whose ticket is in a state with `worker_end = true` reports status "ended"
- [ ] A dead worker whose ticket is in a state that is neither `terminal` nor `worker_end` reports status "crashed"
- [ ] A dead worker whose ticket is in a `terminal` state reports status "ended" (existing behaviour preserved)
- [ ] A live worker always reports status "running" regardless of ticket state (existing behaviour preserved)
- [ ] `StateConfig` parses a `worker_end` boolean from TOML, defaulting to `false` when absent
- [ ] The project's `.apm/workflow.toml` has `worker_end = true` on the `specd` and `implemented` states

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
| 2026-04-04T00:12Z | — | new | apm |
| 2026-04-04T06:01Z | new | groomed | apm |
| 2026-04-04T06:34Z | groomed | in_design | philippepascal |