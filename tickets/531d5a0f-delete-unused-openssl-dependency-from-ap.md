+++
id = "531d5a0f"
title = "Delete unused openssl dependency from apm-server"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/531d5a0f-delete-unused-openssl-dependency-from-ap"
created_at = "2026-04-19T01:23:54.115820Z"
updated_at = "2026-04-19T01:47:54.458575Z"
epic = "7bc3561c"
target_branch = "epic/7bc3561c-trim-dependency-footprint"
+++

## Spec

### Problem

`apm-server` declares `openssl` as a dependency but no source file uses it. `cargo-machete` flags it as unused, and grep for `use openssl` or `openssl::` across `apm-server/src/` returns zero hits. Removing the declaration drops roughly 17 transitive crates and eliminates the native OpenSSL build from CI on platforms that don't already link it.

### Acceptance criteria

- [ ] `apm-server/Cargo.toml` no longer contains an `openssl` entry\n- [ ] `cargo check -p apm-server` succeeds after the removal\n- [ ] `cargo build -p apm-server` succeeds after the removal\n- [ ] `cargo machete` no longer flags `openssl` as unused in `apm-server`\n- [ ] `Cargo.lock` no longer contains entries for `openssl`, `openssl-macros`, `openssl-src`, or `openssl-sys` (assuming no other workspace member pulls them in)

### Out of scope

- Removing or replacing TLS libraries used by other crates (e.g. `rustls`, `tokio-rustls`, `rustls-acme`)\n- Auditing whether other workspace members (`apm`, `apm-core`) have unused dependencies\n- Changing how TLS is handled in `apm-server` (the server already uses rustls)\n- Vendoring or upgrading any remaining dependency

### Approach

How the implementation will work.

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-04-19T01:23Z | — | new | philippepascal |
| 2026-04-19T01:47Z | new | groomed | philippepascal |
| 2026-04-19T01:47Z | groomed | in_design | philippepascal |