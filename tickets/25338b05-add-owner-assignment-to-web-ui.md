+++
id = "25338b05"
title = "Add owner assignment to web UI"
state = "new"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
branch = "ticket/25338b05-add-owner-assignment-to-web-ui"
created_at = "2026-04-06T20:57:16.722499Z"
updated_at = "2026-04-06T21:06:55.558363Z"
depends_on = ["f38a9b24", "87fb645e"]
+++

## Spec

### Problem

The CLI supports assigning ticket owners via 'apm assign <id> <username>' and filtering by owner with 'apm list --owner', but the web UI has no equivalent. There is no way to view, set, or clear ticket owners from the dashboard. Additionally, PATCH /api/tickets/:id only accepts effort, risk, and priority — there is no API endpoint to change the owner field, so even if the UI wanted to support it, there is no backend route to call. The web UI needs: (1) an owner field visible on every ticket (showing the current owner or empty), (2) a way to assign or reassign a ticket to a collaborator, (3) a way to clear the owner, and (4) a PATCH or dedicated endpoint to update the owner field server-side.

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
| 2026-04-06T20:57Z | — | new | philippepascal |