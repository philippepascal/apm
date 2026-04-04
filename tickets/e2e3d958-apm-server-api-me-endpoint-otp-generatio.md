+++
id = "e2e3d958"
title = "apm-server: /api/me endpoint, OTP generation, session store, and localhost bypass"
state = "closed"
priority = 0
effort = 5
risk = 2
author = "apm"
branch = "ticket/e2e3d958-apm-server-api-me-endpoint-otp-generatio"
created_at = "2026-04-02T20:54:13.959036Z"
updated_at = "2026-04-04T06:02:25.189158Z"
epic = "8db73240"
target_branch = "epic/8db73240-user-mgmt"
depends_on = ["4cec7a17", "90ebf40b"]
+++

## Spec

### Problem

apm-server has no authentication and no way to identify the current user beyond the localhost case implemented in ticket #90ebf40b. Any client that can reach the server gets full access. External clients (phone, remote laptop) need a secure auth scheme, but before the WebAuthn registration and login ceremonies can be built, the underlying infrastructure must exist: a place to store short-lived OTPs, a place to store authenticated sessions, a mechanism for localhost requests to bypass auth entirely, and a `/api/me` endpoint that answers correctly for all three request categories (localhost, authenticated session, unauthenticated remote).

Ticket #90ebf40b already implements the localhost case of `/api/me` (reading `.apm/local.toml`). This ticket extends that endpoint to also handle session-authenticated remote requests, and adds the OTP generation endpoint (`POST /api/auth/otp`) that `apm register` will call (CLI command is a separate ticket). It does not implement the WebAuthn ceremonies or any session issuance — those depend on this foundation.

### Acceptance criteria

- [x] `POST /api/auth/otp` from 127.0.0.1 with body `{"username": "alice"}` returns HTTP 200 and `{"otp": "<8-char alphanumeric>"}`
- [x] `POST /api/auth/otp` from a non-loopback IP returns HTTP 403
- [x] `POST /api/auth/otp` with a missing or malformed body returns HTTP 400
- [x] Two consecutive `POST /api/auth/otp` calls for the same username replace the first OTP (only one active OTP per user at a time)
- [x] A stored OTP has a creation timestamp and a 5-minute TTL; the OTP store's `validate_otp` function returns an error for an expired OTP
- [x] `validate_otp(username, otp)` returns `Ok(())` on the first correct call and an error on a second call with the same OTP (single-use)
- [x] `GET /api/me` from 127.0.0.1 returns `{"username": "<value>"}` matching the `username` field in `.apm/local.toml`
- [x] `GET /api/me` from 127.0.0.1 when `.apm/local.toml` is absent or has no `username` field returns `{"username": "unassigned"}`
- [x] `GET /api/me` from a remote IP with a valid, non-expired `__Host-apm-session` cookie returns `{"username": "<session username>"}`
- [x] `GET /api/me` from a remote IP with an expired `__Host-apm-session` cookie returns `{"username": "unassigned"}`
- [x] `GET /api/me` from a remote IP with no session cookie returns `{"username": "unassigned"}`
- [x] Sessions survive a server restart: the session store is loaded from `.apm/sessions.json` at startup and written to it when entries are added
- [x] Session entries older than 7 days are not returned as valid by the session store lookup

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

New file: apm-server/src/auth.rs

OTP store:
- OtpEntry containing username, otp string, and created_at timestamp; stored in a HashMap keyed by username (one active OTP per user at a time)
- OtpStore wraps Arc<Mutex<HashMap<String, OtpEntry>>>
- generate_otp(): 8 random uppercase alphanumeric chars via rand::distributions::Alphanumeric mapped to uppercase
- OtpStore::insert(username, otp): stores or overwrites the entry for that username
- OtpStore::validate(username, otp) -> Result<(), OtpError>: checks existence, TTL (5 min from created_at), and exact value match; removes entry on success so it cannot be reused; typed errors: NotFound, Expired, Invalid

Session store:
- Session struct containing username and expires_at (serde Serialize/Deserialize)
- SessionStore holds Arc<Mutex<HashMap<String, Session>>> and the file path
- generate_token(): 32 random bytes encoded as lowercase hex
- SessionStore::load(path): reads .apm/sessions.json; starts empty if absent or unparseable (warn on parse failure)
- SessionStore::insert(token, username): stores session expiring 7 days from now; calls save()
- SessionStore::lookup(token) -> Option<String>: returns Some(username) if valid and not expired; removes expired entries and returns None
- SessionStore::save(): writes JSON atomically to .apm/sessions.json via temp file + rename
- File format: JSON object with a "sessions" array, each entry having token, username, expires_at fields

AppState extension (main.rs):
- Add otp_store: OtpStore and session_store: SessionStore fields
- Initialise both in main before building the router; SessionStore::load called once with path to .apm/sessions.json in the repo root

Localhost extractor:
- struct IsLocalhost(bool) implementing axum::extract::FromRequestParts
- Reads ConnectInfo<SocketAddr>, checks addr.ip().is_loopback()
- Ensure main.rs uses into_make_service_with_connect_info::<SocketAddr>() (change from into_make_service() if needed)

POST /api/auth/otp handler:
1. Extract IsLocalhost — return 403 if not localhost
2. Parse JSON body with username field — return 400 if missing, malformed, or empty
3. Generate OTP, store via otp_store.insert(username, otp)
4. Return JSON {otp: ...} with HTTP 200

Extend GET /api/me (from ticket #90ebf40b):
1. Extract IsLocalhost
2. Localhost: unchanged — resolve via apm_core::resolve_identity, return username or "unassigned"
3. Remote: read Cookie header, parse with the cookie crate, look up __Host-apm-session token value
   - Valid non-expired session found: return {username: session_username}
   - No cookie or expired: return {username: unassigned}

Route change (main.rs):
- Add .route("/api/auth/otp", post(otp_handler)) to the router

New dependencies (apm-server/Cargo.toml):
- rand = "0.8" for OTP and token generation
- cookie = "0.18" for Cookie header parsing
Neither needs workspace-level promotion.

Tests:
Unit tests in auth.rs:
- OtpStore: insert+validate happy path; validate expired OTP; validate wrong value; validate twice (second call fails after entry removed on first success)
- SessionStore: insert+lookup happy path; lookup expired session returns None

Integration tests via tower::ServiceExt (matching existing test patterns in apm-server):
- POST /api/auth/otp with loopback peer addr: HTTP 200, body is 8-char alphanumeric OTP
- POST /api/auth/otp with non-loopback peer addr: HTTP 403
- POST /api/auth/otp with malformed body: HTTP 400
- GET /api/me from loopback with local.toml in temp repo: returns local.toml username
- GET /api/me from loopback without local.toml: returns "unassigned"
- GET /api/me from remote with valid session inserted directly into store and matching Cookie header: returns session username
- GET /api/me from remote with expired session: returns "unassigned"
- GET /api/me from remote with no Cookie header: returns "unassigned"

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-04-02T20:54Z | — | new | apm |
| 2026-04-02T23:22Z | new | groomed | apm |
| 2026-04-02T23:45Z | groomed | in_design | philippepascal |
| 2026-04-02T23:51Z | in_design | specd | claude-0402-1445-b7e2 |
| 2026-04-04T00:29Z | specd | ready | apm |
| 2026-04-04T03:06Z | ready | in_progress | philippepascal |
| 2026-04-04T03:18Z | in_progress | implemented | claude-0403-2010-f4a2 |
| 2026-04-04T06:02Z | implemented | closed | apm-sync |
