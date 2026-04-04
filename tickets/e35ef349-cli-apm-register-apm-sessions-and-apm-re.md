+++
id = "e35ef349"
title = "CLI: apm register, apm sessions, and apm revoke commands"
state = "closed"
priority = 0
effort = 4
risk = 3
author = "apm"
branch = "ticket/e35ef349-cli-apm-register-apm-sessions-and-apm-re"
created_at = "2026-04-02T20:54:25.629052Z"
updated_at = "2026-04-04T06:02:30.658812Z"
epic = "8db73240"
target_branch = "epic/8db73240-user-mgmt"
depends_on = ["8a08637c"]
+++

## Spec

### Problem

The APM server supports WebAuthn-based authentication for remote collaborators, but there is no CLI surface to operate the auth lifecycle. Specifically:

- **Device registration**: a new device must be bootstrapped with a one-time password generated server-side. Without `apm register`, a project admin has no way to produce that OTP from the command line; they would need to call the raw HTTP endpoint manually.
- **Session visibility**: there is no way to inspect which devices hold active sessions, making it impossible to audit access or detect stale/compromised tokens.
- **Session revocation**: there is no way to invalidate a session. If a device is lost or a token is leaked, the only recourse is to wait for the 7-day TTL to expire.

The desired behaviour is three new subcommands — `apm register`, `apm sessions`, and `apm revoke` — that let a localhost-connected admin drive the full auth lifecycle without leaving the terminal. The server-side list and revoke endpoints also do not yet exist and must be added as part of this ticket.

### Acceptance criteria

- [x] `apm register <username>` prints an 8-character OTP on stdout when apm-server is running on localhost
- [x] `apm register <username>` exits non-zero and prints a human-readable error when the server is not reachable
- [x] `apm sessions` prints a table of active (non-expired) sessions with columns: Username, Device, Last Seen, Expires
- [x] `apm sessions` prints "No active sessions." when the session store is empty or all sessions are expired
- [x] `apm sessions` exits non-zero and prints a human-readable error when the server is not reachable
- [x] `apm revoke <username>` removes all sessions for that user and prints how many were revoked
- [x] `apm revoke <username>` exits 0 and prints "No sessions found for <username>." when no sessions exist for that user
- [x] `apm revoke <username> --device <hint>` removes only sessions whose device hint matches and exits 0
- [x] `apm revoke --all` removes every session for every user and prints the total count revoked
- [x] `apm revoke` exits non-zero and prints a human-readable error when the server is not reachable
- [x] `GET /api/auth/sessions` returns HTTP 403 when the request originates from a non-localhost address
- [x] `DELETE /api/auth/sessions` returns HTTP 403 when the request originates from a non-localhost address

### Out of scope

- Revoking WebAuthn credentials (passkeys stored in `.apm/credentials.json`) — only session tokens are revoked
- The WebAuthn registration ceremony itself (covered by ticket 8a08637c)
- OTP generation and session-store server logic (covered by ticket e2e3d958)
- `apm login` or any client-side WebAuthn authentication flow
- `apm init` prompting for username (separate ticket)
- `apm list --mine` / `--author <username>` filtering (separate ticket)
- TLS or remote-access configuration for apm-server

### Approach

Tickets e2e3d958 and 8a08637c must land on `epic/8db73240-user-mgmt` before this ticket. `apm-server/src/auth.rs` (with `OtpStore`, `SessionStore`, `IsLocalhost`) and `apm-server/src/credential_store.rs` are expected to exist at that point.

**1. reqwest in CLI** — add to `apm/Cargo.toml`:
```toml
reqwest = { version = "0.11", features = ["blocking", "json"] }
serde_json = "1"
```

**2. Server URL from config** — in `apm-core/src/config.rs`, add an optional `[server]` table:
```rust
#[derive(Deserialize, Default)]
pub struct ServerConfig {
    #[serde(default = "default_server_url")]
    pub url: String,
}
fn default_server_url() -> String { "http://127.0.0.1:3000".to_owned() }
```
Add `pub server: ServerConfig` to the top-level `Config` struct (with `#[serde(default)]`). CLI commands read `config.server.url` to know where to connect.

**3. `apm register <username>`** — new `apm/src/cmd/register.rs`:
- POST `{server_url}/api/auth/otp` with body `{"username":"<u>"}`
- On HTTP 200: parse `{"otp":"XXXXXXXX"}`, print the OTP
- On connection error or non-200: stderr message, exit 1

**4. `apm sessions`** — new `apm/src/cmd/sessions.rs`:
- GET `{server_url}/api/auth/sessions`
- Response: `Vec<SessionInfo>` with `username`, `device_hint: Option<String>`, `last_seen`, `expires_at`
- Empty vec → print "No active sessions."
- Otherwise → aligned table with columns: USERNAME, DEVICE, LAST SEEN, EXPIRES

**5. `apm revoke`** — new `apm/src/cmd/revoke.rs`:
- Args: `[<username>]`, `[--device <hint>]`, `[--all]`; `--all` is mutually exclusive with `--device`; `<username>` required unless `--all`
- DELETE `{server_url}/api/auth/sessions` with JSON body `{"username":..., "device":..., "all":...}`
- Response: `{"revoked": N}`; print "Revoked N session(s)." or "No sessions found for <username>." when N==0

**6. New server endpoints** — extend `apm-server/src/auth.rs` (added by e2e3d958):
- `GET /api/auth/sessions` → handler `list_sessions(IsLocalhost, State)`: filters out expired sessions, returns `Vec<SessionInfo>` (tokens never exposed)
- `DELETE /api/auth/sessions` → handler `revoke_sessions(IsLocalhost, State, Json<RevokeRequest>)`: removes matching sessions; `all=true` clears everything
- New types: `SessionInfo { username, device_hint, last_seen, expires_at }`, `RevokeRequest { username: Option<String>, device: Option<String>, all: bool }`, `RevokeResponse { revoked: usize }`
- `IsLocalhost` extractor (from e2e3d958) returns HTTP 403 for non-localhost callers — no extra guard needed

**7. Route registration** — in `apm-server/src/main.rs`:
```rust
.route("/api/auth/sessions", get(list_sessions).delete(revoke_sessions))
```

**8. Wire CLI commands** — in `apm/src/main.rs` add `Register { username }`, `Sessions`, `Revoke { username, device, all }` variants to the `Commands` enum and dispatch to the new modules.

**9. Tests**
- Unit tests in `apm-server/src/auth.rs`: `list_sessions` excludes expired sessions; `revoke_sessions` with `all=true` clears store; `revoke_sessions` with username filters correctly
- Integration tests in `apm/tests/integration.rs`: use a mock HTTP server (`wiremock` or `mockito`) to verify request shape and output formatting for all three commands

**Order**: config change → server handlers → server routes → CLI cargo deps → CLI commands → wiring → tests

### Dependencies assumed present

Tickets e2e3d958 and 8a08637c will have landed on `epic/8db73240-user-mgmt` before this ticket is implemented. `apm-server/src/auth.rs` (with `OtpStore`, `SessionStore`, `IsLocalhost`) and `apm-server/src/credential_store.rs` are expected to exist.

### 1. reqwest in CLI

Add to `apm/Cargo.toml`:
```toml
reqwest = { version = "0.11", features = ["blocking", "json"] }
serde_json = "1"
```

### 2. Server URL from config

In `apm-core/src/config.rs`, add an optional `[server]` table:
```rust
#[derive(Deserialize, Default)]
pub struct ServerConfig {
    #[serde(default = "default_server_url")]
    pub url: String,
}
fn default_server_url() -> String { "http://127.0.0.1:3000".to_owned() }
```
Add `pub server: ServerConfig` to the top-level `Config` struct (optional, default).

Add a small helper in `apm/src/cmd/mod.rs` (or a new `apm/src/server_client.rs`):
```rust
pub fn server_url(cfg: &Config) -> &str { &cfg.server.url }
```

### 3. `apm register <username>` — `apm/src/cmd/register.rs`

- POST `{server_url}/api/auth/otp` with `Content-Type: application/json`, body `{"username":"<u>"}`
- On HTTP 200: parse `{"otp":"XXXXXXXX"}`, print the OTP string
- On connection error or non-200: print error to stderr, exit 1

Wire up in `apm/src/main.rs`:
```rust
Register { username: String }
```

### 4. `apm sessions` — `apm/src/cmd/sessions.rs`

- GET `{server_url}/api/auth/sessions`
- Response: `Vec<SessionInfo>` (JSON), where `SessionInfo` has `username`, `device_hint: Option<String>`, `last_seen: DateTime<Utc>`, `expires_at: DateTime<Utc>`
- If empty vec: print "No active sessions."
- Otherwise: print aligned table, e.g.:
  ```
  USERNAME   DEVICE      LAST SEEN             EXPIRES
  alice      MacBook     2026-04-01 14:32 UTC  2026-04-08 14:32 UTC
  bob        iPhone      2026-03-30 09:11 UTC  2026-04-06 09:11 UTC
  ```

### 5. `apm revoke` — `apm/src/cmd/revoke.rs`

Args:
```
apm revoke [<username>] [--device <hint>] [--all]
```
- `--all`: no username required; mutually exclusive with `--device`
- `<username>` required unless `--all` is set (clap validation)

- DELETE `{server_url}/api/auth/sessions` with JSON body:
  ```json
  {"username": "alice", "device": "MacBook", "all": false}
  ```
- Response: `{"revoked": N}`
- Print `"Revoked N session(s)."` or `"No sessions found for <username>."` when N == 0

### 6. New server endpoints — `apm-server/src/auth.rs`

Add alongside existing OTP handlers (the `IsLocalhost` extractor from e2e3d958 enforces localhost-only):

```rust
async fn list_sessions(
    IsLocalhost: IsLocalhost,
    State(state): State<AppState>,
) -> Json<Vec<SessionInfo>>
```
- Reads `state.session_store`, filters out expired sessions, returns `Vec<SessionInfo>` (no tokens exposed)

```rust
async fn revoke_sessions(
    IsLocalhost: IsLocalhost,
    State(state): State<AppState>,
    Json(req): Json<RevokeRequest>,
) -> Json<RevokeResponse>
```
- `RevokeRequest { username: Option<String>, device: Option<String>, all: bool }`
- `RevokeResponse { revoked: usize }`
- Removes matching sessions from the store; if `all == true`, clears everything regardless of other fields

Add new types:
```rust
pub struct SessionInfo {
    pub username: String,
    pub device_hint: Option<String>,
    pub last_seen: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
}
```

### 7. Route registration — `apm-server/src/main.rs`

```rust
.route("/api/auth/sessions", get(list_sessions).delete(revoke_sessions))
```

### 8. Tests

- Integration test in `apm/tests/integration.rs`: spin up a mock HTTP server (use `wiremock` or `mockito`) to verify each CLI command sends the correct request and formats output correctly
- Unit tests in `apm-server/src/auth.rs`: `list_sessions` returns only non-expired sessions; `revoke_sessions` with `all=true` clears the store; `revoke_sessions` with username filters correctly

### Order of implementation

1. `apm-core/src/config.rs` — add `ServerConfig`
2. `apm-server/src/auth.rs` — add `SessionInfo`, `RevokeRequest`, `RevokeResponse`, `list_sessions`, `revoke_sessions`
3. `apm-server/src/main.rs` — register routes
4. `apm/Cargo.toml` — add reqwest + serde_json
5. `apm/src/cmd/register.rs`, `sessions.rs`, `revoke.rs`
6. `apm/src/main.rs` — wire commands
7. Tests

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-04-02T20:54Z | — | new | apm |
| 2026-04-02T23:22Z | new | groomed | apm |
| 2026-04-02T23:59Z | groomed | in_design | philippepascal |
| 2026-04-03T00:05Z | in_design | specd | claude-0402-0000-spec1 |
| 2026-04-04T00:29Z | specd | ready | apm |
| 2026-04-04T03:42Z | ready | in_progress | philippepascal |
| 2026-04-04T03:54Z | in_progress | implemented | claude-0404-0342-7690 |
| 2026-04-04T06:02Z | implemented | closed | apm-sync |
