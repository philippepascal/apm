+++
id = "e2781682"
title = "apm-server and apm-ui audit: update API and frontend for schema changes"
state = "new"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/e2781682-apm-server-and-apm-ui-audit-update-api-a"
created_at = "2026-05-31T02:59:20.324716Z"
updated_at = "2026-05-31T02:59:20.324716Z"
epic = "9c3c4c20"
target_branch = "epic/9c3c4c20-workflow-schema-cleanup-state-level-work"
depends_on = ["e05c0463", "4d20ba2f"]
+++

## Spec

### Problem

STEP 10 of the incremental workflow schema cleanup. After the schema is fully migrated (e05c0463 drops transition.worker_profile, 4d20ba2f makes workers.default mandatory), audit apm-server endpoints and apm-ui components for any surface that depended on the old shape.

AUDIT TARGETS:

1. apm-server/src/handlers/ (every .rs file):
   - tickets handler and related: look for serialised workflow metadata in the ticket envelope or detail response. Fields like 'actionable', 'role' on a transition, 'worker_profile' on a transition.
   - Any workflow / states endpoint.
   - The agents / diagnostics endpoint (resolve_for_diagnostic surface) — verify it still produces correct output after the dispatch path moved to state-level worker_profile.
   - The merge-failure / recovery endpoint surfaces: 'merge_failed to in_progress' is removed in 071886fc, so the recovery options should no longer offer that transition.

2. apm-server/src/models.rs (every response / request struct). Search for fields named role, worker_profile (on a transition-shaped struct), actionable, or related. Update or remove.

3. apm-ui/src/ TypeScript components:
   - TicketCard.tsx, Swimlane.tsx, SupervisorView.tsx, TicketDetail.tsx
   - Anything rendering 'actionable' or 'role' from the API
   - Recovery surfaces from earlier merge_failed work (ensure the UI no longer offers 'merge_failed to in_progress' since that transition is gone)
   - Workflow visualisation components

4. apm-ui type definitions: hand-written or generated types mirroring the API. Update field names accordingly.

5. End-to-end check: start apm-server locally, open the UI, exercise:
   - Viewing a ticket in each state in the new workflow
   - Triggering apm start (in_progress dispatch)
   - The recovery surfaces on a merge_failed ticket (only 'to ready' and 'to implemented' should be offered)

TESTS:
- cargo test -p apm-server passes
- apm-ui frontend tests pass (vitest)
- Manual: open the UI against the apm repo; verify rendering for each lifecycle state and that no offered action references removed transitions

OUT OF SCOPE:
- Schema struct changes (handled by earlier tickets).
- Help text sweep (a5cffb01).
- New UI features. Only existing surfaces that touch changed fields.

REFERENCES:
- apm-server/src/handlers/
- apm-server/src/models.rs
- apm-ui/src/components/
- apm-ui/src/store/
- 071886fc for the removed transitions

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
| 2026-05-31T02:59Z | — | new | philippepascal |
