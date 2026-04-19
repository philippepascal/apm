+++
id = "38c93480"
title = "Swap reqwest for ureq to shed transitive dependencies"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/38c93480-swap-reqwest-for-ureq-to-shed-transitive"
created_at = "2026-04-19T01:24:03.141769Z"
updated_at = "2026-04-19T01:52:17.764037Z"
epic = "7bc3561c"
target_branch = "epic/7bc3561c-trim-dependency-footprint"
+++

## Spec

### Problem

reqwest is one of the largest contributors to the workspace's transitive dependency footprint (pulling in hyper, tokio-native-tls, mio, and their chains — roughly 200 crates). Only five call-sites use it, and every one constructs a plain reqwest::blocking::Client::new() and issues a single HTTP request: apm/src/cmd/register.rs, apm/src/cmd/sessions.rs, apm/src/cmd/revoke.rs, and apm-core/src/github.rs (two calls). No streaming, multipart, cookie-jar, or async features are in use. ureq covers the same blocking request/response shape with a minimal transitive graph. Swapping the five call-sites is the largest single dependency reduction available in this epic.

### Acceptance criteria

- [ ] `cargo build -p apm -p apm-core` succeeds with no errors after the swap
- [ ] `reqwest` no longer appears in `cargo tree -p apm` or `cargo tree -p apm-core`
- [ ] `ureq` appears in `cargo tree -p apm` and `cargo tree -p apm-core`
- [ ] `apm register` issues a POST to apm-server and handles success and error responses correctly
- [ ] `apm sessions` issues a GET to apm-server and prints session list on success
- [ ] `apm revoke` issues a DELETE to apm-server and handles success and error responses correctly
- [ ] GitHub `fetch_authenticated_user()` issues a GET to `https://api.github.com/user` with `Authorization`, `Accept`, and `User-Agent` headers and returns parsed JSON on 200
- [ ] GitHub `fetch_repo_collaborators()` issues a GET to the collaborators URL with the same three headers and returns parsed JSON on 200
- [ ] A non-2xx response from apm-server in any of the three `apm/` commands surfaces a human-readable error (not a panic)
- [ ] A non-2xx response from the GitHub API in either `apm-core` functions surfaces an `anyhow::Error` with context
- [ ] `cargo test --workspace` passes (no regressions in existing tests)

### Out of scope

- Replacing reqwest in apm-server (apm-server uses reqwest for different purposes and is covered by sibling tickets in the epic)\n- Connection pooling or agent reuse across requests (each existing call-site creates a fresh client; ureq free functions match this behaviour)\n- Async HTTP anywhere in the workspace\n- Timeout configuration or retry logic (neither exists today)\n- Windows-specific testing of the new signal handling (not related to this ticket)\n- Auditing other unused dependencies in the workspace (covered by sibling tickets)

### Approach

### Cargo.toml changes

**`Cargo.toml` (workspace root):**
- Remove `reqwest = { version = "0.12", ... }` from `[workspace.dependencies]`
- Add `ureq = { version = "2", features = ["json"] }`

**`apm/Cargo.toml`:**
- Replace `reqwest = { workspace = true }` with `ureq = { workspace = true }`

**`apm-core/Cargo.toml`:**
- Replace `reqwest = { workspace = true }` with `ureq = { workspace = true }`

---

### API mapping reference

| reqwest | ureq |
|---------|------|
| `use reqwest::blocking::Client;` / `Client::new()` | removed — call ureq free functions directly |
| `client.get(url)` | `ureq::get(url)` |
| `client.post(url)` | `ureq::post(url)` |
| `client.delete(url)` | `ureq::delete(url)` |
| `.header("Key", "val")` | `.set("Key", "val")` |
| `.json(&body).send()` | `.send_json(&body)` |
| `.send()` (no body) | `.call()` |
| `resp.json::<T>()` | `resp.into_json::<T>()` |
| `.error_for_status()` | not needed — ureq errors on non-2xx automatically |
| `resp.status().is_success()` | replace with match on Result (see per-file detail) |

**Important:** ureq returns `Err(ureq::Error::Status(code, response))` for non-2xx HTTP responses. The `is_success()` guard pattern in the three `apm/` call-sites must be restructured into a match.

---

### `apm/src/cmd/register.rs`

- Remove `use reqwest::blocking::Client;`
- Remove the `let client = Client::new();` line
- Replace the send + status-check + json block with a match on `ureq::post(&url).send_json(&body)`:
  - `Ok(resp)` branch: call `resp.into_json::<serde_json::Value>()` and continue existing success-path logic
  - `Err(ureq::Error::Status(code, resp))` branch: surface the HTTP status code and body text as an error
  - `Err(e)` branch: return `anyhow::anyhow!("error: cannot connect to apm-server: {e}")`

---

### `apm/src/cmd/sessions.rs`

- Remove `use reqwest::blocking::Client;` and `let client = Client::new();`
- Replace `client.get(&url).send()` with `ureq::get(&url).call()`
- Same three-arm match: Ok calls `.into_json::<Vec<SessionInfo>>()`, status-error surfaces code, transport-error maps to connect message

---

### `apm/src/cmd/revoke.rs`

- Remove `use reqwest::blocking::Client;` and `let client = Client::new();`
- Replace `client.delete(&url).json(&body).send()` with `ureq::delete(&url).send_json(&body)`
- Same three-arm match as register/sessions

---

### `apm-core/src/github.rs` — both functions

Both `fetch_authenticated_user()` and `fetch_repo_collaborators()` follow the same pattern. For each:

- Remove `use reqwest::blocking::Client;` and `let client = Client::new();`
- Replace the header chain: `.header("Key", "val")` becomes `.set("Key", "val")`
- Replace `.send()` with `.call()`
- Remove `.error_for_status().context(...)` — ureq already errors on non-2xx; the `.context("GitHub API request failed")?` on `.call()` is sufficient
- Replace `.json().context(...)` with `.into_json().context("GitHub API response is not valid JSON")?`

The resulting chain for each function:
```
ureq::get(&url)
    .set("Authorization", &format!("Bearer {token}"))
    .set("Accept", "application/vnd.github+json")
    .set("User-Agent", "apm")
    .call()
    .context("GitHub API request failed")?
    .into_json()
    .context("GitHub API response is not valid JSON")?
```

---

### Verification steps

1. `cargo build -p apm -p apm-core` — must compile clean
2. `cargo tree -p apm | grep reqwest` — must produce no output
3. `cargo tree -p apm | grep ureq` — must show ureq in the tree
4. `cargo test --workspace` — must pass

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-04-19T01:24Z | — | new | philippepascal |
| 2026-04-19T01:47Z | new | groomed | philippepascal |
| 2026-04-19T01:52Z | groomed | in_design | philippepascal |