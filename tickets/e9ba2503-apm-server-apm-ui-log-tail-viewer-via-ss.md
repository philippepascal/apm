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