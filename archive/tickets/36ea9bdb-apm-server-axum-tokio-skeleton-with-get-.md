+++
id = "36ea9bdb"
title = "apm-server: axum/tokio skeleton with GET /health endpoint"
state = "closed"
priority = 100
effort = 2
risk = 1
author = "apm"
agent = "79925"
branch = "ticket/36ea9bdb-apm-server-axum-tokio-skeleton-with-get-"
created_at = "2026-03-31T06:05:42.967376Z"
updated_at = "2026-04-01T04:54:44.734484Z"
+++

## Spec

### Problem

The UI roadmap (initial_specs/UIdraft_spec_starter.md) requires a Rust HTTP backend. Currently no server crate exists in the workspace. This ticket adds the `apm-server` crate wired to axum + tokio, with a single `GET /health` endpoint returning `{"ok":true}`. No business logic is added yet — the goal is to confirm the crate compiles, ships, and serves HTTP traffic before later steps build on it.

### Acceptance criteria

- [x] cargo build -p apm-server succeeds
- [x] Running apm-server starts HTTP server on port 3000
- [x] GET /health returns HTTP 200
- [x] GET /health body is exactly {"ok":true}
- [x] GET /health has Content-Type: application/json
- [x] cargo test --workspace passes

### Out of scope

- Any business logic or apm-core integration (later tickets)
- The apm serve subcommand in the apm CLI binary
- Authentication, TLS, or CORS configuration
- Configurable host/port via flags (hard-code 0.0.0.0:3000)
- Any API endpoints beyond GET /health

### Approach

1. Add apm-server to workspace
   - Append "apm-server" to the members list in the root Cargo.toml
   - Add axum and tokio to [workspace.dependencies]: axum = { version = "0.7", features = [] }, tokio = { version = "1", features = ["full"] }, tower-http = { version = "0.5" } (optional, for future middleware)

2. Create apm-server/Cargo.toml
   - [package] name = "apm-server", edition = "2021"
   - [[bin]] name = "apm-server", path = "src/main.rs"
   - [dependencies]: axum from workspace, tokio from workspace, serde_json from workspace

3. Create apm-server/src/main.rs
   - Declare a tokio::main async fn main()
   - Build an axum Router with one route: .route("/health", get(health_handler))
   - health_handler returns Json(serde_json::json\!({"ok": true}))
   - Bind to "0.0.0.0:3000" with tokio::net::TcpListener::bind, then axum::serve
   - Print "Listening on 0.0.0.0:3000" to stdout before serving

4. No tests needed for this ticket beyond confirming cargo test --workspace passes (the crate has no logic to unit-test).

File changes:
- Cargo.toml (root): add "apm-server" to members and axum/tokio to workspace.dependencies
- apm-server/Cargo.toml: new file
- apm-server/src/main.rs: new file (~25 lines)

### Open questions



### Amendment requests



## History

| When | From | To | By |
|------|------|----|----|
| 2026-03-31T06:05Z | — | new | apm |
| 2026-03-31T06:06Z | new | in_design | philippepascal |
| 2026-03-31T06:09Z | in_design | specd | claude-0330-0600-b7f2 |
| 2026-03-31T19:43Z | specd | ready | apm |
| 2026-03-31T19:45Z | ready | in_progress | philippepascal |
| 2026-03-31T19:49Z | in_progress | implemented | claude-0331-1945-x7k2 |
| 2026-03-31T20:22Z | implemented | accepted | apm-sync |
| 2026-04-01T04:54Z | accepted | closed | apm-sync |