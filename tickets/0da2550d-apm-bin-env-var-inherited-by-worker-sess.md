+++
id = "0da2550d"
title = "APM_BIN env var inherited by worker sessions may point to stale Homebrew binary"
state = "in_design"
priority = 0
effort = 3
risk = 2
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/0da2550d-apm-bin-env-var-inherited-by-worker-sess"
created_at = "2026-05-04T05:00:29.668572Z"
updated_at = "2026-05-04T05:17:17.626140Z"
+++

## Spec

### Problem

When a worker session is launched via `apm start`, both wrapper paths compute `APM_BIN` using `std::env::current_exe()`. In production (e.g. a Homebrew install), the running process may be `apm-server`, so `current_exe()` resolves to something like `/opt/homebrew/Cellar/apm/0.1.18/bin/apm-server`. That path is set as `APM_BIN` and propagated into the worker environment.

This causes two failures in practice. First, the worker agent itself (Claude Code) attempts to invoke subcommands such as `apm spec` and `apm state` via `APM_BIN`; pointing at `apm-server` means those calls fail immediately because `apm-server` does not expose the CLI surface. Second, any `cargo test --workspace` run inside the worker session also inherits `APM_BIN`. The `find_apm_bin()` test helper in `apm-core/src/start.rs` honours `APM_BIN` first (if the path exists on disk), so tests resolve the stale or wrong binary instead of the freshly compiled `target/{profile}/apm`, causing mock-wrapper tests that invoke `apm spec` or `apm state` to fail.

The desired behaviour is that `APM_BIN` passed to workers always points to the `apm` CLI binary (the one that exposes all subcommands), and that developers have a documented escape hatch for the residual case where even the CLI binary is an older installed version that predates a feature under active development.

### Acceptance criteria

- [ ] When the wrapper runs from an `apm-server` executable, `APM_BIN` set in the worker environment points to the sibling `apm` CLI binary in the same directory, not to `apm-server`
- [ ] When the wrapper runs from an `apm` CLI executable, `APM_BIN` continues to point to that `apm` binary (no regression)
- [ ] When no sibling `apm` binary exists next to `current_exe()`, `APM_BIN` falls back to `current_exe()` itself (graceful degradation)
- [ ] The existing `claude_wrapper_sets_apm_env_vars` test (`apm-core/src/start.rs`) passes without modification
- [ ] A new assertion in `claude_wrapper_sets_apm_env_vars` (or a companion test) verifies that the resolved `APM_BIN` path's file stem is `apm`, not `apm-server`
- [ ] `CONTRIBUTING.md` documents that running `cargo test --workspace` inside a worker session should be done as `env -u APM_BIN cargo test --workspace` when `APM_BIN` may point to an installed binary that predates the feature under test

### Out of scope

- Handling the case where both the sibling `apm` CLI binary and a freshly built dev binary coexist and the sibling is stale — the `env -u APM_BIN` workaround documented in `CONTRIBUTING.md` covers this
- Changes to `find_apm_bin()` in `apm-core/src/start.rs`
- Changes to mock-script shell templates that reference `${APM_BIN:?}`
- Changes to the path-guard logic in `apm/src/cmd/path_guard.rs` that reads `APM_BIN`
- Changing the wrapper environment contract version or the documented list of env vars in `apm-core/src/agents.rs`

### Approach

#### Shared helper

Add a `resolve_apm_cli_bin() -> String` function to `apm-core/src/wrapper/mod.rs`. The function:

1. Calls `std::env::current_exe()` and canonicalizes the result.
2. Takes the parent directory of the canonicalized path.
3. Constructs a candidate path by joining that directory with `"apm"`.
4. If the candidate exists on disk AND differs from `current_exe()`, returns the candidate as a `String`.
5. Otherwise returns `current_exe()` as a `String` (preserving today's fallback behaviour).
6. If `current_exe()` itself fails, returns an empty `String` (same as today).

```rust
// apm-core/src/wrapper/mod.rs
pub(crate) fn resolve_apm_cli_bin() -> String {
    std::env::current_exe()
        .and_then(|p| p.canonicalize())
        .ok()
        .map(|exe| {
            let candidate = exe
                .parent()
                .map(|dir| dir.join("apm"))
                .filter(|p| p.is_file() && *p != exe);
            candidate.unwrap_or(exe)
        })
        .map(|p| p.to_string_lossy().into_owned())
        .unwrap_or_default()
}
```

#### claude.rs call site

In `apm-core/src/wrapper/builtin/claude.rs`, replace the `current_exe()` block at the top of `spawn()`:

```rust
// before
let apm_bin = std::env::current_exe()
    .and_then(|p| p.canonicalize())
    .map(|p| p.to_string_lossy().into_owned())
    .unwrap_or_default();

// after
let apm_bin = super::super::resolve_apm_cli_bin();
```

#### custom.rs call site

In `apm-core/src/wrapper/custom.rs`, replace lines 118–121:

```rust
// before
let apm_bin = std::env::current_exe()
    .and_then(|p| p.canonicalize())
    .map(|p| p.to_string_lossy().into_owned())
    .unwrap_or_default();

// after
let apm_bin = super::resolve_apm_cli_bin();
```

#### builtin/mod.rs call site

In `apm-core/src/wrapper/builtin/mod.rs`, replace lines 128–133 (the `unwrap_or_else` branch that calls `current_exe()`):

```rust
// before
.unwrap_or_else(|| {
    std::env::current_exe()
        .and_then(|p| p.canonicalize())
        .map(|p| p.to_string_lossy().into_owned())
        .unwrap_or_default()
})

// after
.unwrap_or_else(|| super::resolve_apm_cli_bin())
```

The `ctx.options["apm_bin"]` override path (used by tests) is preserved unchanged.

#### Test coverage

In `apm-core/src/start.rs`, extend or add an assertion to the `claude_wrapper_sets_apm_env_vars` test verifying that the `APM_BIN` value's file stem is `"apm"` and not `"apm-server"`:

```rust
if let Some(line) = env_content.lines().find(|l| l.starts_with("APM_BIN=")) {
    let path = std::path::Path::new(line.trim_start_matches("APM_BIN="));
    assert_eq!(
        path.file_stem().and_then(|s| s.to_str()),
        Some("apm"),
        "APM_BIN must point to the apm CLI binary, not apm-server: {path:?}"
    );
}
```

#### CONTRIBUTING.md note

Append a "Testing inside a worker session" paragraph to `CONTRIBUTING.md` stating:

> When running `cargo test --workspace` from inside an APM worker session, the environment inherits `APM_BIN` which may point to the installed system binary rather than the freshly compiled one. If you are testing a feature that does not yet exist in the installed version, prefix the command with `env -u APM_BIN`:
>
> ```sh
> env -u APM_BIN cargo test --workspace
> ```
>
> This lets the test harness derive the correct binary from the build output directory.

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-05-04T05:00Z | — | new | claude-0504-0441-a918|philippepascal |
| 2026-05-04T05:07Z | new | groomed | philippepascal |
| 2026-05-04T05:07Z | groomed | in_design | philippepascal |
| 2026-05-04T05:13Z | in_design | specd | claude-0504-0507-6e98 |
| 2026-05-04T05:16Z | specd | ammend | philippepascal |
| 2026-05-04T05:17Z | ammend | in_design | philippepascal |
