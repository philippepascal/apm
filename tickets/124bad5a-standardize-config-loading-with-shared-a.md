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

Config loading in `apm-server` uses 4 different patterns across files, with 19+ occurrences total:

1. `tokio::task::spawn_blocking(move || Config::load(&root))` with `.await??` (work.rs, some main.rs handlers)
2. `let Ok(config) = Config::load(root) else { return ... }` (some main.rs handlers)
3. `Config::load(root)?` direct call (workers.rs, queue.rs)
4. `spawn_blocking` with inline closure and `?` (other main.rs handlers)

The inconsistency makes error handling unpredictable. A shared utility function — e.g., `async fn load_config(root: &Path) -> Result<Config>` that always uses `spawn_blocking` (since `Config::load` does filesystem I/O) — would standardize this across all handlers and reduce boilerplate.

Similarly, the `tokio::task::spawn_blocking` wrapper pattern appears 27+ times for various blocking operations beyond config loading. A generic helper like `async fn blocking<F, T>(f: F) -> Result<T>` would reduce noise.

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
