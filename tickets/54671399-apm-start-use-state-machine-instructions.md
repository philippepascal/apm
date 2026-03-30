+++
id = "54671399"
title = "apm start: use state machine instructions field as worker system prompt"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
agent = "philippepascal"
branch = "ticket/54671399-apm-start-use-state-machine-instructions"
created_at = "2026-03-30T22:51:08.077356Z"
updated_at = "2026-03-30T22:51:25.248845Z"
+++

## Spec

### Problem

Each state in `apm.toml` has an `instructions` field naming the markdown file that should be used as the system prompt when an agent works that state. For example:

```toml
[[workflow.states]]
id = "new"
instructions = "apm.spec-writer.md"

[[workflow.states]]
id = "ready"
instructions = "apm.worker.md"
```

These files live in `.apm/` (e.g. `.apm/apm.spec-writer.md`, `.apm/apm.worker.md`).

`apm start` currently ignores the `instructions` field entirely. It hardcodes `.apm/apm.worker.md` as the system prompt for every spawned worker subprocess, regardless of which state the ticket is in.

The fix is mechanical: in `apm-core/src/start.rs`, at each of the three spawn sites (`run()`, `run_next()`, `spawn_next_worker()`), look up the ticket's pre-transition state in `config.workflow.states`, read the `instructions` field, and load `.apm/<instructions>` as the system prompt. Fall back to `.apm/apm.worker.md` if the field is absent or the file cannot be read. No state names should be hardcoded in `start.rs`.

### Acceptance criteria


### Out of scope

Explicit list of what this ticket does not cover.

### Approach

How the implementation will work.

### Open questions



### Amendment requests



## History

| When | From | To | By |
|------|------|----|----|
| 2026-03-30T22:51Z | — | new | philippepascal |
| 2026-03-30T22:51Z | new | in_design | philippepascal |
