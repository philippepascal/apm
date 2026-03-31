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

- [ ] A '+ New ticket' button is visible in the supervisorview column header area
- [ ] Pressing the '+ New ticket' button opens a modal dialog
- [ ] Pressing the 'n' key (when no text input is focused) opens the new ticket modal
- [ ] The modal contains a Title field that is required
- [ ] The modal contains optional textarea fields for Problem, Acceptance criteria, Out of scope, and Approach
- [ ] Attempting to submit the form with an empty title shows a validation error and does not call the API
- [ ] Submitting a valid form calls POST /api/tickets with the title and any non-empty section content
- [ ] POST /api/tickets returns 201 with the created ticket as JSON on success
- [ ] POST /api/tickets returns 400 when title is absent or empty
- [ ] After successful creation, the new ticket appears in the supervisor swimlanes (TanStack Query cache is invalidated)
- [ ] Pressing Escape or clicking Cancel closes the modal without creating a ticket
- [ ] The form shows a loading indicator while the mutation is in flight
- [ ] The form shows an inline error message if the API call fails

### Out of scope

- Setting effort, risk, or priority at creation time (covered by Step 13b inline editing)
- Choosing a custom author or supervisor at creation time (defaults to server-configured author)
- Attaching files or images
- Auto-saving draft form state across page reloads
- Editing or deleting tickets from the modal (edit is Step 9; delete is not planned)
- Any section beyond the four standard spec sections (Problem, Acceptance criteria, Out of scope, Approach)

### Approach

How the implementation will work.

### Open questions



### Amendment requests



## History

| When | From | To | By |
|------|------|----|----|
| 2026-03-31T06:12Z | — | new | apm |
| 2026-03-31T06:53Z | new | in_design | philippepascal |