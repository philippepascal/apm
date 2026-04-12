+++
id = "124bad5a"
title = "Standardize config loading with shared async utility"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/124bad5a-standardize-config-loading-with-shared-a"
created_at = "2026-04-12T09:03:01.972783Z"
updated_at = "2026-04-12T09:40:12.987669Z"
epic = "1e706443"
target_branch = "epic/1e706443-refactor-apm-server-code-organization"
+++

## Spec

### Problem

Config loading in `apm-server` is inconsistent across async handlers. The same operation — wrapping `Config::load` (a synchronous, filesystem-heavy function) in `tokio::task::spawn_blocking` — is written four different ways across `main.rs`, `agents.rs`, `work.rs`, and `queue.rs`. Some handlers use a named `root_clone` variable, some use a block-capture closure, and at least one handler calls `Config::load` directly in an async context without `spawn_blocking` at all.

The double-`?` await idiom (`.await??`) also appears 37+ times for all kinds of blocking work, not just config loading. This noise obscures what the code is actually doing and makes it easy to accidentally call a blocking function from the async executor.

A shared `util.rs` module with two helpers removes both problems:
1. `async fn load_config(root: PathBuf) -> Result<Config, AppError>` — a single, correctly-wrapped call site for config loading in async handlers.
2. `async fn blocking<F, T>(f: F) -> Result<T, AppError>` — a generic wrapper that absorbs `JoinError`, flattens the double-`?`, and gives every `spawn_blocking` call a consistent shape.

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
| 2026-04-12T09:03Z | — | new | philippepascal |
| 2026-04-12T09:09Z | new | groomed | apm |
| 2026-04-12T09:40Z | groomed | in_design | philippepascal |