+++
id = "48105624"
title = "apm-server: embed apm-ui static assets at build time via include_dir"
state = "closed"
priority = 0
effort = 3
risk = 2
author = "apm"
branch = "ticket/48105624-apm-server-embed-apm-ui-static-assets-at"
created_at = "2026-04-02T20:54:40.869103Z"
updated_at = "2026-04-04T06:20:16.519246Z"
epic = "8db73240"
target_branch = "epic/8db73240-user-mgmt"
+++

## Spec

### Problem

apm-server requires a separately running Vite dev server to serve the UI. For distribution as a single binary, the built UI static assets must be embedded in the server at compile time using `include_dir!` or equivalent. Without this, deploying apm-server requires a separate static file deployment step. See `initial_specs/DESIGN-users.md` point 6.

### Acceptance criteria

- [x] `apm-server` binary serves apm-ui HTML, JS, CSS, and other static assets without requiring `apm-ui/dist` to exist at runtime
- [x] Requests to paths not matching any embedded file fall back to serving the embedded `index.html` (SPA client-side routing is preserved)
- [x] Each served file has a correct `Content-Type` header (e.g. `text/html` for `.html`, `application/javascript` for `.js`, `text/css` for `.css`)
- [x] The `tower-http` `fs` feature is removed from `apm-server/Cargo.toml` (no longer needed for static file serving)
- [x] `cargo build -p apm-server` succeeds when `apm-ui/dist` exists at build time
- [x] `cargo build -p apm-server` fails with a clear error when `apm-ui/dist` does not exist at build time

### Out of scope

- Serving assets in development mode (Vite dev server continues to be used for local development)
- Automating the `npm run build` step from within the Rust build (no build.rs UI compilation)
- Asset compression or Brotli/gzip pre-encoding of embedded files
- Cache-control headers, ETags, or conditional GET support
- Any changes to the apm-ui source or build configuration

### Approach

Add `include_dir` to the workspace and to `apm-server`:

In Cargo.toml root workspace, add: include_dir = "0.7"
In apm-server/Cargo.toml, add: include_dir = { workspace = true }, mime_guess = "2"

Remove the `fs` feature from `tower-http` in `apm-server/Cargo.toml` since filesystem-based static serving is no longer needed.

In `apm-server/src/main.rs`, add a compile-time static using include_dir's include_dir! macro pointing to `$CARGO_MANIFEST_DIR/../apm-ui/dist`. `$CARGO_MANIFEST_DIR` resolves to the `apm-server/` directory at compile time, so the path navigates correctly to `apm-ui/dist` in the workspace root.

Replace the current `ServeDir::new("apm-ui/dist")` / `.nest_service("/", serve_dir)` with a plain Axum fallback handler that:
1. Extracts the URI path, strips the leading `/`
2. Looks up the path in UI_DIR (the embedded include_dir! static)
3. If found, returns the file bytes with a Content-Type derived from the file extension via mime_guess::from_path(...).first_or_octet_stream()
4. If not found, falls back to serving the embedded `index.html` (SPA routing)
5. Registers as `.fallback(serve_ui)` instead of `.nest_service("/", serve_dir)`

Build prerequisite: The include_dir! macro fails at compile time if `apm-ui/dist` does not exist. Developers and CI must run `npm run build` in `apm-ui/` before building `apm-server`. No `build.rs` needed.

Files changed:
- Cargo.toml (root): add `include_dir = "0.7"` to [workspace.dependencies]
- apm-server/Cargo.toml: add `include_dir` and `mime_guess`; remove `fs` feature from `tower-http`
- apm-server/src/main.rs: replace ServeDir/ServeFile imports and usage with UI_DIR static and serve_ui fallback handler

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-04-02T20:54Z | — | new | apm |
| 2026-04-02T23:23Z | new | groomed | apm |
| 2026-04-03T00:14Z | groomed | in_design | philippepascal |
| 2026-04-03T00:17Z | in_design | specd | claude-0402-2015-spec1 |
| 2026-04-04T00:29Z | specd | ready | apm |
| 2026-04-04T02:52Z | ready | in_progress | philippepascal |
| 2026-04-04T02:55Z | in_progress | implemented | claude-0403-0252-w48b |
| 2026-04-04T06:20Z | implemented | closed | apm-sync |
