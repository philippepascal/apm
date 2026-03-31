+++
id = "a6c115e1"
title = "apm-ui: markdown editor with RO/RW sections (CodeMirror 6) and save API"
state = "new"
priority = 0
effort = 0
risk = 0
author = "apm"
branch = "ticket/a6c115e1-apm-ui-markdown-editor-with-ro-rw-sectio"
created_at = "2026-03-31T06:12:48.893575Z"
updated_at = "2026-03-31T06:12:48.893575Z"
+++

## Spec

### Problem

The review button on the ticket detail panel should open a full markdown editor. Frontmatter and the History section must be read-only (CodeMirror compartments); all other sections are editable. Checkboxes render as interactive UI elements. Add PUT /api/tickets/:id/body to commit the edited content back to the ticket branch. Full spec context: initial_specs/UIdraft_spec_starter.md Step 9. Requires Step 8.

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
