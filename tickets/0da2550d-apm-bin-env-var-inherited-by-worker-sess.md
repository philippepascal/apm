+++
id = "0da2550d"
title = "APM_BIN env var inherited by worker sessions may point to stale Homebrew binary"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/0da2550d-apm-bin-env-var-inherited-by-worker-sess"
created_at = "2026-05-04T05:00:29.668572Z"
updated_at = "2026-05-04T05:07:10.085185Z"
+++

## Spec

### Problem

When a worker session is launched via `apm start`, both wrapper paths compute `APM_BIN` using `std::env::current_exe()`. In production (e.g. a Homebrew install), the running process may be `apm-server`, so `current_exe()` resolves to something like `/opt/homebrew/Cellar/apm/0.1.18/bin/apm-server`. That path is set as `APM_BIN` and propagated into the worker environment.

This causes two failures in practice. First, the worker agent itself (Claude Code) attempts to invoke subcommands such as `apm spec` and `apm state` via `APM_BIN`; pointing at `apm-server` means those calls fail immediately because `apm-server` does not expose the CLI surface. Second, any `cargo test --workspace` run inside the worker session also inherits `APM_BIN`. The `find_apm_bin()` test helper in `apm-core/src/start.rs` honours `APM_BIN` first (if the path exists on disk), so tests resolve the stale or wrong binary instead of the freshly compiled `target/{profile}/apm`, causing mock-wrapper tests that invoke `apm spec` or `apm state` to fail.

The desired behaviour is that `APM_BIN` passed to workers always points to the `apm` CLI binary (the one that exposes all subcommands), and that developers have a documented escape hatch for the residual case where even the CLI binary is an older installed version that predates a feature under active development.

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
| 2026-05-04T05:00Z | — | new | claude-0504-0441-a918|philippepascal |
| 2026-05-04T05:07Z | new | groomed | philippepascal |
| 2026-05-04T05:07Z | groomed | in_design | philippepascal |