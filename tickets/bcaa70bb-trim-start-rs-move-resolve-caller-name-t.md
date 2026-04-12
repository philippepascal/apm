+++
id = "bcaa70bb"
title = "Trim start.rs: move resolve_caller_name to config.rs"
state = "new"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/bcaa70bb-trim-start-rs-move-resolve-caller-name-t"
created_at = "2026-04-12T06:04:15.262188Z"
updated_at = "2026-04-12T06:04:15.262188Z"
epic = "57bce963"
target_branch = "epic/57bce963-refactor-apm-core-module-structure"
+++

## Spec

### Problem

`start.rs` contains `resolve_caller_name()` which resolves the current user/agent identity. This is a configuration/identity concern, not a worker-spawning concern. It belongs in `config.rs` alongside `resolve_identity()` and `try_github_username()`.

See [REFACTOR-CORE.md](../../REFACTOR-CORE.md) section 7 for the full plan.

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
| 2026-04-12T06:04Z | — | new | philippepascal |