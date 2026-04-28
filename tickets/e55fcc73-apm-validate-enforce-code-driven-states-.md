+++
id = "e55fcc73"
title = "apm validate: enforce code-driven states are declared in workflow.toml"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/e55fcc73-apm-validate-enforce-code-driven-states-"
created_at = "2026-04-28T22:42:06.291026Z"
updated_at = "2026-04-28T22:48:19.267181Z"
depends_on = ["50649e84"]
+++

## Spec

### Problem

`apm-core/src/state.rs` hard-codes `state = "merge_failed"` when an attempted merge fails during the `in_progress → implemented` transition (lines 161–184). This write bypasses the state machine entirely — `workflow.toml` is never consulted. As a result a ticket can land in a state that the project's `workflow.toml` does not declare: no transitions are defined for it, `apm state` cannot move the ticket out, and it is visible only via `apm list`.

Ticket 63f5e6d2 hit this exactly: it ended up in `merge_failed` on a project initialised before commit `a7bce26b` (the commit that added `merge_failed` to the default template). The only escape was a manual `workflow.toml` edit.

The fix has two parts:

**1. `apm validate` enforces that every state the code can write is declared in `workflow.toml`.**
A small registry — `SYSTEM_STATES` in `apm-core/src/state.rs` — lists every state value the code may write directly (currently just `"merge_failed"`). `apm validate` walks this list against the loaded config; any registered state absent from `workflow.toml` is reported as a config error. Because this is a config-level check it runs even under `--config-only`.

**2. `apm validate --fix` ports missing states from the embedded default template.**
For each missing state, the fix locates the corresponding `[[workflow.states]]` block in the default `workflow.toml` (shipped inside the binary via `include_str!`) and appends it verbatim to the project's `.apm/workflow.toml`. The operation is idempotent. If the default template itself has no block for a registered state (i.e. `SYSTEM_STATES` and the template have drifted), the fix reports an error and exits non-zero rather than silently skipping.

The existing hash-trip on config-file changes surfaces this check automatically on the next mutating command. Tying re-validation to the binary version (so a binary upgrade triggers it) is a natural follow-up but is not part of this ticket.

### Acceptance criteria

Checkboxes; each one independently testable.

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
| 2026-04-28T22:42Z | — | new | philippepascal |
| 2026-04-28T22:44Z | new | groomed | philippepascal |
| 2026-04-28T22:48Z | groomed | in_design | philippepascal |