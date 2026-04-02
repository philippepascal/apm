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

- [ ] An "Epic" dropdown appears in the supervisor board filter bar, positioned after the "All agents" dropdown
- [ ] The dropdown contains an "All epics" option that is selected by default and shows all tickets
- [ ] The dropdown options are populated from `GET /api/epics` and show each epic's title
- [ ] Selecting an epic hides all ticket cards whose `epic` field does not match the selected epic id
- [ ] Tickets with no `epic` field are hidden when any specific epic is selected
- [ ] Selecting "All epics" after a specific epic restores the full board view
- [ ] The "No tickets match the current filters" empty state appears when an epic is selected and no tickets match
- [ ] The epic filter composes with the existing state, agent, and search filters (all active simultaneously)
- [ ] `GET /api/epics` returns a JSON array; each element has `id`, `title`, and `branch` string fields
- [ ] `GET /api/epics` returns an empty array when no `epic/*` branches exist
- [ ] The dropdown renders but shows only "All epics" when `GET /api/epics` returns an empty array

### Out of scope

- Epic filter in the Queue panel (separate item in docs/epics.md UI section)
- Epic column in the Queue panel
- Epic selector in Engine controls
- POST /api/epics (create epic)
- GET /api/epics/:id (epic detail with ticket list)
- Ticket lock icon for unresolved `depends_on` entries
- Clickable epic label in Ticket detail panel
- Derived epic state or ticket counts in the `GET /api/epics` response
- `epic` and `target_branch` fields on `CreateTicketRequest`
- Any changes to `apm work --epic` or the work engine epic filter

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