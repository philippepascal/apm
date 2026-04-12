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

- [ ] `apm-server/src/util.rs` exists and is declared as `mod util` in `main.rs`
- [ ] `util::blocking` accepts any `FnOnce() -> anyhow::Result<T>` that is `Send + 'static` and returns `Result<T, AppError>`
- [ ] `util::blocking` does not require callers to double-`?` the result (`.await?` is sufficient)
- [ ] `util::load_config` accepts a `PathBuf`, calls `Config::load` via `blocking`, and returns `Result<apm_core::config::Config, AppError>`
- [ ] All async handlers in `main.rs` that previously called `Config::load` via inline `spawn_blocking` now use `util::load_config`
- [ ] All async handlers in `agents.rs` (`get_agents_config`, `patch_agents_config`) use `util::load_config`
- [ ] All async handlers in `work.rs` (`post_work_start`, `get_work_dry_run`) use `util::load_config`
- [ ] `queue.rs` `queue_handler` uses `util::blocking` for its outer blocking closure (direct `Config::load` inside the closure is acceptable since it is already in a sync context)
- [ ] `workers.rs` `workers_handler` uses `util::blocking` instead of bare `spawn_blocking`
- [ ] No remaining `.await??` patterns appear in async handlers (all have been replaced by `.await?`)
- [ ] Synchronous helper functions (`compute_blocking_deps`, `compute_valid_transitions`, `collect_workers`) are unchanged — they call `Config::load` directly as they run in a sync context
- [ ] Startup initialization code (`build_app`, `setup_cors`) and test code are unchanged
- [ ] `cargo test` passes with no regressions

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