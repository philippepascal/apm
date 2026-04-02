+++
id = "90ebf40b"
title = "apm-server: expose author field in ticket API responses"
state = "in_design"
priority = 0
effort = 3
risk = 0
author = "apm"
agent = "65291"
branch = "ticket/90ebf40b-apm-server-expose-author-field-in-ticket"
created_at = "2026-04-02T20:54:08.576527Z"
updated_at = "2026-04-02T23:44:44.511932Z"
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

- [ ] `GET /api/tickets` response objects always include an `author` field (never omitted), using `"unassigned"` when the frontmatter has no author set
- [ ] `GET /api/tickets/:id` response object always includes an `author` field, using `"unassigned"` when the frontmatter has no author set
- [ ] `GET /api/tickets?author=<username>` returns only tickets where `author` equals `<username>`
- [ ] `GET /api/tickets?author=unassigned` returns only tickets whose frontmatter author is absent or equal to `"unassigned"`
- [ ] `GET /api/me` returns `{"username": "<value>"}` where `<value>` is the `username` from `.apm/local.toml` when that file exists and contains a non-empty key
- [ ] `GET /api/me` returns `{"username": "unassigned"}` when `.apm/local.toml` is absent or contains no `username` key
- [ ] Existing tests continue to pass after the change

### Out of scope

- WebAuthn / OTP authentication (DESIGN-users.md point 5)
- `GET /api/me` for WebAuthn-authenticated sessions (returns local identity only; session-cookie resolution is a later auth ticket)
- `apm list --mine` / `apm list --author` CLI flags (ticket #610be42e out of scope, separate ticket)
- Supervisor board UI changes — filter bar, default author filter, card display (separate UI ticket)
- Collaborators list validation or sync from GitHub (DESIGN-users.md point 4)
- `apm init` prompting for username
- Removing or migrating existing `agent` field values in frontmatter (ticket #610be42e)

### Approach

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

3. **`GET /api/me` endpoint** — add a handler:
   ```rust
   async fn me_handler(State(state): State<Arc<AppState>>) -> Json<serde_json::Value> {
       let username = state.git_root()
           .and_then(|root| apm_core::identity::read_local_username(root).ok())
           .flatten()
           .unwrap_or_else(|| "unassigned".to_string());
       Json(serde_json::json!({ "username": username }))
   }
   ```
   Register as `.route("/api/me", get(me_handler))`.

   This depends on `apm_core::identity` being available (landed in ticket #610be42e). If the function name differs, match whatever #610be42e exposes. The function should read `.apm/local.toml`, return `Option<String>`.

**Tests to add (in apm-server/src/main.rs #[cfg(test)] block)**

- `list_tickets_author_field_always_present` — ticket with `author: None`; assert response JSON includes `"author": "unassigned"`
- `list_tickets_author_filter` — two tickets with different authors; `?author=alice` returns only Alice's
- `list_tickets_author_unassigned_filter` — ticket with `author: None`; `?author=unassigned` returns it
- `get_ticket_author_field_always_present` — ticket with `author: None`; assert detail response includes `"author": "unassigned"`
- `me_handler_returns_unassigned_when_no_local_toml` — in-memory source, no git root; assert `{"username":"unassigned"}`

**Dependency note**: This ticket is listed as depending on #610be42e (identity module). If `apm_core::identity` is not yet available when this is implemented, stub the `/api/me` read with a direct inline read of `.apm/local.toml` using `toml` crate, and leave a `// TODO: use apm_core::identity once #610be42e lands` comment. Do not block on it.

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-04-02T20:54Z | — | new | apm |
| 2026-04-02T23:22Z | new | groomed | apm |
| 2026-04-02T23:42Z | groomed | in_design | philippepascal |