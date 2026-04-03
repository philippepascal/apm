+++
id = "48105624"
title = "apm-server: embed apm-ui static assets at build time via include_dir"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "apm"
agent = "52530"
branch = "ticket/48105624-apm-server-embed-apm-ui-static-assets-at"
created_at = "2026-04-02T20:54:40.869103Z"
updated_at = "2026-04-03T00:14:22.830748Z"
epic = "8db73240"
target_branch = "epic/8db73240-user-mgmt"
+++

## Spec

### Problem

apm-server requires a separately running Vite dev server to serve the UI. For distribution as a single binary, the built UI static assets must be embedded in the server at compile time using `include_dir!` or equivalent. Without this, deploying apm-server requires a separate static file deployment step. See `initial_specs/DESIGN-users.md` point 6.

### Acceptance criteria

- [ ] `apm-server` binary serves apm-ui HTML, JS, CSS, and other static assets without requiring `apm-ui/dist` to exist at runtime
- [ ] Requests to paths not matching any embedded file fall back to serving the embedded `index.html` (SPA client-side routing is preserved)
- [ ] Each served file has a correct `Content-Type` header (e.g. `text/html` for `.html`, `application/javascript` for `.js`, `text/css` for `.css`)
- [ ] The `tower-http` `fs` feature is removed from `apm-server/Cargo.toml` (no longer needed for static file serving)
- [ ] `cargo build -p apm-server` succeeds when `apm-ui/dist` exists at build time
- [ ] `cargo build -p apm-server` fails with a clear error when `apm-ui/dist` does not exist at build time

### Out of scope

- Serving assets in development mode (Vite dev server continues to be used for local development)
- Automating the `npm run build` step from within the Rust build (no build.rs UI compilation)
- Asset compression or Brotli/gzip pre-encoding of embedded files
- Cache-control headers, ETags, or conditional GET support
- Any changes to the apm-ui source or build configuration

### Approach

How the implementation will work.

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-04-02T20:54Z | — | new | apm |
| 2026-04-02T23:23Z | new | groomed | apm |
| 2026-04-03T00:14Z | groomed | in_design | philippepascal |