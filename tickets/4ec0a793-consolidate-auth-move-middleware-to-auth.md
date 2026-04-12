+++
id = "4ec0a793"
title = "Consolidate auth: move middleware to auth.rs, merge webauthn_state and credential_store"
state = "new"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/4ec0a793-consolidate-auth-move-middleware-to-auth"
created_at = "2026-04-12T09:03:28.810627Z"
updated_at = "2026-04-12T09:03:28.810627Z"
epic = "1e706443"
target_branch = "epic/1e706443-refactor-apm-server-code-organization"
depends_on = ["9698c4c6"]
+++

## Spec

### Problem

Auth-related code in `apm-server` is split across main.rs and three small modules, with some logic in the wrong place:

1. **In main.rs (should be in auth.rs):**
   - `require_auth()` middleware (~20 lines, line ~1406) — checks session cookie, returns 401 if invalid
   - `find_session_username()` (~15 lines, line ~1391) — looks up username from session cookie
   - OTP login/verify handlers (~100 lines)
   - WebAuthn registration/login handlers (~200 lines)
   - Session management handlers (~50 lines)

2. **Already in auth.rs (361 lines):** OTP generation, session store, token verification

3. **Tiny standalone modules:**
   - `webauthn_state.rs` (66 lines) — just a `WebAuthnState` struct holding pending registration/auth state
   - `credential_store.rs` (134 lines) — passkey credential persistence

The middleware and session helpers in main.rs should move to auth.rs. The two tiny modules (`webauthn_state.rs`, `credential_store.rs`) should be consolidated into auth.rs since they're tightly coupled and too small to justify separate files.

This ticket depends on the previous handler extractions to avoid main.rs merge conflicts.

### Acceptance criteria

Checkboxes; each one independently testable.

### Out of scope

Explicit list of what this ticket does not cover.

### Approach

How the implementation will work.

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-04-12T09:03Z | — | new | philippepascal |