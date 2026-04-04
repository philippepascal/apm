+++
id = "70d58b2d"
title = "apm-server: WebAuthn login/assertion ceremony and embedded login page"
state = "closed"
priority = 0
effort = 4
risk = 3
author = "apm"
branch = "ticket/70d58b2d-apm-server-webauthn-login-assertion-cere"
created_at = "2026-04-02T20:54:21.301151Z"
updated_at = "2026-04-04T06:01:31.475606Z"
epic = "8db73240"
target_branch = "epic/8db73240-user-mgmt"
depends_on = ["8a08637c"]
+++

## Spec

### Problem

Registered devices have no way to authenticate on subsequent visits to apm-server. Once a user has enrolled a passkey via the registration ceremony (ticket 8a08637c), there is no endpoint that issues a session cookie to a returning device. Without a WebAuthn assertion ceremony and a matching login page, every visit after the first requires re-registration, which defeats the purpose of passkeys.

The desired behaviour: a returning browser that holds a registered passkey visits apm-server, sees a login page, enters their username, completes a biometric prompt, and receives a session cookie granting access. The server verifies the signed challenge against the stored public key — no shared secret is transmitted.

External devices (phone, remote laptop) are the primary audience. Localhost requests bypass auth entirely (established in ticket e2e3d958) so the login flow is only reachable by non-loopback clients.

### Acceptance criteria

- [x] `GET /login` returns HTTP 200 with Content-Type `text/html` and an HTML page containing a username input field and a sign-in button
- [x] `POST /api/auth/login/challenge` with `{"username": "alice"}` where alice has at least one registered credential returns HTTP 200 and a JSON body containing a `login_id` string and a `publicKey` object with `challenge` and `allowCredentials` fields
- [x] `POST /api/auth/login/challenge` with a username that has no registered credentials returns HTTP 400
- [x] `POST /api/auth/login/challenge` with a missing or malformed `username` field returns HTTP 400
- [x] `POST /api/auth/login/complete` with a valid `login_id` and a correctly-signed WebAuthn assertion returns HTTP 200 and sets a `__Host-apm-session` session cookie
- [x] `POST /api/auth/login/complete` with an unknown `login_id` returns HTTP 400
- [x] `POST /api/auth/login/complete` with a tampered or structurally invalid assertion response returns HTTP 400
- [x] `POST /api/auth/login/complete` with a `login_id` whose pending session is older than 5 minutes returns HTTP 400
- [x] After successful login, `GET /api/me` with the issued session cookie returns `{"username": "alice"}`
- [x] After successful login, the credential counter update from `AuthenticationResult` is persisted to `.apm/credentials.json`
- [x] A second `POST /api/auth/login/complete` call with the same `login_id` (after it was consumed on the first call) returns HTTP 400

### Out of scope

- WebAuthn registration ceremony — ticket 8a08637c
- OTP generation and session/OTP infrastructure — ticket e2e3d958
- `apm register <username>` CLI command — separate ticket
- `apm sessions` and `apm revoke` CLI commands — separate ticket
- Auth enforcement on existing API routes (all existing routes remain publicly accessible) — separate ticket
- Redirect-on-unauthenticated: when a remote client hits a non-login route without a session, no redirect is issued (enforcement is deferred)
- TLS termination — handled by apm-proxy
- Attestation or authenticator policy — any stored passkey is accepted for assertion
- Multi-device login UI (the login page asks only for username; the browser selects the matching credential automatically via allowCredentials)

### Approach

This ticket builds directly on the types and infrastructure established by tickets e2e3d958 and 8a08637c. All changes are additive; nothing existing is removed or restructured.

**Extend `WebauthnState` in `apm-server/src/webauthn_state.rs`:**
- Add `AuthenticationSession { username: String, passkey_auth: PasskeyAuthentication, created_at: std::time::Instant }`
- Add `pending_auth: Arc<Mutex<HashMap<String, AuthenticationSession>>>` field to `WebauthnState`
- Initialise `pending_auth` to an empty map in `WebauthnState::new`

**Extend `CredentialStore` in `apm-server/src/credential_store.rs`:**
- Add `get(username) -> Option<Vec<Passkey>>`: acquires lock, clones and returns the vec for that username (None if absent or empty)
- Add `update_credential(username: &str, auth_result: &AuthenticationResult)`: acquires lock, calls `passkey.update_credential(auth_result)` for every passkey in the user's vec (webauthn-rs updates the matching entry in place), then calls `save()`

**New routes — add to `build_app` router in `apm-server/src/main.rs`:**
- `GET /login` → `login_page_handler`
- `POST /api/auth/login/challenge` → `login_challenge_handler`
- `POST /api/auth/login/complete` → `login_complete_handler`

**`login_page_handler`:**
- Returns `include_str!("login.html")` as `text/html; charset=utf-8`, HTTP 200

**`login_challenge_handler`:**
1. Parse `{username: String}` — return 400 if malformed or missing
2. `credential_store.get(&username)` — return 400 if None (user unknown or has no passkeys)
3. `webauthn_state.webauthn.start_passkey_authentication(&credentials)` → `(challenge, passkey_auth)` — return 400 on error
4. Generate `login_id` using `generate_token()` (already defined in `auth.rs` by e2e3d958)
5. Store `AuthenticationSession { username, passkey_auth, created_at: Instant::now() }` in `pending_auth` under `login_id`
6. Return HTTP 200: `{"login_id": login_id, "publicKey": <RequestChallengeResponse JSON>}`

**`login_complete_handler`:**
1. Parse `{login_id: String, response: PublicKeyCredential}` — return 400 if malformed
2. Remove and retrieve `AuthenticationSession` by `login_id` from `pending_auth` — return 400 if not found (covers both unknown and already-consumed)
3. Check `session.created_at.elapsed() < Duration::from_secs(300)` — return 400 if expired
4. `credential_store.get(&session.username)` to fetch current passkeys — return 400 if gone
5. `webauthn_state.webauthn.finish_passkey_authentication(&response, &session.passkey_auth)` → `auth_result` — return 400 on error
6. `credential_store.update_credential(&session.username, &auth_result)` — persists updated counter
7. `generate_token()` for session token; `session_store.insert(token.clone(), session.username)`
8. Return HTTP 200 with `Set-Cookie: __Host-apm-session=<token>; HttpOnly; Secure; SameSite=Lax; Path=/; Max-Age=604800`

**New file `apm-server/src/login.html` (embedded via `include_str!`):**
- Minimal HTML: username input, "Sign in with passkey" button, status div
- Vanilla JS (no external dependencies):
  1. POST `/api/auth/login/challenge` with `{username}`
  2. Decode base64url fields to ArrayBuffer (challenge; credential id bytes in each `allowCredentials` entry)
  3. `navigator.credentials.get({publicKey: ...})` — triggers OS biometric prompt
  4. Encode response fields (clientDataJSON, authenticatorData, signature, userHandle) back to base64url
  5. POST `/api/auth/login/complete` with `{login_id, response}`
  6. Display success message; redirect to `/` after 2 seconds on success, or display error message on failure
- Reuse the same base64url helper functions as `register.html` (copy inline — both pages are self-contained)

**Tests:**
Unit tests in `apm-server/src/webauthn_state.rs` or a new `login_tests` module:
- `AuthenticationSession` with `created_at` 6 minutes in the past fails the TTL check (construct with overridden `created_at` via a helper)

Integration tests (matching existing tower::ServiceExt test patterns):
- `GET /login` returns 200 with content-type `text/html`
- `POST /api/auth/login/challenge` with malformed body returns 400
- `POST /api/auth/login/challenge` with username having no credentials returns 400 (empty `CredentialStore`)
- `POST /api/auth/login/complete` with unknown `login_id` returns 400
- Full WebAuthn assertion ceremony tests require a real authenticator and are excluded (same carve-out as registration ticket)

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-04-02T20:54Z | — | new | apm |
| 2026-04-02T23:22Z | new | groomed | apm |
| 2026-04-02T23:56Z | groomed | in_design | philippepascal |
| 2026-04-02T23:59Z | in_design | specd | claude-0402-2356-b7f2 |
| 2026-04-04T00:29Z | specd | ready | apm |
| 2026-04-04T03:32Z | ready | in_progress | philippepascal |
| 2026-04-04T03:38Z | in_progress | implemented | claude-0403-2100-f4e2 |
| 2026-04-04T06:01Z | implemented | closed | apm-sync |
