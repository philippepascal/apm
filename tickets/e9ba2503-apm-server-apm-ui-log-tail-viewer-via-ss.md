+++
id = "e9ba2503"
title = "apm-server + apm-ui: log tail viewer via SSE"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "apm"
agent = "philippepascal"
branch = "ticket/e9ba2503-apm-server-apm-ui-log-tail-viewer-via-ss"
created_at = "2026-03-31T06:13:19.097973Z"
updated_at = "2026-03-31T07:23:22.371249Z"
+++

## Spec

### Problem

There is no visibility into the apm log from the UI. Add GET /api/log/stream as a Server-Sent Events endpoint tailing the configured log file. A collapsible log panel in the UI shows the live stream. Full spec context: initial_specs/UIdraft_spec_starter.md Step 14. Requires Step 12a.

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
