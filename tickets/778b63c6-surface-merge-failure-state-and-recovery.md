+++
id = "778b63c6"
title = "Surface merge-failure state and recovery hints in apm-server and apm-ui (read-only)"
state = "new"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/778b63c6-surface-merge-failure-state-and-recovery"
created_at = "2026-05-30T02:11:35.270399Z"
updated_at = "2026-05-30T02:11:45.498658Z"
depends_on = ["ae4104f2"]
+++

## Spec

### Problem

GOAL: in the apm-ui supervisor view, a ticket that landed in a merge-failure state (merge_failed in the default workflow; the state name is configured per project) should be visually identifiable AND its detail view should show what went wrong plus the supervisor's recovery options as APM CLI command text. NO ACTION BUTTONS. The UI is read-only here — the user runs the apm CLI to actually transition the ticket. Keeping the UI read-only avoids state-transition API surface area, server-side authorization complications, and accidental-click footguns; it also keeps the apm CLI as the single source of truth for state changes.

PROBLEM: in the current UI, merge_failed tickets are visually indistinguishable from in_progress / implemented / etc. The supervisor has to open the ticket in the CLI to see the captured Merge notes (the git error from set_merge_notes) and to know what apm state command to run. The UI is the natural triage surface for the supervisor — it should surface (a) which tickets are stuck on a merge failure, and (b) what the failure was and how to recover, without becoming a state-transition control plane.

DEPENDENCY: depends_on ae4104f2 (the CLI ticket that adds the config-derived recovery helper in apm-core). The server and UI consume the helper output via a new field on the existing ticket API response.

APPROACH (direction; spec-writer to refine):
1. Server (apm-server): extend the per-ticket API response (the one apm-ui already fetches for ticket detail) with a new field, e.g. recovery_options, computed by calling the helper added in ae4104f2 for the ticket's current state. The field is an array of objects: target state, label (from transition.label or the to-state ID), kind (retry-merge / return-to-worker / abandon / other), and the exact apm CLI command text the supervisor would run (e.g. apm state ID implemented). Also expose has_merge_notes (bool) or the merge_notes string itself, parsed from the ticket body's Merge notes section using the existing section parser.

   For list endpoints, no new field is required if the UI relies on the existing state field to apply visual styling (next section).

2. Frontend (apm-ui SupervisorView):
   a) Visual badge: any ticket whose state is the on_failure target of a merging-completion transition (i.e. its current state has retry-merge recovery options available) renders with a distinct visual marker on the TicketCard — a small red/amber pill or border. The classification comes from the recovery_options field being non-empty; the frontend does NOT hardcode the state name merge_failed.
   b) Detail panel: when a ticket with recovery options is opened, the detail view shows:
      - A Merge failure section displaying the merge_notes string from the API (when present) in monospace, with the heading derived from the state's configured label.
      - A Recovery section listing each option from recovery_options, formatted as: option label, a short kind description (Retry / Return to worker / Abandon / Other), and the apm CLI command in a copy-friendly code block. NO buttons that perform the action.
      - A link to docs/merge-failed-recovery.md for context.

OUT OF SCOPE:
- Any UI action buttons that POST to the server to trigger a state transition. Recovery happens via apm CLI; the UI displays guidance only.
- A state-transition API endpoint. None is added by this ticket.
- Inline editing of merge notes or any other ticket body section.
- Dispatcher / apm work behavior changes around merge failure (separate concern).
- Hardcoding merge_failed or any other state name in either the server or the frontend. All classification flows from the helper output via the API field.
- Changes to the underlying recovery logic itself (delivered by ae4104f2).

TESTS:
- Server: a per-ticket API integration test that loads a ticket in merge_failed state and asserts recovery_options is populated correctly under the default workflow. A second test under a custom workflow where the recovery state is renamed (e.g. ship_failed → shipped) — assert recovery_options reflects the renamed states and that the apm CLI command strings are derived from config, not hardcoded.
- Server: a per-ticket API test that loads a ticket whose body contains a Merge notes section — assert the parsed string is returned.
- Frontend: a SupervisorView render test that asserts a ticket with non-empty recovery_options renders the visual badge. A detail-view test that asserts the Merge failure section displays the merge_notes string verbatim and the Recovery section lists the apm CLI command lines exactly as supplied by the API. A negative test: a ticket whose recovery_options is empty (e.g. a ticket in a normal in_progress state) shows neither the badge nor the recovery panel.

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
| 2026-05-30T02:11Z | — | new | philippepascal |