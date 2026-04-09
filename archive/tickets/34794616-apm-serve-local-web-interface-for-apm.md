+++
id = "34794616"
title = "apm serve: local web interface for APM"
state = "closed"
priority = 0
effort = 7
risk = 4
author = "claude-0330-0245-main"
agent = "92477"
branch = "ticket/34794616-apm-serve-local-web-interface-for-apm"
created_at = "2026-03-30T06:46:19.342367Z"
updated_at = "2026-03-30T18:09:23.097640Z"
+++

## Spec

### Problem

APM's supervisor workflow is entirely CLI-based. Reviewing specs, approving state
transitions, monitoring workers, and triggering dispatch all require terminal
access and knowledge of apm commands. This creates constant context-switching
and makes async supervision (stepping away while workers run, then coming back
to review) harder than it needs to be.

There is no way to get a high-level picture of the project state at a glance,
no way to tail a worker's output without finding the log path manually, and no
way to interact with APM from a device that doesn't have the repo cloned.

The end goal is a remote-capable interface: a Linux server hosts the repo,
build toolchain, and apm workers; the supervisor interacts entirely through a
browser. `apm serve` v1 targets local use, but every design decision must keep
the remote path open.

### Acceptance criteria

**Server**
- [ ] `apm serve` starts an HTTP server; default port 3131, configurable via
  `--port` flag and `serve.port` in `apm.toml`
- [ ] Bind address defaults to `127.0.0.1`; configurable via `--bind` flag and
  `serve.bind` in `apm.toml` (set to `0.0.0.0` for remote access)
- [ ] Auth is disabled by default (`serve.auth = false`); when enabled
  (`serve.auth = true`), all routes require a bearer token set via
  `serve.token` in `apm.toml` or `APM_SERVE_TOKEN` env var
- [ ] Server prints the URL on startup: `apm serve listening on http://127.0.0.1:3131`
- [ ] All state-mutating actions go through the HTTP API — no direct in-process
  function calls from the frontend layer

**Kanban board**
- [ ] `GET /` renders a kanban board: one column per workflow state, each
  ticket shown as a card with ID, title, agent, and effort
- [ ] Board auto-refreshes every 30 seconds (or on SSE event)
- [ ] Tickets are sorted within each column by score (same as `apm next`)

**Ticket detail**
- [ ] `GET /tickets/:id` renders the full ticket: frontmatter, spec sections,
  and history table
- [ ] State transition buttons are shown for valid next states; clicking one
  calls the API and redirects back to the ticket

**Worker status**
- [ ] `GET /workers` lists all tickets currently `in_progress` with their
  agent name, elapsed time, and a link to their log
- [ ] `GET /workers/:id/log` streams the worker's `.apm-worker.log` via SSE
  (`text/event-stream`); exits cleanly if the log file doesn't exist

**Dispatch**
- [ ] `GET /work` renders a simple form: worker count input and a "Start" button
- [ ] `POST /work` triggers `apm work` with the given worker count as
  `agents.max_concurrent`; streams stdout back to the browser via SSE

**API**
- [ ] `GET /api/tickets` returns all tickets as JSON
- [ ] `GET /api/tickets/:id` returns a single ticket as JSON
- [ ] `POST /api/tickets/:id/state` accepts `{"state": "..."}` and transitions
  the ticket; returns the updated ticket or an error
- [ ] `GET /api/workers` returns running workers as JSON
- [ ] All API responses use appropriate HTTP status codes

### Out of scope

- File browser or diff viewer (phase 2, targets remote use case)
- Terminal in browser (phase 2)
- Multi-user auth or session management
- Mobile-optimised UI
- Dark mode / theming
- Websocket (SSE is sufficient for v1)
- Remote deployment guide or Dockerfile

### Approach

**New crate: `apm-serve`**

Add a new binary crate `apm-serve` to the workspace, invoked as `apm serve`
via the main `apm` dispatcher. Keeps the server out of the core binary when not
needed.

Dependencies:
- `axum` — HTTP server and routing
- `tokio` — async runtime (already likely in workspace)
- `tower-http` — static file serving, compression
- `askama` or `minijinja` — server-side HTML templating (no JS build pipeline)
- `tokio-stream` — SSE support

**Frontend: HTMX + server-side HTML**

No JavaScript build step. HTML templates rendered server-side, HTMX for
in-place updates and form submissions. This keeps the server self-contained
(single binary, no `node_modules`) and works well over high-latency remote
connections.

CSS: a single bundled stylesheet (Tailwind via CDN for v1, inline for v2 to
remove the external dependency).

**Data layer**

`apm-serve` reads ticket state by calling `apm-core` functions directly (same
`load_all_from_git` used by the CLI). No separate database. No caching layer —
git reads are fast enough at ticket scale.

State mutations call the same `apm-core` functions used by the CLI, wrapped in
async tasks. This ensures CLI and web UI are always in sync.

**Auth**

A single middleware layer checks the `Authorization: Bearer <token>` header
when `serve.auth = true`. Token is read from config or env at startup. No
sessions, no cookies — stateless, works identically locally and remotely.

**Remote-readiness by design**

- Bind address is configurable: `0.0.0.0` + auth token = remote-ready
- All mutations go through HTTP API routes, not in-process calls — the API
  contract holds whether the client is a browser on localhost or across the
  internet
- SSE chosen over polling — works over long-lived remote connections
- No filesystem paths exposed to the client — log streaming reads server-side,
  sends events

**`apm.toml` additions**

```toml
[serve]
port  = 3131
bind  = "127.0.0.1"
auth  = false
token = ""          # required when auth = true
```

## History

| When | From | To | By |
|------|------|----|----|
| 2026-03-30T06:46Z | — | new | claude-0330-0245-main |
| 2026-03-30T16:31Z | new | in_design | philippepascal |
| 2026-03-30T16:33Z | in_design | specd | claude-0330-1631-spec9 |
| 2026-03-30T18:09Z | specd | closed | philippepascal |