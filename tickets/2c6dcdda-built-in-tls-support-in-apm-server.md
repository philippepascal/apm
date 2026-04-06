+++
id = "2c6dcdda"
title = "Built-in TLS support in apm-server"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/2c6dcdda-built-in-tls-support-in-apm-server"
created_at = "2026-04-06T06:09:24.235043Z"
updated_at = "2026-04-06T06:12:49.250564Z"
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
- [ ] When TLS is enabled, port 80 listens and redirects all requests to HTTPS (301)
- [ ] HSTS header (Strict-Transport-Security) is set on all HTTPS responses
- [ ] Without `--tls`, apm-server runs plain HTTP on port 3000 as today — no behavior change
- [ ] apm-proxy Docker setup remains functional but is documented as optional/legacy

### Out of scope

- mTLS / client certificate authentication
- OCSP stapling or certificate revocation lists
- Multiple domains or Subject Alternative Names per certificate
- HTTP/2-specific configuration (comes automatically with rustls but not explicitly configured)
- Changing the default plain-HTTP port (stays 3000)
- Removing or replacing the apm-proxy Docker image (it remains, documented as optional/legacy)
- Integration tests that contact Let's Encrypt staging or production (network-dependent; testing uses self-signed path)

### Approach

How the implementation will work.

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-04-06T06:09Z | — | new | philippepascal |
| 2026-04-06T06:12Z | new | groomed | apm |
| 2026-04-06T06:12Z | groomed | in_design | philippepascal |