+++
id = "e8c16580"
title = "Enforce session auth on external API requests"
state = "closed"
priority = 0
effort = 3
risk = 2
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/e8c16580-enforce-session-auth-on-external-api-req"
created_at = "2026-04-06T17:41:57.255655Z"
updated_at = "2026-04-06T19:09:24.175459Z"
+++

## Spec

### Problem

The apm-server has a complete WebAuthn/passkey registration and login flow that issues session cookies (__Host-apm-session), but no API route actually validates the session. All 20+ API endpoints (ticket CRUD, sync, work management, batch operations) accept requests from any client without authentication. This means anyone who can reach the server over the network can read and mutate all project data. The auth system needs a middleware layer that: (1) allows localhost requests through without a session (preserving CLI/agent access), and (2) requires a valid session cookie for all external requests, returning 401 otherwise.

### Acceptance criteria

- [x] GET `/api/tickets` from an external IP with no session cookie returns 401 with `{"error":"unauthorized"}`
- [x] GET `/api/tickets` from an external IP with a valid `__Host-apm-session` cookie returns 200
- [x] GET `/api/tickets` from an external IP with an expired or invalid session token returns 401
- [x] GET `/api/tickets` from loopback (127.0.0.1 or ::1) with no session cookie returns 200
- [x] POST `/api/auth/login/challenge` from an external IP with no session cookie returns 200 (auth routes stay open)
- [x] POST `/api/auth/register/challenge` from an external IP with no session cookie returns 200
- [x] GET `/health` from an external IP with no session cookie returns 200
- [x] All protected routes (`/api/sync`, `/api/clean`, `/api/tickets/:id`, `/api/tickets/:id/body`, `/api/tickets/:id/transition`, `/api/tickets/batch/*`, `/api/queue`, `/api/workers`, `/api/workers/:pid`, `/api/work/*`, `/api/agents/config`, `/api/log/stream`, `/api/epics`, `/api/epics/:id`, `/api/me`, `/api/auth/otp`, `/api/auth/sessions`) return 401 when called externally without a valid session
- [x] Existing localhost-only guards in `otp_handler` and session handlers continue to work after the middleware is added

### Out of scope

- CSRF protection (SameSite=Lax on the cookie provides a baseline; separate ticket if needed)
- Per-user authorization / RBAC (all authenticated users have equal access)
- Rate limiting on auth or API endpoints
- Session refresh or rotation logic
- Changes to the WebAuthn registration/login flow itself

### Approach

All building blocks exist in `apm-server/src/main.rs` and `apm-server/src/auth.rs`; this is purely a wiring task.

1. Add `require_auth` middleware function in `main.rs` that:
   - Extracts `ConnectInfo<SocketAddr>` from request extensions
   - Allows loopback IPs through unconditionally
   - Checks `__Host-apm-session` cookie via existing `find_session_username()`
   - Returns 401 `{"error":"unauthorized"}` if neither condition is met

2. Split `build_app` into two sub-routers:
   - **protected** — all data API routes, with `.layer(axum::middleware::from_fn_with_state(state, require_auth))`
   - **open** — `/health`, `/register`, `/login`, `/api/auth/register/*`, `/api/auth/login/*`, `serve_ui` fallback

3. Merge both routers into the final `Router` with `.with_state(state)`

No new dependencies. Reuses existing `find_session_username` and `is_loopback()` pattern.

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-04-06T17:41Z | — | new | philippepascal |
| 2026-04-06T17:43Z | new | groomed | apm |
| 2026-04-06T17:49Z | groomed | in_design | philippepascal |
| 2026-04-06T18:27Z | in_design | specd | claude-0406-1735-b2e1 |
| 2026-04-06T18:30Z | specd | ready | apm |
| 2026-04-06T18:30Z | ready | in_progress | philippepascal |
| 2026-04-06T18:45Z | in_progress | implemented | claude-0406-1830-83a8 |
| 2026-04-06T19:09Z | implemented | closed | apm-sync |
