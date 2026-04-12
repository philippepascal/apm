+++
id = "4ec0a793"
title = "Consolidate auth: move middleware to auth.rs, merge webauthn_state and credential_store"
state = "specd"
priority = 0
effort = 4
risk = 2
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/4ec0a793-consolidate-auth-move-middleware-to-auth"
created_at = "2026-04-12T09:03:28.810627Z"
updated_at = "2026-04-12T10:01:50.305444Z"
epic = "1e706443"
target_branch = "epic/1e706443-refactor-apm-server-code-organization"
depends_on = ["9698c4c6"]
+++

## Spec

### Problem

Auth-related code in `apm-server` is scattered across `main.rs` and three separate modules, making it hard to navigate and maintain. Specifically:

- `main.rs` contains ~370 lines of auth logic that belong elsewhere: the `require_auth()` axum middleware (~20 lines, ~line 1406), the `find_session_username()` helper (~15 lines, ~line 1391), OTP handlers (~100 lines), WebAuthn registration/login handlers (~200 lines), and session management handlers (~50 lines).
- `auth.rs` (361 lines) already holds OTP generation, session storage, and token verification -- the natural home for all the above.
- `webauthn_state.rs` (66 lines) and `credential_store.rs` (134 lines) are standalone modules that exist solely to support auth; both are too small to warrant separate files and are tightly coupled to the rest of the auth subsystem.

The desired state is a single `auth.rs` that owns all auth concerns: session management, OTP, WebAuthn state, credential persistence, middleware, and handlers. `webauthn_state.rs` and `credential_store.rs` are deleted. `main.rs` shrinks by ~370 lines and all auth handler references in the router point to `auth::`.

### Acceptance criteria

- [ ] `webauthn_state.rs` no longer exists as a source file
- [ ] `credential_store.rs` no longer exists as a source file
- [ ] `auth.rs` contains `WebauthnState` with `RegistrationSession`, `AuthenticationSession` structs (previously in `webauthn_state.rs`)
- [ ] `auth.rs` contains `CredentialStore` with all passkey persistence methods (previously in `credential_store.rs`)
- [ ] `auth.rs` exports a public `require_auth` middleware function (previously in `main.rs`)
- [ ] `auth.rs` exports a public `find_session_username` helper function (previously in `main.rs`)
- [ ] `auth.rs` exports public handler functions: `otp_handler`, `register_page_handler`, `register_challenge_handler`, `register_complete_handler`, `login_page_handler`, `login_challenge_handler`, `login_complete_handler`, `list_sessions_handler`, `revoke_sessions_handler`
- [ ] `main.rs` contains no `mod webauthn_state;` or `mod credential_store;` declarations
- [ ] `main.rs` contains none of the handler functions or middleware listed above
- [ ] `AppState` in `main.rs` references `auth::WebauthnState` and `auth::CredentialStore` (instead of `webauthn_state::WebauthnState` and `credential_store::CredentialStore`)
- [ ] The router in `main.rs` calls all auth handlers via the `auth::` namespace
- [ ] `cargo build` succeeds with no errors or warnings
- [ ] `cargo test` passes, including unit tests migrated from `webauthn_state.rs` and `credential_store.rs`

### Out of scope

- Changing any auth handler behaviour (request/response shapes, route paths, cookie names, TTLs)
- Adding new auth functionality (e.g. new login methods, rate limiting)
- Moving `AppState` to a separate module
- Splitting `auth.rs` into submodules (e.g. `auth/webauthn.rs`)
- Any changes to the WebAuthn or credential logic itself
- Updating integration tests or end-to-end tests beyond what is needed to keep them compiling

### Approach

This is a pure code-movement refactor -- no behaviour changes. Work in the worktree for branch `ticket/4ec0a793-consolidate-auth-move-middleware-to-auth`, which targets `epic/1e706443-refactor-apm-server-code-organization`.

**File: `apm-server/src/auth.rs`** -- absorbs everything

1. Append the full contents of `webauthn_state.rs` verbatim (structs, impls, tests). Add any missing `use` imports (`webauthn_rs` types, `std::time::Instant`, etc.) that were only present in that file.
2. Append the full contents of `credential_store.rs` verbatim (struct, impls, tests). Add any missing `use` imports (`webauthn_rs::prelude::{Passkey, AuthenticationResult}`, `serde_json`, etc.).
3. Add the `find_session_username` function from `main.rs` (~line 1391). Its signature stays the same; it references `crate::AppState` through its `State` extractor.
4. Add the `require_auth` middleware from `main.rs` (~line 1406). Same logic, same signature -- just now lives in `auth.rs` and uses `crate::AppState`.
5. Add all auth handler functions from `main.rs` (lines ~1429--1720): `otp_handler`, `register_page_handler`, `register_challenge_handler`, `register_complete_handler`, `login_page_handler`, `login_challenge_handler`, `login_complete_handler`, `list_sessions_handler`, `revoke_sessions_handler`. Each one takes `State<Arc<crate::AppState>>` -- `crate::` is the standard way for a submodule to reference the parent crate's types. Mark all functions `pub`.
6. Add any `use` statements the handlers need that are not already present (axum extractors, `http` types, etc.).

**File: `apm-server/src/main.rs`**

1. Remove `mod webauthn_state;` and `mod credential_store;` declarations.
2. Remove the `find_session_username` and `require_auth` function bodies.
3. Remove all nine auth handler function bodies.
4. In `AppState`, change field types:
   - `webauthn_state: Arc<webauthn_state::WebauthnState>` -> `webauthn_state: Arc<auth::WebauthnState>`
   - `credential_store: credential_store::CredentialStore` -> `credential_store: auth::CredentialStore`
5. Remove any `use webauthn_state::...` or `use credential_store::...` imports that are now dead.
6. In the router, replace bare function names with `auth::`-prefixed names everywhere auth handlers appear (both the `route_layer` reference for `require_auth` and the individual `.route()` calls for all nine handlers).
7. Any `AppState` construction sites that call `WebauthnState::new` or `CredentialStore::load` now call `auth::WebauthnState::new` and `auth::CredentialStore::load`.

**Files: `apm-server/src/webauthn_state.rs` and `apm-server/src/credential_store.rs`** -- delete both.

**Order of steps** (prevents intermediate compile failures):
1. Copy code into `auth.rs` and get it compiling (imports first, then structs, then handlers).
2. Update `main.rs` (remove dead code, fix type references, fix router).
3. Delete the two now-empty source files.
4. Run `cargo test` to confirm all tests pass including the migrated ones.

**Known constraint:** Because handlers in `auth.rs` reference `crate::AppState`, and `main.rs` references `auth::*` types, there is a mutual reference between the two files. This is normal in Rust (submodules reference the crate root freely) and does not create a circular dependency -- the compiler resolves it without issue.

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-04-12T09:03Z | — | new | philippepascal |
| 2026-04-12T09:10Z | new | groomed | apm |
| 2026-04-12T09:57Z | groomed | in_design | philippepascal |
| 2026-04-12T10:01Z | in_design | specd | claude-0412-0957-a030 |
