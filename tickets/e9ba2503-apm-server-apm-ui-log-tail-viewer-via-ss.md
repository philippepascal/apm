+++
id = "e9ba2503"
title = "apm-server + apm-ui: log tail viewer via SSE"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "apm"
agent = "37259"
branch = "ticket/e9ba2503-apm-server-apm-ui-log-tail-viewer-via-ss"
created_at = "2026-03-31T06:13:19.097973Z"
updated_at = "2026-03-31T07:23:22.371249Z"
+++

## Spec

### Problem

The apm server writes operational events to a log file (configured under `[logging].file` in `apm.toml`). There is no way for users of the browser dashboard to observe this stream; they must SSH into the server and `tail -f` the file manually.

This ticket adds:
1. `GET /api/log/stream` — an SSE endpoint that opens the configured log file, emits the last 100 lines as initial events, then follows new lines as they are appended.
2. A collapsible log panel in `apm-ui` that subscribes to the stream and renders a scrolling tail, auto-scrolling to the bottom as new lines arrive unless the user has scrolled up.

The dependency on Step 12a (ticket 56499b61) means `apm-server`, its `AppState`, and the `apm-ui` workscreen layout are all already in place when this work begins.

### Acceptance criteria

- [ ] `GET /api/log/stream` returns `Content-Type: text/event-stream` with HTTP 200 when the log file exists
- [ ] On connect, the server sends the last 100 lines of the existing log file as individual `data:` events before following new lines
- [ ] Each line appended to the log file is delivered to connected clients as a `data:` event within 1 second of being written
- [ ] `GET /api/log/stream` returns HTTP 404 when the configured log file does not exist
- [ ] The server sends SSE keepalive comments (`: keepalive`) at least every 15 seconds to prevent proxy timeouts
- [ ] When a client disconnects, the server stops polling the log file (no task leak)
- [ ] The workscreen renders a collapsible log panel (toggle labelled "Logs") below the 3-column layout
- [ ] Clicking the toggle opens and closes the log panel; open/closed state persists in the Zustand store
- [ ] When the log panel is open, lines received via SSE are appended at the bottom and the panel auto-scrolls to the newest line
- [ ] Auto-scroll is suppressed when the user has manually scrolled up; it resumes when the user scrolls back to the bottom
- [ ] The panel buffers at most 500 lines; older lines are dropped when the buffer is full
- [ ] When the SSE connection drops, the panel shows a "Reconnecting..." indicator and the EventSource auto-reconnects using the browser built-in retry

### Out of scope

Explicit list of what this ticket does not cover.

### Approach

How the implementation will work.

### Open questions



### Amendment requests



## History

| When | From | To | By |
|------|------|----|----|
| 2026-03-31T06:13Z | — | new | apm |
| 2026-03-31T07:23Z | new | in_design | philippepascal |