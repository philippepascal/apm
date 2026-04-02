+++
id = "70d58b2d"
title = "apm-server: WebAuthn login/assertion ceremony and embedded login page"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "apm"
agent = "21293"
branch = "ticket/70d58b2d-apm-server-webauthn-login-assertion-cere"
created_at = "2026-04-02T20:54:21.301151Z"
updated_at = "2026-04-02T23:56:27.321648Z"
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

- [ ] `GET /login` returns HTTP 200 with Content-Type `text/html` and an HTML page containing a username input field and a sign-in button
- [ ] `POST /api/auth/login/challenge` with `{"username": "alice"}` where alice has at least one registered credential returns HTTP 200 and a JSON body containing a `login_id` string and a `publicKey` object with `challenge` and `allowCredentials` fields
- [ ] `POST /api/auth/login/challenge` with a username that has no registered credentials returns HTTP 400
- [ ] `POST /api/auth/login/challenge` with a missing or malformed `username` field returns HTTP 400
- [ ] `POST /api/auth/login/complete` with a valid `login_id` and a correctly-signed WebAuthn assertion returns HTTP 200 and sets a `__Host-apm-session` session cookie
- [ ] `POST /api/auth/login/complete` with an unknown `login_id` returns HTTP 400
- [ ] `POST /api/auth/login/complete` with a tampered or structurally invalid assertion response returns HTTP 400
- [ ] `POST /api/auth/login/complete` with a `login_id` whose pending session is older than 5 minutes returns HTTP 400
- [ ] After successful login, `GET /api/me` with the issued session cookie returns `{"username": "alice"}`
- [ ] After successful login, the credential counter update from `AuthenticationResult` is persisted to `.apm/credentials.json`
- [ ] A second `POST /api/auth/login/complete` call with the same `login_id` (after it was consumed on the first call) returns HTTP 400

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

How the implementation will work.

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-04-02T20:54Z | — | new | apm |
| 2026-04-02T23:22Z | new | groomed | apm |
| 2026-04-02T23:56Z | groomed | in_design | philippepascal |