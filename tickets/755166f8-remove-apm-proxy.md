+++
id = "755166f8"
title = "remove apm-proxy"
state = "specd"
priority = 0
effort = 1
risk = 1
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/755166f8-remove-apm-proxy"
created_at = "2026-04-06T20:50:54.696634Z"
updated_at = "2026-04-06T21:42:22.582717Z"
+++

## Spec

### Problem

apm-proxy is a Docker-based Nginx reverse proxy added to provide TLS termination for apm-server. It supports two modes: automatic certificate provisioning via ACME (certbot) and self-signed certificates for local testing.

apm-server now has native TLS built in, implemented in apm-server/src/tls.rs, covering all the same scenarios: --tls=self-signed for local testing, --tls with --tls-domain/--tls-email for automatic ACME, and --tls-cert/--tls-key for custom certificates. The apm-proxy README already marks it as optional/legacy.

Keeping apm-proxy in the repository creates unnecessary maintenance surface, misleads users about which TLS path to choose, and leaves dead code that no release pipeline builds or tests.

### Acceptance criteria

- [ ] The apm-proxy/ directory no longer exists in the repository
- [ ] scripts/proxy-build.sh no longer exists in the repository
- [ ] scripts/proxy-deploy.sh no longer exists in the repository
- [ ] scripts/proxy-local.sh no longer exists in the repository
- [ ] No file in the repository (excluding the ticket file itself) contains the string apm-proxy
- [ ] cargo build --workspace succeeds after the removal
- [ ] cargo test --workspace passes after the removal

### Out of scope

- Changes to apm-server TLS implementation (it already covers all scenarios apm-proxy handled)
- Updating user-owned deployment files outside this repository
- Publishing migration guidance to external channels
- Changes to docs/external-tls-setup.md -- it already documents native TLS and does not reference apm-proxy

### Approach

This is a pure deletion with no code changes required. apm-proxy is a standalone Docker artifact: it is not a Cargo workspace member, not imported by any crate, and not referenced by .github/workflows/release.yml.

Files to delete:
1. apm-proxy/ -- entire directory (Dockerfile, nginx.conf, nginx.conf.template, entrypoint.sh, README.md)
2. scripts/proxy-build.sh
3. scripts/proxy-deploy.sh
4. scripts/proxy-local.sh

Implementation steps:
1. git rm -r apm-proxy/ scripts/proxy-build.sh scripts/proxy-deploy.sh scripts/proxy-local.sh
2. Verify no remaining references: grep -r apm-proxy . --exclude-dir=.git -- expect zero matches outside the ticket file
3. cargo test --workspace
4. Commit: chore: remove apm-proxy (redundant since apm-server has native TLS)

No changes are needed anywhere in the Rust workspace, documentation, or CI configuration.

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|---
| 2026-04-06T20:50Z | — | new | philippepascal |
| 2026-04-06T21:22Z | new | groomed | apm |
| 2026-04-06T21:30Z | groomed | in_design | philippepascal |
| 2026-04-06T21:42Z | in_design | specd | claude-0406-2130-eda0 |
