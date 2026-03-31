+++
id = "15b7b28e"
title = "apm-server + apm-ui: sync button (POST /api/sync)"
state = "in_design"
priority = 0
effort = 3
risk = 0
author = "apm"
agent = "5398"
branch = "ticket/15b7b28e-apm-server-apm-ui-sync-button-post-api-s"
created_at = "2026-03-31T06:13:15.004948Z"
updated_at = "2026-03-31T07:13:15.774086Z"
+++

## Spec

### Problem

The UI will have no way to pull the latest ticket state from git branches once it is running. Without a sync mechanism, the browser shows stale data until the server process is restarted. A single button press should trigger the same operations as `apm sync --offline`: refresh local ticket-branch refs (and optionally fetch from remote), then return up-to-date ticket data to the frontend.

This ticket adds the `POST /api/sync` endpoint to `apm-server` and the corresponding sync button (with keyboard shortcut and loading state) to `apm-ui`. It does not auto-accept or auto-close tickets — those are destructive, confirmation-requiring operations that belong in a later ticket.

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
| 2026-03-31T07:10Z | new | in_design | philippepascal |