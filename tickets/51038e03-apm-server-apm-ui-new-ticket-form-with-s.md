+++
id = "51038e03"
title = "apm-server + apm-ui: new ticket form with section pre-population"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "apm"
agent = "92131"
branch = "ticket/51038e03-apm-server-apm-ui-new-ticket-form-with-s"
created_at = "2026-03-31T06:12:50.437393Z"
updated_at = "2026-03-31T06:53:50.711229Z"
+++

## Spec

### Problem

There is no way to create a ticket from the UI. Steps 1–9 deliver the backend server, ticket list/detail API, and the markdown editor; Step 10 adds ticket creation.

A '+ New ticket' button (and keyboard shortcut) must open a modal form with a required title field and optional fields for each standard spec section (Problem, Acceptance criteria, Out of scope, Approach). Submitting the form calls a new POST /api/tickets endpoint in apm-server, which delegates atomically to ticket::create in apm-core — the same function the CLI uses. All provided sections are written to the ticket body at creation time (parity with the --section/--set CLI feature).

Without this, supervisors using the web UI have no way to capture new work items without switching to the command line.

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
| 2026-03-31T06:12Z | — | new | apm |
| 2026-03-31T06:53Z | new | in_design | philippepascal |