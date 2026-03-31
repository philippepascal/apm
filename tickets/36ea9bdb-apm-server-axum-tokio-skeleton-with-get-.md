+++
id = "36ea9bdb"
title = "apm-server: axum/tokio skeleton with GET /health endpoint"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "apm"
agent = "2356"
branch = "ticket/36ea9bdb-apm-server-axum-tokio-skeleton-with-get-"
created_at = "2026-03-31T06:05:42.967376Z"
updated_at = "2026-03-31T06:06:10.291031Z"
+++

## Spec

### Problem

The UI roadmap (initial_specs/UIdraft_spec_starter.md) requires a Rust HTTP backend. Currently no server crate exists in the workspace. This ticket adds the `apm-server` crate wired to axum + tokio, with a single `GET /health` endpoint returning `{"ok":true}`. No business logic is added yet — the goal is to confirm the crate compiles, ships, and serves HTTP traffic before later steps build on it.

### Acceptance criteria

- [ ] cargo build -p apm-server succeeds
- [ ] Running apm-server starts HTTP server on port 3000
- [ ] GET /health returns HTTP 200
- [ ] GET /health body is exactly {"ok":true}
- [ ] GET /health has Content-Type: application/json
- [ ] cargo test --workspace passes

### Out of scope

Explicit list of what this ticket does not cover.

### Approach

How the implementation will work.

### Open questions



### Amendment requests



## History

| When | From | To | By |
|------|------|----|----|
| 2026-03-31T06:05Z | — | new | apm |
| 2026-03-31T06:06Z | new | in_design | philippepascal |