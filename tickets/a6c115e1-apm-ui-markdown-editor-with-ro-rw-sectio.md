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

- [ ] Clicking the Review button on the ticket detail panel opens the CodeMirror 6 editor spanning the supervisor-view and ticket-detail columns; the worker view remains visible
- [ ] The frontmatter block (the +++ delimited block at the top of the document) is read-only: the user cannot edit, delete, or insert text within it
- [ ] The History section (from the ## History heading to end of document) is read-only: the user cannot edit, delete, or insert text within it
- [ ] All other sections (Problem, Acceptance criteria, Out of scope, Approach, Open questions, Amendment requests) are fully editable
- [ ] Checkboxes render as interactive HTML checkbox elements; clicking a checkbox toggles its checked state and updates the underlying markdown source accordingly
- [ ] The editor has a Save button that calls PUT /api/tickets/:id/body with the full edited document text
- [ ] PUT /api/tickets/:id/body returns 200 on success and commits the new content to the ticket branch via git::commit_to_branch
- [ ] PUT /api/tickets/:id/body returns 404 when the ticket ID is not found
- [ ] PUT /api/tickets/:id/body returns 422 when the submitted content modifies the frontmatter block or the History section relative to the current branch content
- [ ] The editor has a Cancel button that closes the editor and returns to the read-only detail view
- [ ] If there are unsaved changes, clicking Cancel shows a confirmation dialog before discarding
- [ ] Cmd+S / Ctrl+S inside the editor triggers the save action

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