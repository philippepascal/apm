+++
id = "bcaa70bb"
title = "Trim start.rs: move resolve_caller_name to config.rs"
state = "specd"
priority = 0
effort = 2
risk = 1
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/bcaa70bb-trim-start-rs-move-resolve-caller-name-t"
created_at = "2026-04-12T06:04:15.262188Z"
updated_at = "2026-04-12T06:17:00.816854Z"
epic = "57bce963"
target_branch = "epic/57bce963-refactor-apm-core-module-structure"
+++

## Spec

### Problem

`start.rs` currently defines `resolve_caller_name()`, a function that resolves the acting identity for the current process by reading `APM_AGENT_NAME` → `USER` → `USERNAME` → `"apm"`. This is a pure identity/configuration concern: the same kind of look-up that `resolve_identity()` and `try_github_username()` perform, both of which already live in `config.rs`.

Having `resolve_caller_name()` in `start.rs` means callers in `apm/src/cmd/next.rs` and `apm/src/main.rs` import it as `apm_core::start::resolve_caller_name()`, coupling a CLI concern to the worker-spawning module. Moving it to `config.rs` groups all identity resolution in one place and removes that coupling.

### Acceptance criteria

- [ ] `apm_core::config::resolve_caller_name()` exists and is publicly exported from `config.rs`
- [ ] `apm_core::start::resolve_caller_name()` no longer exists (removed from `start.rs`)
- [ ] `apm/src/cmd/next.rs` calls `apm_core::config::resolve_caller_name()` instead of `apm_core::start::resolve_caller_name()`
- [ ] `apm/src/main.rs` calls `apm_core::config::resolve_caller_name()` instead of `apm_core::start::resolve_caller_name()`
- [ ] Internal callers in `start.rs` use `crate::config::resolve_caller_name()` instead of the local function
- [ ] The three unit tests for `resolve_caller_name()` (`prefers_apm_agent_name`, `falls_back_to_user`, `defaults_to_apm`) are present in `config.rs` and pass
- [ ] `cargo test` passes across the full workspace with no compilation errors

### Out of scope

- Merging `resolve_caller_name()` with `resolve_identity()` — they serve distinct purposes and remain separate functions
- Changing the resolution logic (env var order, fallback value) — behaviour is preserved exactly
- Moving any other functions out of `start.rs` — covered by sibling tickets in the epic
- Updating integration tests beyond fixing import paths

### Approach

**Files that change:**

- `apm-core/src/start.rs` — source of the move
- `apm-core/src/config.rs` — destination of the move
- `apm/src/cmd/next.rs` — external caller, update import path
- `apm/src/main.rs` — external caller, update import path

**Steps:**

1. **`apm-core/src/config.rs`** — Append `resolve_caller_name()` near the end of the file, after `try_github_username()`. Copy the full doc comment and function body verbatim from `start.rs` lines 62–76. Add three unit tests (`prefers_apm_agent_name`, `falls_back_to_user`, `defaults_to_apm`) to the existing `#[cfg(test)]` block, copied from `start.rs` tests.

2. **`apm-core/src/start.rs`** — Delete the `resolve_caller_name()` function (lines 62–76). Update the two call sites:
   - Line ~400 in `run_next()`: `resolve_caller_name()` → `crate::config::resolve_caller_name()`
   - Line ~571 in `spawn_next_worker()`: same substitution
   - In the test module `use super::{resolve_caller_name, ...}` — remove `resolve_caller_name` from the import; update the three test functions to call `crate::config::resolve_caller_name()` or remove them entirely (they now live in `config.rs`).

3. **`apm/src/cmd/next.rs` line ~19** — Change `apm_core::start::resolve_caller_name()` to `apm_core::config::resolve_caller_name()`.

4. **`apm/src/main.rs` line ~784** — Change `apm_core::start::resolve_caller_name()` to `apm_core::config::resolve_caller_name()`.

5. Run `cargo test --workspace` to confirm no regressions.

**Constraints:** No behaviour change — function signature, doc comment, and resolution order must be preserved exactly.

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-04-12T06:04Z | — | new | philippepascal |
| 2026-04-12T06:12Z | new | groomed | apm |
| 2026-04-12T06:14Z | groomed | in_design | philippepascal |
| 2026-04-12T06:17Z | in_design | specd | claude-0412-0614-1b78 |
