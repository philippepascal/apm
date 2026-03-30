+++
id = "54671399"
title = "apm start: use state machine instructions field as worker system prompt"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
agent = "83841"
branch = "ticket/54671399-apm-start-use-state-machine-instructions"
created_at = "2026-03-30T22:51:08.077356Z"
updated_at = "2026-03-30T22:51:25.248845Z"
+++

## Spec

### Problem

In `apm-core/src/start.rs`, the `run()` function (used by `apm start <id> --spawn`) hardcodes `.apm/apm.worker.md` as the worker system prompt (line 233), ignoring the `instructions` field on each state in `apm.toml`.

The other two spawn paths — `run_next()` and `spawn_next_worker()` — already look up the ticket's pre-transition state in `config.workflow.states`, read its `instructions` field, and load the referenced file. `run()` has no equivalent lookup.

### Acceptance criteria

- [ ] `apm start <id> --spawn` uses the `instructions` file named in the ticket's pre-transition state config, not the hardcoded `.apm/apm.worker.md`
- [ ] When the state has no `instructions` field, or the referenced file cannot be read, the spawn falls back to `.apm/apm.worker.md` (and then to the inline default string)
- [ ] No state name strings are hardcoded in `start.rs`
- [ ] `cargo test --workspace` passes

### Out of scope

- `run_next()` and `spawn_next_worker()` already handle state-specific instructions correctly — no changes needed there
- Adding new fields to `apm.toml` or changing the config schema
- Changing how `focus_hint` is assembled or printed

### Approach

How the implementation will work.

### Open questions



### Amendment requests



## History

| When | From | To | By |
|------|------|----|----|
| 2026-03-30T22:51Z | — | new | philippepascal |
| 2026-03-30T22:51Z | new | in_design | philippepascal |