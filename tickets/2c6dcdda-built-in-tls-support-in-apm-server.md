+++
id = "2c6dcdda"
title = "Built-in TLS support in apm-server"
state = "in_design"
priority = 0
effort = 6
risk = 5
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/2c6dcdda-built-in-tls-support-in-apm-server"
created_at = "2026-04-06T06:09:24.235043Z"
updated_at = "2026-04-06T06:24:45.469645Z"
+++

## Spec

### Problem

apm-server currently runs plain HTTP only. Production deployments require a separate reverse proxy (nginx via apm-proxy Docker image) for TLS termination. This adds operational complexity — users need Docker, a separate component, and a different mental model — undermining APM's single-binary simplicity.

The goal is to make production HTTPS as easy as `apm-server --tls --domain=apm.example.com --email=you@example.com`, with automatic certificate management built into the binary itself.

### Acceptance criteria

- [ ] `--tls` flag enables HTTPS on port 443 with automatic Let's Encrypt via rustls-acme (TLS-ALPN-01 challenge)
- [ ] `--tls-domain <domain>` and `--tls-email <email>` configure the ACME certificate request
- [ ] Certificates are cached to `~/.apm/certs/` (or configurable path via `--tls-cert-dir`) and survive restarts without re-issuing
- [ ] Automatic renewal before expiry (background task)
- [ ] `--tls-cert <path> --tls-key <path>` allows using a custom certificate instead of Let's Encrypt (e.g. corporate CA, wildcard cert)
- [ ] `--tls=self-signed` generates a self-signed certificate for local development/testing (no internet required)
- [ ] HSTS header (Strict-Transport-Security) is set on all HTTPS responses
- [ ] Without `--tls`, apm-server runs plain HTTP on port 3000 as today — no behavior change
- [ ] `--port <port>` overrides the default listening port (3000 for HTTP, 443 for HTTPS)
- [ ] `--bind <addr>` configures the bind address (defaults to `0.0.0.0`)
- [ ] apm-proxy Docker setup remains functional but is documented as optional/legacy

### Out of scope

- mTLS / client certificate authentication
- OCSP stapling or certificate revocation lists
- Multiple domains or Subject Alternative Names per certificate
- HTTP/2-specific configuration (comes automatically with rustls but not explicitly configured)
- Removing or replacing the apm-proxy Docker image (it remains, documented as optional/legacy)
- Integration tests that contact Let's Encrypt staging or production (network-dependent; testing uses self-signed path)
- HTTP-to-HTTPS redirect (apm-server listens on a single port only)

### Approach

**Dependencies to add (apm-server/Cargo.toml)**

- clap — workspace dep already defined; add to apm-server deps
- rustls-acme (version 0.11, features [axum]) — Let's Encrypt via TLS-ALPN-01, handles renewal automatically
- rcgen (version 0.13) — self-signed certificate generation
- tokio-rustls (version 0.26) — TLS stream wrapping for custom and self-signed modes
- rustls (version 0.23) — ServerConfig construction
- Enable the set-header feature on the existing tower-http dep

**CLI argument parsing**

Add a clap::Parser struct to main.rs (or a dedicated cli.rs module).

Fields:
- --tls: Option<TlsMode> with default_missing_value="acme" and num_args 0..=1 (--tls alone means ACME; --tls=self-signed means self-signed)
- --tls-domain: Option<String>
- --tls-email: Option<String>
- --tls-cert-dir: Option<PathBuf>
- --tls-cert: Option<PathBuf> (requires tls-key)
- --tls-key: Option<PathBuf> (requires tls-cert)
- --port: Option<u16> — overrides the default port; defaults to 3000 in HTTP mode, 443 in any TLS mode
- --bind: Option<String> — bind address; defaults to "0.0.0.0"

TlsMode is a clap::ValueEnum with variants: Acme, SelfSigned

**Mode dispatch in main()**

Resolve bind address and port, then branch on (cli.tls, cli.tls_cert):
- (None, None) -> plain HTTP; addr = `{bind}:{port}` where port defaults to 3000
- (None, Some(_)) -> custom cert mode; addr = `{bind}:{port}` where port defaults to 443
- (Some(Acme), _) -> Let's Encrypt mode; addr = `{bind}:{port}` where port defaults to 443
- (Some(SelfSigned), _) -> self-signed mode; addr = `{bind}:{port}` where port defaults to 443

Bind address: `format!("{}:{}", cli.bind.as_deref().unwrap_or("0.0.0.0"), port)`

**Plain HTTP (unchanged)**

`tokio::net::TcpListener::bind(addr)` plus `axum::serve()` — addr resolved from --bind and --port as above.

**Let's Encrypt (ACME) mode**

1. Require --tls-domain and --tls-email; exit with clear error if missing.
2. Cert cache dir: cli.tls_cert_dir or ~/.apm/certs/ (expand via std::env::var("HOME")).
3. Build rustls_acme::AcmeConfig with domain, email, cache dir, production ACME URL.
4. rustls_acme manages a rustls::ServerConfig and handles renewal; wrap with tokio_rustls::TlsAcceptor.
5. Bind the resolved addr, accept raw TCP, wrap each stream in TlsAcceptor, serve axum with the HSTS router.

**Self-signed mode**

1. rcgen::generate_simple_self_signed(vec![domain_or_localhost])
2. Convert to rustls pki_types (CertificateDer and PrivateKeyDer)
3. Build rustls::ServerConfig from the cert+key.
4. Wrap listener on the resolved addr with tokio_rustls::TlsAcceptor.
5. Print a startup warning that the cert is self-signed and untrusted by browsers.
6. Same HSTS setup as ACME mode.

**Custom certificate mode**

1. Read PEM files from --tls-cert and --tls-key paths.
2. Parse with rustls_pemfile (pulled in transitively by rustls).
3. Build rustls::ServerConfig with the loaded cert chain and private key.
4. Wrap listener on the resolved addr with tokio_rustls::TlsAcceptor.
5. Same HSTS setup.

**HSTS middleware**

Use tower_http::set_header::SetResponseHeaderLayer to inject `Strict-Transport-Security: max-age=63072000; includeSubDomains` on every response when TLS is enabled. Applied as a layer on the axum router before passing to axum::serve(). Only applied in TLS modes.

**File layout**

- All new code goes into apm-server/src/main.rs; extract tls.rs module if it grows large.
- apm-proxy/ is untouched; add a note in the docs directory marking it as optional/legacy.

**Order of implementation**

1. Add deps to apm-server/Cargo.toml
2. Add Cli struct and main() arg parsing (including --port and --bind)
3. Verify existing plain-HTTP path and all tests still pass
4. Implement custom cert mode (easiest to test locally)
5. Implement self-signed mode (rcgen; test with curl --insecure)
6. Implement ACME mode (wired but not integration-tested against LE)
7. Add HSTS layer
8. Run cargo test --workspace

### Open questions


### Amendment requests

- [x] Add `--port <port>` flag to configure the listening port (defaults to 3000 for HTTP, 443 for HTTPS)
- [x] Add `--bind <addr>` flag to configure the bind address (defaults to `0.0.0.0`)
- [x] Remove HTTP→HTTPS redirect on port 80 — apm-server listens on a single port only
- [x] Remove "Changing the default plain-HTTP port (stays 3000)" from Out of scope (now in scope)
- [ ] Update Approach to use configurable port/bind instead of hardcoded values

### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-04-06T06:09Z | — | new | philippepascal |
| 2026-04-06T06:12Z | new | groomed | apm |
| 2026-04-06T06:12Z | groomed | in_design | philippepascal |
| 2026-04-06T06:17Z | in_design | specd | claude-0406-0612-s7w2 |
| 2026-04-06T06:19Z | specd | ammend | philippe |
| 2026-04-06T06:20Z | ammend | in_design | philippepascal |
| 2026-04-06T06:22Z | in_design | specd | claude-0406-0621-spec1 |
| 2026-04-06T06:24Z | specd | ammend | apm |
| 2026-04-06T06:24Z | ammend | in_design | philippepascal |