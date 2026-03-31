+++
id = "51038e03"
title = "apm-server + apm-ui: new ticket form with section pre-population"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "apm"
agent = "philippepascal"
branch = "ticket/51038e03-apm-server-apm-ui-new-ticket-form-with-s"
created_at = "2026-03-31T06:12:50.437393Z"
updated_at = "2026-03-31T06:53:50.711229Z"
+++

## Spec

### Problem

There is no way to create a ticket from the UI. A '+ New ticket' button/shortcut opens a modal with fields for title (required) and optional spec sections (problem, acceptance criteria, out of scope, approach). Add POST /api/tickets backed by ticket::create in apm-core. Sections are written atomically at creation. Full spec context: initial_specs/UIdraft_spec_starter.md Step 10. Requires Step 9.

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
