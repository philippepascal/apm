+++
id = "22f539f2"
title = "Replace ctrlc crate with tokio::signal"
state = "in_design"
priority = 0
effort = 2
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/22f539f2-replace-ctrlc-crate-with-tokio-signal"
created_at = "2026-04-19T01:23:58.653223Z"
updated_at = "2026-04-19T01:51:45.883869Z"
epic = "7bc3561c"
target_branch = "epic/7bc3561c-trim-dependency-footprint"
+++

## Spec

### Problem

`apm` pulls in the `ctrlc` crate solely to register a Ctrl-C handler at `apm/src/cmd/work.rs:28` (`ctrlc::set_handler(...)`). That is the only call-site. `tokio` is already a first-class dependency in the workspace, and `tokio::signal::ctrl_c()` returns a future that resolves on SIGINT, which covers the same need without a second signal-handling crate. Replacing the one call-site removes `ctrlc` and roughly 11 transitive dependencies and consolidates signal handling onto the async runtime we already ship.

### Acceptance criteria

- [ ] `cargo build -p apm` succeeds with no reference to `ctrlc` in the build graph
- [ ] `ctrlc` does not appear in `apm/Cargo.toml`
- [ ] `tokio` appears as a dependency in `apm/Cargo.toml` (workspace = true)
- [ ] Pressing Ctrl-C once during a normal (non-daemon) `apm work` run causes the process to exit the dispatch loop
- [ ] Pressing Ctrl-C once during a daemon `apm work --daemon` run triggers graceful drain (same behaviour as before)
- [ ] Pressing Ctrl-C twice during a daemon run triggers immediate forced exit
- [ ] Existing unit test `sig_count_increments_correctly` passes unchanged
- [ ] `cargo test -p apm` passes without modification to any test

### Out of scope

- Converting `run()` to a fully async function or restructuring the work command's event loop
- Replacing or removing other tokio usages in the workspace
- Signal handling on Windows (ctrlc had cross-platform support; tokio::signal::ctrl_c() also supports Windows — no behaviour change expected, but Windows-specific testing is out of scope)
- Auditing other unused dependencies in `apm` (covered by sibling tickets in the epic)

### Approach

**Files to change: 2**

**`apm/Cargo.toml`**
- Remove the line `ctrlc = "3"`
- Add `tokio = { workspace = true }` (workspace root already declares `tokio = { version = "1", features = ["full"] }`, which includes the `signal` feature)

**`apm/src/cmd/work.rs`**
- Remove the `use ctrlc` import (or the extern crate reference if present)
- Replace the `ctrlc::set_handler` block (lines 28-30) with a background thread that owns a minimal current-thread tokio runtime and loops on `tokio::signal::ctrl_c().await`, incrementing `sig_count` on each resolution:

```rust
let sig_count_clone = Arc::clone(&sig_count);
std::thread::spawn(move || {
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("tokio runtime for signal handling");
    rt.block_on(async move {
        loop {
            tokio::signal::ctrl_c()
                .await
                .expect("failed to listen for ctrl-c");
            sig_count_clone.fetch_add(1, Ordering::Relaxed);
        }
    });
});
```

This keeps `run()` synchronous — no tokio::main, no restructuring of the blocking poll loop. The background thread is the only async boundary introduced.

The rest of `work.rs` (the 500 ms polling loop, daemon-vs-normal logic, forced-exit path) is unchanged.

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-04-19T01:23Z | — | new | philippepascal |
| 2026-04-19T01:47Z | new | groomed | philippepascal |
| 2026-04-19T01:49Z | groomed | in_design | philippepascal |