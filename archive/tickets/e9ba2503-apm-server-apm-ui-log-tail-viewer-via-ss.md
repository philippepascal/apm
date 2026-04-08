+++
id = "e9ba2503"
title = "apm-server + apm-ui: log tail viewer via SSE"
state = "closed"
priority = 25
effort = 5
risk = 3
author = "apm"
agent = "12029"
branch = "ticket/e9ba2503-apm-server-apm-ui-log-tail-viewer-via-ss"
created_at = "2026-03-31T06:13:19.097973Z"
updated_at = "2026-04-01T07:47:40.117689Z"
+++

## Spec

### Problem

The apm server writes operational events to a log file (configured under `[logging].file` in `apm.toml`). There is no way for users of the browser dashboard to observe this stream; they must SSH into the server and `tail -f` the file manually.

This ticket adds:
1. `GET /api/log/stream` — an SSE endpoint that opens the configured log file, emits the last 100 lines as initial events, then follows new lines as they are appended.
2. A collapsible log panel in `apm-ui` that subscribes to the stream and renders a scrolling tail, auto-scrolling to the bottom as new lines arrive unless the user has scrolled up.

The dependency on Step 12a (ticket 56499b61) means `apm-server`, its `AppState`, and the `apm-ui` workscreen layout are all already in place when this work begins.

### Acceptance criteria

- [x] `GET /api/log/stream` returns `Content-Type: text/event-stream` with HTTP 200 when the log file exists
- [x] On connect, the server sends the last 100 lines of the existing log file as individual `data:` events before following new lines
- [x] Each line appended to the log file is delivered to connected clients as a `data:` event within 1 second of being written
- [x] `GET /api/log/stream` returns HTTP 404 when the configured log file does not exist
- [x] The server sends SSE keepalive comments (`: keepalive`) at least every 15 seconds to prevent proxy timeouts
- [x] When a client disconnects, the server stops polling the log file (no task leak)
- [x] The workscreen renders a collapsible log panel (toggle labelled "Logs") below the 3-column layout
- [x] Clicking the toggle opens and closes the log panel; open/closed state persists in the Zustand store
- [x] When the log panel is open, lines received via SSE are appended at the bottom and the panel auto-scrolls to the newest line
- [x] Auto-scroll is suppressed when the user has manually scrolled up; it resumes when the user scrolls back to the bottom
- [x] The panel buffers at most 500 lines; older lines are dropped when the buffer is full
- [x] When the SSE connection drops, the panel shows a "Reconnecting..." indicator and the EventSource auto-reconnects using the browser built-in retry

### Out of scope

- Filtering or searching log lines on the server side
- Log rotation handling (the stream does not follow across a log rotate/rename)
- Replaying the full log history beyond the initial 100 lines
- Configuring the log file path from the UI
- Authentication or authorisation on the stream endpoint
- Downloading or exporting the log from the UI

### Approach

**Prerequisites:** `apm-server` crate (Step 1), its `AppState` struct (populated through Steps 2–12a), and the `apm-ui` workscreen layout (Step 4) are already in place.

---

**Server-side — `apm-server/src/routes/log.rs` (new file)**

1. Add `log_file: PathBuf` to `AppState` in `apm-server/src/main.rs`, populated at startup from `config.logging.file` (the same field already used by `apm-core::logger`).
2. Wire the new route in the axum router:
   ```
   GET /api/log/stream  →  log::stream_handler
   ```
3. Handler logic (`stream_handler`):
   - Open the log file; if it does not exist, return HTTP 404 immediately.
   - Read the last 100 lines: seek to end, walk backward byte-by-byte collecting newlines, read the resulting slice. Send each line as a `data: <line>\n\n` SSE event.
   - Record the current file length as `offset`.
   - Spawn a `tokio::task` that:
     - Every 250 ms: stat the file; if the length grew, read the new bytes from `offset`, split on `\n`, send each non-empty line as a `data:` event, advance `offset`.
     - Every 15 s: send a `: keepalive\n\n` comment to the client.
     - Exits when the response channel is closed (client disconnected).
   - Return `Sse::new(ReceiverStream::new(rx))` with the appropriate headers (`Cache-Control: no-cache`).
4. Use `axum::response::sse::{Event, Sse}` and `tokio::sync::mpsc` for the channel.

---

**UI-side — `apm-ui/src/components/LogPanel.tsx` (new file)**

1. Use the browser's built-in `EventSource` (no polyfill needed); open it when the panel is mounted, close it when unmounted.
2. Maintain a `lines: string[]` state capped at 500 entries; on each SSE `message` event, append the new line and slice the oldest off if over the cap.
3. Track `isReconnecting` state: set it true in the `EventSource` `onerror` handler, clear it on the next successful `onmessage`.
4. Auto-scroll: use a `ref` on the scroll container. After each `lines` update, if the user has not scrolled up (tracked via `onScroll`), call `container.scrollTop = container.scrollHeight`.
5. Render:
   - A header row with the "Logs" label and a toggle chevron.
   - A monospace `<pre>` / `<code>` block for the log lines.
   - A "Reconnecting…" badge overlaid at the top of the panel when `isReconnecting` is true.
6. Integrate into the workscreen layout (`apm-ui/src/App.tsx` or the workscreen root component) as a collapsible section below the three columns, using the shadcn/ui `Collapsible` component.
7. Add `logPanelOpen: boolean` and `setLogPanelOpen` to the Zustand workscreen store.

---

**File changes summary:**
- `apm-server/src/routes/log.rs` — new file (~80 lines)
- `apm-server/src/main.rs` — add `log_file` field to `AppState`, register route
- `apm-ui/src/components/LogPanel.tsx` — new file (~120 lines)
- `apm-ui/src/store.ts` — add `logPanelOpen` + `setLogPanelOpen`
- `apm-ui/src/App.tsx` (or workscreen root) — mount `LogPanel` below columns, wire toggle

### Open questions



### Amendment requests



## History

| When | From | To | By |
|------|------|----|----|
| 2026-03-31T06:13Z | — | new | apm |
| 2026-03-31T07:23Z | new | in_design | philippepascal |
| 2026-03-31T07:26Z | in_design | specd | claude-0331-0723-aea0 |
| 2026-03-31T19:44Z | specd | ready | apm |
| 2026-04-01T07:08Z | ready | in_progress | philippepascal |
| 2026-04-01T07:15Z | in_progress | implemented | claude-0401-0709-6398 |
| 2026-04-01T07:46Z | implemented | accepted | apm-sync |
| 2026-04-01T07:47Z | accepted | closed | apm-sync |