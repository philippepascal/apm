+++
id = "a88c5096"
title = "UI: assign buttons and window"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/a88c5096-ui-assign-buttons-and-window"
created_at = "2026-04-09T05:16:55.238687Z"
updated_at = "2026-04-09T05:29:43.023335Z"
+++

## Spec

### Problem

The ticket detail panel has a "Reassign to me" button that only assigns a ticket to the current user. There is no way in the UI to assign a ticket to another collaborator. This is limiting in a team setting where a user may want to hand off a ticket or triage work to others.\n\nAdditionally, the current button calls POST /api/tickets/{id}/take, an endpoint that does not exist on the server, so the feature is currently broken end-to-end.\n\nThe desired behaviour: clicking an "Assign" button opens a small picker listing all project collaborators plus an "Unassigned" option. Selecting an entry assigns or clears the owner and dismisses the picker. The existing PATCH /api/tickets/:id endpoint already accepts an owner field, so the main work is a new collaborators API endpoint and a frontend picker component.

### Acceptance criteria

- [ ] The "Reassign to me" button is replaced by an "Assign" button in the TransitionButtons component\n- [ ] Clicking "Assign" opens a picker listing all project collaborators\n- [ ] The picker includes an "Unassigned" entry at the top to clear the owner\n- [ ] The current user is always present in the picker list\n- [ ] Selecting a collaborator calls PATCH /api/tickets/:id with { owner: username } and dismisses the picker\n- [ ] Selecting "Unassigned" calls PATCH /api/tickets/:id with { owner: "" } and dismisses the picker\n- [ ] After a successful assignment the ticket detail panel reflects the updated owner without a full page reload\n- [ ] While the assignment request is in-flight, the Assign button is disabled\n- [ ] If the assignment request fails, an error message is shown near the button\n- [ ] Pressing Escape or clicking outside the picker dismisses it without making any change\n- [ ] GET /api/collaborators returns the project collaborator list (from GitHub API or config.toml fallback)

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
| 2026-04-09T05:16Z | — | new | philippepascal |
| 2026-04-09T05:17Z | new | groomed | apm |
| 2026-04-09T05:29Z | groomed | in_design | philippepascal |