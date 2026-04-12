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

- Making `Config::load` itself async (it lives in `apm-core` and is out of scope for a server-side refactor)
- Refactoring sync helper functions (`compute_blocking_deps`, `compute_valid_transitions`) into async functions
- Changing startup/initialization code that calls `Config::load` synchronously with `.expect()` (acceptable at boot time)
- Changing test code that calls `Config::load` with `.unwrap()` (tests run synchronously)
- Adding caching or memoization of loaded config (separate concern)
- Moving `blocking` or `load_config` into `apm-core` (the helpers wrap `AppError`, which is server-specific)

### Approach

**1. Create `apm-server/src/util.rs`**

```rust
use std::path::PathBuf;
use crate::AppError;

/// Runs a blocking closure on the Tokio blocking thread pool.
/// Flattens the JoinError and the inner anyhow::Error into AppError,
/// so callers only need `.await?` instead of `.await??`.
pub async fn blocking<F, T>(f: F) -> Result<T, AppError>
where
    F: FnOnce() -> anyhow::Result<T> + Send + 'static,
    T: Send + 'static,
{
    tokio::task::spawn_blocking(f)
        .await
        .map_err(AppError::from)?
        .map_err(AppError::from)
}

/// Loads the APM config for the given repo root on the blocking thread pool.
/// Accepts PathBuf (owned) so the closure satisfies the 'static bound.
pub async fn load_config(root: PathBuf) -> Result<apm_core::config::Config, AppError> {
    blocking(move || apm_core::config::Config::load(&root)).await
}
```

**2. Add `mod util;` in `main.rs`** (alongside the existing `mod agents;`, `mod work;`, etc.)

**3. Replace patterns in `main.rs` async handlers**

For every handler that contains:
```rust
tokio::task::spawn_blocking({ let root = root.clone(); move || apm_core::config::Config::load(&root) }).await??
// or
tokio::task::spawn_blocking(move || apm_core::config::Config::load(&root_clone)).await??
```
Replace with:
```rust
util::load_config(root.clone()).await?
// (drop the intermediate root_clone variable if it was only used for this call)
```

For every other `tokio::task::spawn_blocking(move || { ... }).await??` in an async handler, replace with:
```rust
util::blocking(move || { ... }).await?
```

Handlers to touch in `main.rs` (identified by pattern, verify line numbers at time of implementation):
- `list_epics` — Config::load via spawn_blocking
- `get_epic` — Config::load via spawn_blocking
- `get_ticket` / similar — any direct Config::load in async without spawn_blocking
- All other async handlers using `spawn_blocking` for non-Config blocking work

**4. Replace patterns in `agents.rs`**

Both `get_agents_config` and `patch_agents_config` contain identical inline spawn_blocking+Config::load blocks. Replace each with `util::load_config(root.clone()).await?`. Remove the `use tokio::task::spawn_blocking` import if it becomes unused.

**5. Replace patterns in `work.rs`**

`post_work_start` (line ~117) and `get_work_dry_run` (line ~182): replace spawn_blocking+Config::load with `util::load_config(root.clone()).await?`. The second spawn_blocking in `post_work_start` (which runs `run_engine_loop`) should be converted to `util::blocking(move || { ... }).await?`.

**6. Replace pattern in `queue.rs`**

The outer `spawn_blocking` closure in `queue_handler` wraps Config::load plus ticket loading plus processing logic. Replace:
```rust
let entries = tokio::task::spawn_blocking(move || { ... }).await??;
```
with:
```rust
let entries = util::blocking(move || { ... }).await?;
```
The `Config::load` call inside the closure stays as-is (it is already in a sync context).

**7. Replace pattern in `workers.rs`**

`workers_handler` calls `spawn_blocking(move || collect_workers(&root, &tickets_dir)).await??`. Replace with `util::blocking(move || collect_workers(&root, &tickets_dir)).await?`. The `collect_workers` function itself is unchanged.

**8. Verify**

Run `cargo test -p apm-server` and confirm all tests pass. Grep for remaining `.await??` to confirm none survive in async handlers (only acceptable in test code or sync contexts, which shouldn't have `.await` at all).

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-04-12T09:03Z | — | new | philippepascal |
| 2026-04-12T09:09Z | new | groomed | apm |
| 2026-04-12T09:40Z | groomed | in_design | philippepascal |