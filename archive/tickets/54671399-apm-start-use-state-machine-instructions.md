+++
id = "54671399"
title = "apm start: use state machine instructions field as worker system prompt"
state = "closed"
priority = 0
effort = 2
risk = 1
author = "philippepascal"
agent = "60240"
branch = "ticket/54671399-apm-start-use-state-machine-instructions"
created_at = "2026-03-30T22:51:08.077356Z"
updated_at = "2026-03-31T05:05:02.413593Z"
+++

## Spec

### Problem

In `apm-core/src/start.rs`, the `run()` function (used by `apm start <id> --spawn`) hardcodes `.apm/apm.worker.md` as the worker system prompt (line 233), ignoring the `instructions` field on each state in `apm.toml`.

The other two spawn paths — `run_next()` and `spawn_next_worker()` — already look up the ticket's pre-transition state in `config.workflow.states`, read its `instructions` field, and load the referenced file. `run()` has no equivalent lookup.

### Acceptance criteria

- [x] `apm start <id> --spawn` uses the `instructions` file named in the ticket's pre-transition state config, not the hardcoded `.apm/apm.worker.md`
- [x] When the state has no `instructions` field, or the referenced file cannot be read, the spawn falls back to `.apm/apm.worker.md` (and then to the inline default string)
- [x] No state name strings are hardcoded in `start.rs`
- [x] `cargo test --workspace` passes

### Out of scope

- `run_next()` and `spawn_next_worker()` already handle state-specific instructions correctly — no changes needed there
- Adding new fields to `apm.toml` or changing the config schema
- Changing how `focus_hint` is assembled or printed

### Approach

In `run()` in `apm-core/src/start.rs`, before the `if !spawn { return ... }` guard (currently around line 215), add a state-aware instructions lookup identical to the pattern already used in `run_next()` and `spawn_next_worker()`:

```rust
let worker_system = config.workflow.states.iter()
    .find(|s| s.id == old_state)
    .and_then(|sc| sc.instructions.as_ref())
    .and_then(|path| std::fs::read_to_string(root.join(path)).ok()
        .or_else(|| { eprintln!("warning: instructions file not found"); None }))
    .or_else(|| std::fs::read_to_string(root.join(".apm/apm.worker.md")).ok())
    .unwrap_or_else(|| "You are an APM worker agent.".to_string());
```

This replaces the single hardcoded `std::fs::read_to_string(root.join(".apm/apm.worker.md"))` call at line 233. The rest of the function is unchanged. No other files need editing.

### Open questions



### Amendment requests



## History

| When | From | To | By |
|------|------|----|----|
| 2026-03-30T22:51Z | — | new | philippepascal |
| 2026-03-30T22:51Z | new | in_design | philippepascal |
| 2026-03-30T22:54Z | in_design | specd | claude-0330-2251-b7f2 |
| 2026-03-30T23:58Z | specd | ready | apm |
| 2026-03-30T23:58Z | ready | in_progress | philippepascal |
| 2026-03-31T00:02Z | in_progress | implemented | claude-0330-2251-b7f2 |
| 2026-03-31T00:19Z | implemented | accepted | apm-sync |
| 2026-03-31T05:05Z | accepted | closed | apm-sync |