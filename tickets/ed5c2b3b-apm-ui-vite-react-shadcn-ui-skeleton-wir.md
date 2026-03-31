+++
id = "ed5c2b3b"
title = "apm-ui: Vite + React + shadcn/ui skeleton wired to backend"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "apm"
agent = "98941"
branch = "ticket/ed5c2b3b-apm-ui-vite-react-shadcn-ui-skeleton-wir"
created_at = "2026-03-31T06:11:40.599936Z"
updated_at = "2026-03-31T06:16:51.666808Z"
+++

## Spec

### Problem

There is no frontend. The backend steps (Steps 1 and 2) will deliver an axum server on port 3000 with `GET /health` and `GET /api/tickets`, but there is nothing to load in a browser. This ticket creates the `apm-ui/` directory with a Vite + React + TypeScript + shadcn/ui scaffold, wires TanStack Query to make one `useQuery` call to `/api/tickets` that logs results to the browser console, and configures the axum server to serve the built static files at `GET /`. The page is intentionally blank. The goal is to prove the full stack — React build → axum static serving → API fetch → console output — wires together correctly before any UI is built on top of it.

### Acceptance criteria

- [ ] `npm run build` in `apm-ui/` exits 0 with no TypeScript errors
- [ ] `cargo run -p apm-server` (from repo root, after building the UI) serves `GET /` with HTTP 200 and `Content-Type: text/html`
- [ ] Loading `http://localhost:3000/` in a browser renders a blank page with no visible content
- [ ] After the page loads, the browser devtools console shows the array returned by `/api/tickets`
- [ ] If `/api/tickets` returns a non-2xx response or a network error, the console shows the error but the page does not throw an unhandled exception
- [ ] `cargo test --workspace` passes after the static-serving changes are added to apm-server
- [ ] `apm-ui/.gitignore` excludes `node_modules/` and `dist/`

### Out of scope

Explicit list of what this ticket does not cover.

### Approach

How the implementation will work.

### Open questions



### Amendment requests



## History

| When | From | To | By |
|------|------|----|----|
| 2026-03-31T06:11Z | — | new | apm |
| 2026-03-31T06:16Z | new | in_design | philippepascal |