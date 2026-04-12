+++
id = "bcaa70bb"
title = "Trim start.rs: move resolve_caller_name to config.rs"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/bcaa70bb-trim-start-rs-move-resolve-caller-name-t"
created_at = "2026-04-12T06:04:15.262188Z"
updated_at = "2026-04-12T06:14:53.587088Z"
epic = "57bce963"
target_branch = "epic/57bce963-refactor-apm-core-module-structure"
+++

## Spec

### Problem

`start.rs` currently defines `resolve_caller_name()`, a function that resolves the acting identity for the current process by reading `APM_AGENT_NAME` → `USER` → `USERNAME` → `"apm"`. This is a pure identity/configuration concern: the same kind of look-up that `resolve_identity()` and `try_github_username()` perform, both of which already live in `config.rs`.

Having `resolve_caller_name()` in `start.rs` means callers in `apm/src/cmd/next.rs` and `apm/src/main.rs` import it as `apm_core::start::resolve_caller_name()`, coupling a CLI concern to the worker-spawning module. Moving it to `config.rs` groups all identity resolution in one place and removes that coupling.

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
| 2026-04-12T06:12Z | new | groomed | apm |
| 2026-04-12T06:14Z | groomed | in_design | philippepascal |