+++
id = "22f539f2"
title = "Replace ctrlc crate with tokio::signal"
state = "groomed"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/22f539f2-replace-ctrlc-crate-with-tokio-signal"
created_at = "2026-04-19T01:23:58.653223Z"
updated_at = "2026-04-19T01:47:24.313356Z"
epic = "7bc3561c"
target_branch = "epic/7bc3561c-trim-dependency-footprint"
+++

## Spec

### Problem

`apm` pulls in the `ctrlc` crate solely to register a Ctrl-C handler at `apm/src/cmd/work.rs:28` (`ctrlc::set_handler(...)`). That is the only call-site. `tokio` is already a first-class dependency in the workspace, and `tokio::signal::ctrl_c()` returns a future that resolves on SIGINT, which covers the same need without a second signal-handling crate. Replacing the one call-site removes `ctrlc` and roughly 11 transitive dependencies and consolidates signal handling onto the async runtime we already ship.

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
| 2026-04-19T01:23Z | — | new | philippepascal |
| 2026-04-19T01:47Z | new | groomed | philippepascal |
