+++
id = "f7225f77"
title = "Inline small helpers for mime_guess, cookie, rustls-pemfile"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/f7225f77-inline-small-helpers-for-mime-guess-cook"
created_at = "2026-04-19T01:24:07.721264Z"
updated_at = "2026-04-19T01:56:08.969845Z"
epic = "7bc3561c"
target_branch = "epic/7bc3561c-trim-dependency-footprint"
+++

## Spec

### Problem

Three small crates are pulled into `apm-server` for narrow, easily-inlined work:

- **`mime_guess`** — one call at `apm-server/src/main.rs:133` that maps a file extension to a MIME type for static-asset serving. The set of extensions the server actually emits is a dozen or so; a fixed `match` table covers it without a crate.
- **`cookie`** — used at `apm-server/src/auth.rs:365` and `apm-server/src/queue.rs:43` to parse and build a single session-cookie header. A ~20-line parser/serializer matches our usage (name, value, `HttpOnly`, `Secure`, `SameSite`, `Max-Age`).
- **`rustls-pemfile`** — used at `apm-server/src/tls.rs:34` and `:37` to read a PEM-encoded certificate chain and private key from disk. The two block types we need are trivial to parse against the PEM RFC.

Each of these crates is small on its own, but together they add to the transitive-dependency count and the compile-time tax without meaningful code savings. Inlining the three helpers removes the crates with no loss of functionality.

### Acceptance criteria

- [ ] `apm-server/Cargo.toml` no longer declares `mime_guess`, `cookie`, or `rustls-pemfile` as dependencies
- [ ] `mime_guess` is removed from the workspace `Cargo.toml` `[workspace.dependencies]` table (it is only used by `apm-server`)
- [ ] `cargo build -p apm-server` succeeds with no compile errors after the removals
- [ ] `cargo build -p apm-server` produces no new warnings introduced by the inlined code
- [ ] The static-asset handler returns `text/html; charset=utf-8` for `.html` paths
- [ ] The static-asset handler returns `application/javascript` for `.js` paths
- [ ] The static-asset handler returns `text/css` for `.css` paths
- [ ] The static-asset handler returns `application/wasm` for `.wasm` paths
- [ ] The static-asset handler returns `application/octet-stream` for an unrecognised extension
- [ ] The session-cookie extractor returns the correct value when a `Cookie` header contains `__Host-apm-session=<token>` among other cookies
- [ ] The session-cookie extractor returns `None` when `__Host-apm-session` is absent from the header
- [ ] `custom_cert_config` successfully loads a valid PEM certificate chain and PKCS#8 private key from disk and returns a `ServerConfig`
- [ ] `custom_cert_config` returns an error when the cert file contains no `CERTIFICATE` PEM blocks
- [ ] `custom_cert_config` returns an error when the key file contains no recognised private-key PEM block

### Out of scope

- Removing or replacing TLS libraries that other crates already depend on (rustls, tokio-rustls, rustls-acme)
- Auditing other unused dependencies in apm-server or any other workspace member (covered by sibling tickets)
- Supporting PEM block types beyond CERTIFICATE, RSA PRIVATE KEY, EC PRIVATE KEY, and PRIVATE KEY (PKCS8)
- Supporting MIME types not produced by the embedded UI assets (the match table covers the set actually served)
- Cookie serialisation / Set-Cookie header building (the server only reads cookies, never writes them)
- Quoted cookie values or encoded cookie names (the session token is a plain alphanumeric string)
- Replacing reqwest, ctrlc, or openssl (covered by sibling tickets 38c93480, 22f539f2, 531d5a0f)

### Approach

How the implementation will work.

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-04-19T01:24Z | — | new | philippepascal |
| 2026-04-19T01:47Z | new | groomed | philippepascal |
| 2026-04-19T01:56Z | groomed | in_design | philippepascal |