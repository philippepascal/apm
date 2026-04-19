+++
id = "f7225f77"
title = "Inline small helpers for mime_guess, cookie, rustls-pemfile"
state = "in_design"
priority = 0
effort = 4
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/f7225f77-inline-small-helpers-for-mime-guess-cook"
created_at = "2026-04-19T01:24:07.721264Z"
updated_at = "2026-04-19T02:03:17.154242Z"
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

Replace three crates in apm-server with small inline helpers.

Files changed: apm-server/src/main.rs, apm-server/src/auth.rs, apm-server/src/queue.rs,
apm-server/src/tls.rs (new: apm-server/src/pem.rs), apm-server/Cargo.toml, root Cargo.toml.

mime_guess -> apm-server/src/main.rs

Add private fn mime_for_path(path: &str) -> &static str above serve_ui. Match on
path.rsplit_once(dot).map(|(_, ext)| ext): html/htm -> text/html; charset=utf-8,
js/mjs -> application/javascript, css -> text/css, wasm -> application/wasm,
json/map -> application/json, svg -> image/svg+xml, png -> image/png,
ico -> image/x-icon, txt -> text/plain; charset=utf-8, _ -> application/octet-stream.

In serve_ui replace mime_guess::from_path(path).first_or_octet_stream() with
mime_for_path(path) and remove .as_ref() (already &static str).
Remove mime_guess from apm-server/Cargo.toml and [workspace.dependencies] in root Cargo.toml.

cookie -> apm-server/src/auth.rs and apm-server/src/queue.rs

Add private fn cookie_pair(s: &str) -> Option<(&str, &str)> to each file:
trim s; find first = with .find(=); return (s[..eq].trim(), s[eq+1..].trim()).
Using .find rather than .split_once preserves = chars inside values.

In find_session_username (auth.rs) replace:
  if let Ok(c) = cookie::Cookie::parse(part.trim().to_owned()) {
      if c.name() == "__Host-apm-session" { return session_store.lookup(c.value()); }
  }
with:
  if let Some((name, value)) = cookie_pair(part) {
      if name == "__Host-apm-session" { return session_store.lookup(value); }
  }
Apply identical replacement to queue.rs.
Remove cookie imports and cookie from apm-server/Cargo.toml.

rustls-pemfile -> new apm-server/src/pem.rs

Create pem.rs; declare mod pem in main.rs.

Public API:
  pub fn parse_certs(bytes: &[u8]) -> Result<Vec<CertificateDer<static>>>
  pub fn parse_private_key(bytes: &[u8]) -> Result<Option<PrivateKeyDer<static>>>
Types from rustls::pki_types (already a dep via rustls).

Private pem_blocks(bytes: &[u8]) -> Vec<(String, Vec<u8>)>:
- UTF-8 decode bytes (lossy).
- State machine over lines: BEGIN marker -> record label, clear body;
  body line -> append stripped text; END marker -> base64_decode body, push (label, bytes).

Private base64_decode(s: &str) -> Result<Vec<u8>>:
- const [u8; 256] decode table from RFC 4648 standard alphabet (0xFF = invalid).
- Skip whitespace during iteration.
- Four base64 chars -> three output bytes; handle one or two trailing = padding chars.
- Return error on any out-of-alphabet character.

parse_certs: keep blocks labelled CERTIFICATE; wrap each Vec<u8> in CertificateDer::from.

parse_private_key: first block matching:
  PRIVATE KEY     -> PrivateKeyDer::Pkcs8(der.into())
  RSA PRIVATE KEY -> PrivateKeyDer::Pkcs1(der.into())
  EC PRIVATE KEY  -> PrivateKeyDer::Sec1(der.into())
Return Ok(None) if none found.

In tls.rs replace the two rustls_pemfile:: calls with crate::pem::parse_certs and
crate::pem::parse_private_key. Error propagation and .context() calls stay identical.
Remove use rustls_pemfile and rustls-pemfile from apm-server/Cargo.toml.

Order:
1. Create pem.rs with unit tests for base64_decode and pem_blocks; wire mod pem.
2. Patch tls.rs; remove rustls-pemfile from Cargo.toml.
3. Add mime_for_path to main.rs; remove mime_guess from apm-server and root Cargo.toml.
4. Add cookie_pair to auth.rs and queue.rs; remove cookie from Cargo.toml.
5. cargo build -p apm-server to confirm clean compile with no new warnings.

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-04-19T01:24Z | — | new | philippepascal |
| 2026-04-19T01:47Z | new | groomed | philippepascal |
| 2026-04-19T01:56Z | groomed | in_design | philippepascal |