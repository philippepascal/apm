+++
id = "8a08637c"
title = "apm-server: WebAuthn registration ceremony and embedded registration page"
state = "closed"
priority = 0
effort = 5
risk = 4
author = "apm"
branch = "ticket/8a08637c-apm-server-webauthn-registration-ceremon"
created_at = "2026-04-02T20:54:17.589009Z"
updated_at = "2026-04-04T06:01:54.283144Z"
epic = "8db73240"
target_branch = "epic/8db73240-user-mgmt"
depends_on = ["e2e3d958"]
+++

## Spec

### Problem

There is no registration flow for external devices to enroll a passkey with apm-server. Ticket e2e3d958 (dependency) provides the OTP infrastructure, session store, and localhost bypass — the trust foundation. Without the WebAuthn registration ceremony and an embedded HTML registration page, a device that has received an OTP cannot complete enrollment and cannot authenticate at all.

The desired behaviour: a device visits apm-server in a browser, sees a registration page, enters username + OTP, and completes a biometric-secured WebAuthn credential enrollment. After registration, the device holds a session cookie and can access apm-server.

External devices (phone, remote laptop) are the affected audience. The OTP from `apm register` is the sole trust gate for the ceremony — it is never itself a persistent credential.

### Acceptance criteria

- [x] `GET /register` returns HTTP 200 with Content-Type `text/html` and an HTML page containing a form with username and OTP input fields
- [x] `POST /api/auth/register/challenge` with `{"username": "alice", "otp": "<valid OTP>"}` returns HTTP 200 and a JSON body containing a `reg_id` string and a `publicKey` object with `challenge`, `rp`, and `user` fields
- [x] `POST /api/auth/register/challenge` with a valid username but an invalid OTP returns HTTP 400
- [x] `POST /api/auth/register/challenge` with an expired OTP returns HTTP 400
- [x] `POST /api/auth/register/challenge` with a missing `username` or `otp` field returns HTTP 400
- [x] `POST /api/auth/register/complete` with a valid `reg_id` and a correctly-signed WebAuthn response returns HTTP 200 and sets a `__Host-apm-session` session cookie
- [x] `POST /api/auth/register/complete` with an unknown `reg_id` returns HTTP 400
- [x] `POST /api/auth/register/complete` with a tampered or structurally invalid WebAuthn response returns HTTP 400
- [x] After successful registration, `GET /api/me` with the issued session cookie returns `{"username": "alice"}`
- [x] Using the same OTP a second time (after it was consumed during a prior challenge call) returns HTTP 400
- [x] Two separate devices can each register a passkey for the same username (two `Passkey` entries stored under that username in the credential store)
- [x] Registered credentials survive a server restart: credential data is persisted to `.apm/credentials.json` and reloaded at startup

### Out of scope

- WebAuthn login ceremony (`POST /api/auth/login`) — separate ticket
- `apm register <username>` CLI command — separate ticket
- `apm sessions` and `apm revoke` CLI commands — separate ticket
- Auth enforcement on existing API routes (all existing routes remain publicly accessible) — separate ticket
- TLS termination — handled by apm-proxy
- Attestation verification policy — any authenticator type is accepted (no attestation constraints)
- Redirect flow: after registration, the server returns 200 JSON; redirect to the main UI is left for a follow-up

### Approach

**Dependencies — `apm-server/Cargo.toml`:**
- Add `webauthn-rs = { version = "0.5", features = ["danger-allow-state-serialisation"] }`
  (default features provide passkey support; the serialisation feature is included for flexibility but in-flight state stays in memory)
- Add `uuid = { version = "1", features = ["v5"] }` for deterministic per-user UUID generation

**Prerequisite — `config.server.origin` (from ticket 90ebf40b):**
This ticket assumes `ServerConfig { origin: String }` and `Config.server` already exist, added by ticket 90ebf40b. The origin value (default `"http://localhost:3000"`) is read from `config.server.origin` when initialising WebAuthn state.

**New file `apm-server/src/webauthn_state.rs`:**
- `RegistrationSession { username: String, passkey_reg: PasskeyRegistration }`
- `WebauthnState { webauthn: Webauthn, pending: Arc<Mutex<HashMap<String, RegistrationSession>>> }`
- `WebauthnState::new(origin: &str) -> anyhow::Result<Self>`: parse origin URL, extract hostname as rp_id, call `WebauthnBuilder::new(rp_id, &origin_url)?.build()`

**New file `apm-server/src/credential_store.rs`:**
- `CredentialStore { inner: Arc<Mutex<HashMap<String, Vec<Passkey>>>>, path: PathBuf }`
- `CredentialStore::load(path) -> Self`: read `.apm/credentials.json`; start empty on absent file (log warning on parse failure)
- `CredentialStore::insert(username, passkey)`: append passkey to user's vec; call `save()`
- `CredentialStore::save()`: write JSON atomically (temp file + rename) to `.apm/credentials.json`
- File format: `{"credentials": {"alice": [<Passkey JSON>, ...]}}`

**Extend `AppState` in `apm-server/src/main.rs`:**
- Add `webauthn_state: Arc<WebauthnState>` and `credential_store: CredentialStore` fields
- Initialise in `build_app(root)`: read `config.server.origin`, call `WebauthnState::new(&origin)`, call `CredentialStore::load(root.join(".apm/credentials.json"))`

**New routes (add to `build_app` router):**
- `GET /register` -> `register_page_handler` (returns `include_str!("register.html")` with content-type `text/html`)
- `POST /api/auth/register/challenge` -> `register_challenge_handler`
- `POST /api/auth/register/complete` -> `register_complete_handler`

**`register_challenge_handler`:**
1. Parse `{username: String, otp: String}` -- 400 if malformed
2. `otp_store.validate(username, otp)` -- 400 with message if invalid/expired (OTP is consumed here)
3. Build a deterministic `Uuid` per username: `Uuid::new_v5(&Uuid::NAMESPACE_OID, username.as_bytes())`
4. `webauthn.start_passkey_registration(user_uuid, username, username, None)` -> `(challenge, passkey_reg)`
5. Generate `reg_id` (16 random bytes as lowercase hex, reuse `generate_token()` from `auth.rs`)
6. Store `RegistrationSession { username, passkey_reg }` in `webauthn_state.pending` under `reg_id`
7. Return `{"reg_id": reg_id, "publicKey": <CreationChallengeResponse JSON>}` -- 200

**`register_complete_handler`:**
1. Parse `{reg_id: String, response: RegisterPublicKeyCredential}` -- 400 if malformed
2. Remove and retrieve `RegistrationSession` by `reg_id` from `webauthn_state.pending` -- 400 if not found
3. `webauthn.finish_passkey_registration(&response, &passkey_reg)` -- 400 if verification fails
4. `credential_store.insert(username, passkey)` -- persists credential
5. Generate new session token, `session_store.insert(token, username)`
6. Return HTTP 200 with `Set-Cookie: __Host-apm-session=<token>; HttpOnly; Secure; SameSite=Lax; Path=/; Max-Age=604800`
   Note: `__Host-` prefix requires HTTPS; browsers on plain HTTP will reject it. This is intentional -- local access bypasses auth entirely (localhost bypass from e2e3d958), and external access uses apm-proxy with TLS.

**`apm-server/src/register.html`** (embedded via `include_str!`):
- Minimal HTML: username input, OTP input, Register button, status div
- Vanilla JS on button click:
  1. POST `/api/auth/register/challenge` with `{username, otp}`
  2. Convert base64url fields to ArrayBuffer (challenge, user.id)
  3. `navigator.credentials.create({publicKey: ...})` -- triggers biometric prompt
  4. Encode response fields (clientDataJSON, attestationObject, id) back to base64url
  5. POST `/api/auth/register/complete` with `{reg_id, response}`
  6. Display success or error message; on success, redirect to `/` after 2 seconds
- No external JS dependencies; base64url helpers inline (~20 lines)

**Tests (unit, in `apm-server/src/`):**
- `CredentialStore::load` on absent file returns empty store
- `CredentialStore::insert` + reload round-trip preserves passkey count
- `WebauthnState::new` with `"http://localhost:3000"` succeeds
- `register_page_handler` returns 200 HTML (integration test via `build_app_with_auth` test helper)
- Challenge endpoint returns 400 for missing fields (integration test)
- Challenge endpoint returns 400 for invalid OTP (integration test, using a seeded OTP store)
- Full WebAuthn ceremony tests require a real authenticator and are excluded

### Open questions


### Amendment requests

- [x] Move `ServerConfig { origin }` and `pub server: ServerConfig` on `Config` out of this ticket and into ticket 90ebf40b (expose author in API / `/api/me`). The server origin is needed as soon as `/api/me` exists — not only for WebAuthn. 90ebf40b's approach should include the config addition; this ticket should assume `config.server.origin` already exists.
- [x] Set effort and risk to non-zero values.

### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-04-02T20:54Z | — | new | apm |
| 2026-04-02T23:22Z | new | groomed | apm |
| 2026-04-02T23:51Z | groomed | in_design | philippepascal |
| 2026-04-02T23:56Z | in_design | specd | claude-0402-2351-spec1 |
| 2026-04-03T23:42Z | specd | ammend | apm |
| 2026-04-03T23:55Z | ammend | in_design | philippepascal |
| 2026-04-03T23:57Z | in_design | specd | claude-0403-2358-sw01 |
| 2026-04-04T00:29Z | specd | ready | apm |
| 2026-04-04T03:19Z | ready | in_progress | philippepascal |
| 2026-04-04T03:31Z | in_progress | implemented | claude-0403-2320-w8a0 |
| 2026-04-04T06:01Z | implemented | closed | apm-sync |
