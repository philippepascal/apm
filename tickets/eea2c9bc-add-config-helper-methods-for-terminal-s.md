+++
id = "eea2c9bc"
title = "Add Config helper methods for terminal states and section lookup"
state = "groomed"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
branch = "ticket/eea2c9bc-add-config-helper-methods-for-terminal-s"
created_at = "2026-04-07T22:22:22.370019Z"
updated_at = "2026-04-07T22:43:52.449166Z"
epic = "ac0fb648"
target_branch = "epic/ac0fb648-code-separation-and-reuse-cleanup"
+++

## Spec

### Problem

The same terminal-state lookup pattern is repeated in at least 6 files across the codebase: `archive.rs`, `clean.rs`, `sync.rs`, `verify.rs`, `validate.rs`, and `apm/src/cmd/workers.rs`. Each repeats a variant of:

```rust
let terminal_ids: Vec<&str> = config.workflow.states.iter()
    .filter(|s| s.terminal)
    .map(|s| s.id.as_str())
    .collect();
```

Similarly, section name lookups by name are repeated in `spec.rs` and `ticket.rs` without a centralized helper on `Config`.

This creates maintenance risk: if the filtering logic needs to change (e.g., adding a new state property), every call site must be updated independently.

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
| 2026-04-07T22:22Z | — | new | philippepascal |
| 2026-04-07T22:43Z | new | groomed | apm |
