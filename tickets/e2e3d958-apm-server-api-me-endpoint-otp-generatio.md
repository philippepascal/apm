+++
id = "e2e3d958"
title = "apm-server: /api/me endpoint, OTP generation, session store, and localhost bypass"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "apm"
agent = "64526"
branch = "ticket/e2e3d958-apm-server-api-me-endpoint-otp-generatio"
created_at = "2026-04-02T20:54:13.959036Z"
updated_at = "2026-04-02T23:45:23.811161Z"
epic = "8db73240"
target_branch = "epic/8db73240-user-mgmt"
depends_on = ["4cec7a17", "90ebf40b"]
+++

## Spec

### Problem

apm-server has no authentication and no way to identify the current user beyond the localhost case implemented in ticket #90ebf40b. Any client that can reach the server gets full access. External clients (phone, remote laptop) need a secure auth scheme, but before the WebAuthn registration and login ceremonies can be built, the underlying infrastructure must exist: a place to store short-lived OTPs, a place to store authenticated sessions, a mechanism for localhost requests to bypass auth entirely, and a `/api/me` endpoint that answers correctly for all three request categories (localhost, authenticated session, unauthenticated remote).

Ticket #90ebf40b already implements the localhost case of `/api/me` (reading `.apm/local.toml`). This ticket extends that endpoint to also handle session-authenticated remote requests, and adds the OTP generation endpoint (`POST /api/auth/otp`) that `apm register` will call (CLI command is a separate ticket). It does not implement the WebAuthn ceremonies or any session issuance — those depend on this foundation.

### Acceptance criteria

- [ ] `POST /api/auth/otp` from 127.0.0.1 with body `{"username": "alice"}` returns HTTP 200 and `{"otp": "<8-char alphanumeric>"}`
- [ ] `POST /api/auth/otp` from a non-loopback IP returns HTTP 403
- [ ] `POST /api/auth/otp` with a missing or malformed body returns HTTP 400
- [ ] Two consecutive `POST /api/auth/otp` calls for the same username replace the first OTP (only one active OTP per user at a time)
- [ ] A stored OTP has a creation timestamp and a 5-minute TTL; the OTP store's `validate_otp` function returns an error for an expired OTP
- [ ] `validate_otp(username, otp)` returns `Ok(())` on the first correct call and an error on a second call with the same OTP (single-use)
- [ ] `GET /api/me` from 127.0.0.1 returns `{"username": "<value>"}` matching the `username` field in `.apm/local.toml`
- [ ] `GET /api/me` from 127.0.0.1 when `.apm/local.toml` is absent or has no `username` field returns `{"username": "unassigned"}`
- [ ] `GET /api/me` from a remote IP with a valid, non-expired `__Host-apm-session` cookie returns `{"username": "<session username>"}`
- [ ] `GET /api/me` from a remote IP with an expired `__Host-apm-session` cookie returns `{"username": "unassigned"}`
- [ ] `GET /api/me` from a remote IP with no session cookie returns `{"username": "unassigned"}`
- [ ] Sessions survive a server restart: the session store is loaded from `.apm/sessions.json` at startup and written to it when entries are added
- [ ] Session entries older than 7 days are not returned as valid by the session store lookup

### Out of scope

- WebAuthn registration ceremony (POST /api/auth/register challenge/response)
- WebAuthn login ceremony (POST /api/auth/login challenge/response)
- Session cookie issuance — the session store is built here but sessions are only created by the registration/login ceremonies (separate tickets); this ticket only reads the cookie and looks up existing sessions
- OTP consumption during registration — `validate_otp` is implemented and tested here, but it is only called by the registration handler (separate ticket)
- `apm register <username>` CLI command — it calls `POST /api/auth/otp` but the CLI side is a separate ticket
- `apm sessions` and `apm revoke` CLI commands
- Auth enforcement on existing routes — all existing API routes remain publicly accessible; this ticket only adds identity resolution, not access control
- Adding `.apm/sessions.json` to `.gitignore` — this is handled by `apm init` (ticket #4cec7a17 or a follow-up); the server reads/writes the file regardless

### Approach

How the implementation will work.

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-04-02T20:54Z | — | new | apm |
| 2026-04-02T23:22Z | new | groomed | apm |
| 2026-04-02T23:45Z | groomed | in_design | philippepascal |