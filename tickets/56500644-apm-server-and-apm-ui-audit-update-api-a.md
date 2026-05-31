+++
id = "56500644"
title = "apm-server and apm-ui audit: update API and UI surfaces for new workflow schema"
state = "closed"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/56500644-apm-server-and-apm-ui-audit-update-api-a"
created_at = "2026-05-31T02:00:14.619832Z"
updated_at = "2026-05-31T03:04:01.311436Z"
epic = "a42eceea"
target_branch = "epic/a42eceea-workflow-schema-state-level-worker-profi"
depends_on = ["1a13dee7"]
+++

## Spec

### Problem

Audit apm-server (HTTP API) and apm-ui (frontend) for any surface that exposes or relies on the old workflow schema (transition.role, transition.worker_profile, state.actionable_by). Update.

WHY: the schema change in 1e758cd5 + 1a13dee7 alters how worker assignment data is structured. If apm-server endpoints serialize transition.role or actionable_by, those payloads break. The UI may render labels or filter using these fields. Both need a sweep.

AUDIT TARGETS:

1. apm-server/src/handlers/ (every .rs file). Endpoints to check:
   - The tickets endpoint (handlers/tickets.rs and related). Look for any serialised workflow metadata in the ticket envelope or detail response.
   - The workflow / states endpoint if one exists.
   - The agents / diagnostics endpoint (if 36b6f742's diagnostic was wired to the server).
   - Any endpoint that surfaces state metadata (e.g., showing 'Actionable by: supervisor').

2. apm-server/src/models.rs — every response or request struct. Look for fields named role, worker_profile, actionable_by, or related. Update field names to reflect the new model.

3. apm-ui/src/ TypeScript components and stores. Check:
   - TicketCard.tsx, Swimlane.tsx, SupervisorView.tsx, TicketDetail.tsx (already exist; revisited recently)
   - Anything rendering 'actionable_by' or 'role' from the API
   - Workflow visualisation components if any
   - Filters or search controls that use old field names
   - The merge-failure / recovery surfaces from 12f2c7fa (verify still works under new schema)

4. apm-ui type definitions. If there is a generated or hand-written types file mirroring the API response, update it. Pay attention to any TicketDetail, Swimlane, or Workflow types.

5. End-to-end check: start apm-server locally, open the UI, exercise:
   - Viewing a ticket in each state
   - Triggering apm start (the in_progress dispatch)
   - Reviewing the recovery surfaces (merge_failed state — note: the merge_failed to in_progress transition is now removed per the new lifecycle; the UI should only offer merge_failed to ready and merge_failed to implemented)

TESTS:
- All apm-server unit tests pass (cargo test -p apm-server)
- All apm-ui frontend tests pass (npm test or vitest)
- Manual: open the UI against the apm repo, pick a ticket from each lifecycle stage, verify rendering and actions

OUT OF SCOPE:
- Schema structs (in 1e758cd5)
- Dispatch path (in 1a13dee7)
- CLI help and Rust doc audit (in d2a947ea)
- New UI features. Only existing surfaces that touch the changed fields.

REFERENCES:
- apm-server/src/handlers/
- apm-server/src/models.rs
- apm-ui/src/components/
- apm-ui/src/store/ (zustand stores or similar)

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
| 2026-05-31T02:00Z | — | new | philippepascal |
| 2026-05-31T03:04Z | new | closed | philippepascal |
