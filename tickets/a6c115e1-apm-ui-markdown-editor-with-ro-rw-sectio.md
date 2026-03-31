+++
id = "a6c115e1"
title = "apm-ui: markdown editor with RO/RW sections (CodeMirror 6) and save API"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "apm"
agent = "8631"
branch = "ticket/a6c115e1-apm-ui-markdown-editor-with-ro-rw-sectio"
created_at = "2026-03-31T06:12:48.893575Z"
updated_at = "2026-03-31T06:49:16.790304Z"
+++

## Spec

### Problem

The ticket detail panel (right column, Step 6) shows ticket content as read-only markdown. There is no way to edit a ticket body from the UI. The review button on the detail panel (Step 8) should open a full CodeMirror 6 editor occupying the supervisor-view and ticket-detail columns. The editor must enforce read-only constraints on the frontmatter block and the History section while allowing free editing everywhere else. Checkboxes must render as interactive UI elements. A new PUT /api/tickets/:id/body endpoint commits the edited content back to the ticket branch using the existing git::commit_to_branch function in apm-core.

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
| 2026-03-31T06:49Z | new | in_design | philippepascal |