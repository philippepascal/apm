+++
id = "f7225f77"
title = "Inline small helpers for mime_guess, cookie, rustls-pemfile"
state = "groomed"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/f7225f77-inline-small-helpers-for-mime-guess-cook"
created_at = "2026-04-19T01:24:07.721264Z"
updated_at = "2026-04-19T01:47:36.183162Z"
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
| 2026-04-19T01:24Z | — | new | philippepascal |
| 2026-04-19T01:47Z | new | groomed | philippepascal |
