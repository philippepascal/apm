+++
id = "90ebf40b"
title = "apm-server: expose author field in ticket API responses"
state = "closed"
priority = 0
effort = 3
risk = 2
author = "apm"
branch = "ticket/90ebf40b-apm-server-expose-author-field-in-ticket"
created_at = "2026-04-02T20:54:08.576527Z"
updated_at = "2026-04-04T06:02:00.166263Z"
epic = "8db73240"
target_branch = "epic/8db73240-user-mgmt"
depends_on = ["610be42e"]
+++

## Spec

### Problem

The `author` field exists in `Frontmatter` but is declared `#[serde(skip_serializing_if = "Option::is_none")]`. Tickets that lack an `author` value (e.g. created before ticket #610be42e lands, or test fixtures) produce JSON responses with no `author` key at all. The UI cannot reliably read, display, or filter by ticket ownership when the field may be absent.

Two additional gaps compound this: (1) `GET /api/tickets` has no `author` query parameter, so the UI must download and filter all tickets client-side; (2) there is no `GET /api/me` endpoint, so the supervisor board cannot know whose tickets to show by default.

Together these gaps block the supervisor-board author filter and the per-author default view described in DESIGN-users.md points 1 and 8.

### Acceptance criteria

- [x] `GET /api/tickets` response objects always include an `author` field (never omitted), using `"unassigned"` when the frontmatter has no author set
- [x] `GET /api/tickets/:id` response object always includes an `author` field, using `"unassigned"` when the frontmatter has no author set
- [x] `GET /api/tickets?author=<username>` returns only tickets where `author` equals `<username>`
- [x] `GET /api/tickets?author=unassigned` returns only tickets whose frontmatter author is absent or equal to `"unassigned"`
- [x] `GET /api/me` returns `{"username": "<value>"}` where `<value>` is the `username` from `.apm/local.toml` when that file exists and contains a non-empty key
- [x] `GET /api/me` returns `{"username": "unassigned"}` when `.apm/local.toml` is absent or contains no `username` key
- [x] Existing tests continue to pass after the change

### Out of scope

- WebAuthn / OTP authentication (DESIGN-users.md point 5)
- `GET /api/me` for WebAuthn-authenticated sessions (returns local identity only; session-cookie resolution is a later auth ticket)
- `apm list --mine` / `apm list --author` CLI flags (ticket #610be42e out of scope, separate ticket)
- Supervisor board UI changes — filter bar, default author filter, card display (separate UI ticket)
- Collaborators list validation or sync from GitHub (DESIGN-users.md point 4)
- `apm init` prompting for username
- Removing or migrating existing `agent` field values in frontmatter (ticket #610be42e)

### Approach

**apm-core/src/config.rs**

0. **Add `ServerConfig` struct and field** — add a new config struct and wire it into `Config`:
   ```rust
   #[derive(Debug, Clone, Deserialize)]
   pub struct ServerConfig {
       #[serde(default = "default_server_origin")]
       pub origin: String,
   }

   fn default_server_origin() -> String {
       "http://localhost:3000".to_string()
   }

   impl Default for ServerConfig {
       fn default() -> Self {
           Self { origin: default_server_origin() }
       }
   }
   ```
   Add `pub server: ServerConfig` to the `Config` struct with `#[serde(default)]` so existing config files without a `[server]` section continue to parse. Add a test (`server_config_defaults`) that verifies the default origin is `"http://localhost:3000"` when the section is absent, and a test (`server_config_custom_origin`) that verifies a custom origin parses correctly.

**apm-server/src/main.rs**

1. **Always-present author in responses** — in both `list_tickets` and `get_ticket` handlers, normalise the author before building the response struct:
   ```rust
   if t.frontmatter.author.is_none() {
       t.frontmatter.author = Some("unassigned".to_string());
   }
   ```
   This guarantees `skip_serializing_if = "Option::is_none"` will never fire for the `author` field in API responses without touching the TOML serialisation path.

2. **Author filter on `GET /api/tickets`** — add `author: Option<String>` to `ListTicketsQuery`. After the existing `include_closed` filter, add:
   ```rust
   if let Some(ref author) = params.author {
       tickets.retain(|t| {
           let a = t.frontmatter.author.as_deref().unwrap_or("unassigned");
           a == author.as_str()
       });
   }
   ```

3. **`GET /api/me` endpoint** — add a handler that reads the local username from `.apm/local.toml` via `apm_core::identity::read_local_username` (from ticket #610be42e) and returns JSON with the username (or "unassigned" as fallback). Register as `.route("/api/me", get(me_handler))`.

   If `apm_core::identity` is not yet available when this is implemented, stub the `/api/me` read with a direct inline read of `.apm/local.toml` using `toml` crate. Do not block on it.

**Tests to add (in apm-server/src/main.rs tests)**

- `list_tickets_author_field_always_present` — ticket with `author: None`; assert response JSON includes `"author": "unassigned"`
- `list_tickets_author_filter` — two tickets with different authors; `?author=alice` returns only Alice's
- `list_tickets_author_unassigned_filter` — ticket with `author: None`; `?author=unassigned` returns it
- `get_ticket_author_field_always_present` — ticket with `author: None`; assert detail response includes `"author": "unassigned"`
- `me_handler_returns_unassigned_when_no_local_toml` — in-memory source, no git root; assert username is "unassigned"

**Tests to add (in apm-core/src/config.rs tests)**

- `server_config_defaults` — config without `[server]` section; assert `config.server.origin == "http://localhost:3000"`
- `server_config_custom_origin` — config with custom `[server]` origin; assert it parses correctly

**Dependency note**: This ticket depends on #610be42e (identity module). If `apm_core::identity` is not yet available when this is implemented, stub the `/api/me` read with a direct inline read of `.apm/local.toml` using `toml` crate. Do not block on it.

### Open questions


### Amendment requests

- [x] Add `ServerConfig { origin: String }` (default `"http://localhost:3000"`) to `apm-core/src/config.rs` and `pub server: ServerConfig` to `Config`. This was originally in ticket 8a08637c but is needed here since `/api/me` is the first endpoint that benefits from knowing the server origin. Ticket 8a08637c will assume this config exists.
- [x] Set effort and risk to non-zero values.

### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-04-02T20:54Z | — | new | apm |
| 2026-04-02T23:22Z | new | groomed | apm |
| 2026-04-02T23:42Z | groomed | in_design | philippepascal |
| 2026-04-02T23:45Z | in_design | specd | claude-0402-2342-b7f2 |
| 2026-04-03T23:42Z | specd | ammend | apm |
| 2026-04-03T23:52Z | ammend | in_design | philippepascal |
| 2026-04-03T23:55Z | in_design | specd | claude-0403-2355-d8a1 |
| 2026-04-04T00:29Z | specd | ready | apm |
| 2026-04-04T02:41Z | ready | in_progress | philippepascal |
| 2026-04-04T02:45Z | in_progress | implemented | claude-0403-1500-f2c1 |
| 2026-04-04T06:02Z | implemented | closed | apm-sync |
