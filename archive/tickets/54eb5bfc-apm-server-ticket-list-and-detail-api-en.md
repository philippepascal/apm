+++
id = "54eb5bfc"
title = "apm-server: ticket list and detail API endpoints"
state = "closed"
priority = 80
effort = 3
risk = 2
author = "apm"
agent = "1041"
branch = "ticket/54eb5bfc-apm-server-ticket-list-and-detail-api-en"
created_at = "2026-03-31T06:11:28.689659Z"
updated_at = "2026-04-01T04:54:53.746794Z"
+++

## Spec

### Problem

The frontend needs read access to ticket data served over HTTP. Currently there is no API layer — only the CLI and the underlying `apm-core` library. Adding `GET /api/tickets` and `GET /api/tickets/:id` endpoints to the axum server (scaffolded in Step 1) gives the frontend a stable JSON interface to list all tickets and inspect individual ones. It also validates that `apm-core`'s synchronous git-reading functions integrate cleanly with axum's async runtime without blocking the event loop.

### Acceptance criteria

- [x] `GET /api/tickets` returns HTTP 200 with `Content-Type: application/json`
- [x] The response body is a JSON array where each element contains all frontmatter fields plus a `body` string
- [x] `GET /api/tickets/:id` with a valid ticket ID prefix returns HTTP 200 with a JSON object for that ticket
- [x] `GET /api/tickets/:id` with an unknown ID returns HTTP 404
- [x] `GET /api/tickets/:id` accepts a 4–8 hex-char prefix or a zero-padded integer (same matching rules as `apm show`)
- [x] The server does not block the tokio runtime while reading from git (blocking work is off-loaded via spawn_blocking)

### Out of scope

- Write/mutation endpoints (covered by later steps: state transition, body edit, ticket create)
- Authentication or authorization
- Pagination, sorting, or filtering of the ticket list
- The React/Vite frontend that consumes these endpoints (Step 3)
- Worker, sync, or state-transition endpoints
- The apm-server crate scaffold itself (Step 1 prerequisite)

### Approach

**Prerequisite:** Step 1 (`apm-server` crate with axum/tokio and `GET /health`) must be `implemented` before this ticket moves to `ready`.

**Files to change:**

1. `apm-server/Cargo.toml` — add deps: `serde_json`, `apm-core` (path dep)
2. `apm-server/src/main.rs` — extend `AppState` to hold `root: PathBuf` and `tickets_dir: PathBuf`; register the two new routes
3. `apm-server/src/routes/tickets.rs` (new file, or inline in main.rs if small) — implement the two handlers

**AppState:**
```rust
struct AppState {
    root: PathBuf,
    tickets_dir: PathBuf,  // relative, from apm.toml config.tickets_dir
}
```
Populated at startup by reading `apm_core::config::load(&root)`.

**Response type:**
`Frontmatter` already derives `serde::Serialize`. Define a local response struct to avoid leaking the dummy `path`:
```rust
#[derive(serde::Serialize)]
struct TicketResponse<'a> {
    #[serde(flatten)]
    frontmatter: &'a Frontmatter,
    body: &'a str,
}
```

**`GET /api/tickets` handler:**
1. Clone `state.root` and `state.tickets_dir`
2. `tokio::task::spawn_blocking(move || ticket::load_all_from_git(&root, &tickets_dir))` — keeps the async runtime unblocked
3. Map results to `TicketResponse`, serialise with `axum::Json`

**`GET /api/tickets/:id` handler:**
1. Extract `:id` path param
2. Load all tickets via `spawn_blocking` (same as above) — return 500 on git error
3. Call `ticket::resolve_id_in_slice(&tickets, &id)`:
   - `Err` whose message contains "no ticket matches" → return HTTP 404
   - `Err` whose message contains "invalid ticket ID" (format validation from `normalize_id_arg` internally) → return HTTP 400
   - `Ok(full_id)` → find the ticket in the slice and return `axum::Json(TicketResponse { ... })`

`resolve_id_in_slice` (confirmed present in `apm-core/src/ticket.rs`) handles all ID resolution: it calls `id_arg_prefixes` → `normalize_id_arg` internally for format validation, then does prefix matching with deduplication and ambiguity detection. No manual `.starts_with()` logic is needed in the handler. `normalize_id_arg` is also confirmed present but is not called directly by the handler — it is used internally by `resolve_id_in_slice`.

**Error handling:** Use `axum::response::IntoResponse`; map `anyhow::Error` to a 500 with a plain-text body. A thin `AppError` newtype wrapping `anyhow::Error` is sufficient.

**Tests:** Add a unit test in `apm-server/src/main.rs` (or a separate test module) that starts the server against the real repo root and confirms both endpoints return 200/404 as expected. Use `axum::test` or `reqwest` + `tokio::test`.

### Open questions



### Amendment requests

- [x] The Approach references `ticket::normalize_id_arg` — verify this function exists in the current apm-core API. The ID resolution logic may be named differently (e.g. `resolve_id_in_slice` or implemented inline as prefix matching). Update the handler code to use the correct function name.

## History

| When | From | To | By |
|------|------|----|----|
| 2026-03-31T06:11Z | — | new | apm |
| 2026-03-31T06:13Z | new | in_design | philippepascal |
| 2026-03-31T06:16Z | in_design | specd | claude-0330-0615-b7f2 |
| 2026-03-31T18:15Z | specd | ammend | claude-0331-1200-a7b9 |
| 2026-03-31T18:27Z | ammend | in_design | philippepascal |
| 2026-03-31T19:01Z | in_design | ammend | apm |
| 2026-03-31T19:02Z | ammend | in_design | philippepascal |
| 2026-03-31T19:07Z | in_design | specd | claude-0331-1430-f9c2 |
| 2026-03-31T19:43Z | specd | ready | apm |
| 2026-03-31T21:30Z | ready | in_progress | philippepascal |
| 2026-03-31T21:35Z | in_progress | implemented | claude-0331-2130-w4k9 |
| 2026-03-31T21:43Z | implemented | accepted | apm-sync |
| 2026-04-01T04:54Z | accepted | closed | apm-sync |