+++
id = "711e717e"
title = "UI: add epic filter to supervisor board filter bar"
state = "in_design"
priority = 2
effort = 0
risk = 0
author = "claude-0401-2145-a8f3"
agent = "66037"
branch = "ticket/711e717e-ui-add-epic-filter-to-supervisor-board-f"
created_at = "2026-04-01T21:56:24.806901Z"
updated_at = "2026-04-02T00:57:35.927094Z"
+++

## Spec

### Problem

The supervisor board filter bar has state and agent filters but no epic filter. When multiple epics are active, all their tickets appear together, making it impossible for the supervisor to isolate a single epic's work in the board view.

The desired behaviour: an epic dropdown in the filter bar (beside the existing state and agent dropdowns) lets the supervisor select one epic and hide all tickets that belong to other epics, or select "All" to restore the default view. The dropdown is populated from `GET /api/epics`.

That API route does not yet exist. This ticket adds both the server-side endpoint (minimal: branch scan + name parsing, no ticket counts) and the UI dropdown.

### Acceptance criteria

Checkboxes; each one independently testable.

### Out of scope

Explicit list of what this ticket does not cover.

### Approach

How the implementation will work.

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-04-01T21:56Z | — | new | claude-0401-2145-a8f3 |
| 2026-04-01T22:01Z | new | groomed | claude-0401-2145-a8f3 |
| 2026-04-02T00:57Z | groomed | in_design | philippepascal |