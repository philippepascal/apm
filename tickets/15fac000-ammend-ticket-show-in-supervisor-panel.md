+++
id = "15fac000"
title = "ammend ticket show in supervisor panel"
state = "in_design"
priority = 0
effort = 4
risk = 0
author = "philippepascal"
branch = "ticket/15fac000-ammend-ticket-show-in-supervisor-panel"
created_at = "2026-04-07T18:17:11.036755Z"
updated_at = "2026-04-07T18:26:54.144806Z"
+++

## Spec

### Problem

Tickets in the `ammend` state currently appear in the supervisor panel in the APM UI. This is wrong — `ammend` is a state where a spec-writer agent must act (it has `actionable = ["agent"]` in `workflow.toml`), so it belongs in the agent work queue, not in the supervisor's attention queue.

The root cause is that `apm-ui/src/lib/supervisorUtils.ts` hardcodes a `SUPERVISOR_STATES` array that explicitly names `'ammend'`. The supervisor panel uses this array to decide what to render, with no reference to `workflow.toml`. Any future workflow changes (new states, renamed states, changed `actionable` actors) require manual UI edits or risk the same bug recurring.

The desired behaviour is that the supervisor panel derives its visible-state list from the `actionable` property already present in `workflow.toml`: it should show states where the `supervisor` actor is listed as actionable. The `ammend` state (`actionable = ["agent"]`) is then excluded automatically, with no string matching on state names in the UI.

### Acceptance criteria

- [ ] `ammend` tickets do not appear in the supervisor panel
- [ ] `question`, `specd`, `blocked`, and `implemented` tickets continue to appear in the supervisor panel
- [ ] The supervisor panel's visible-state list is derived at runtime from the server's workflow configuration, not from a hardcoded state-name list in the UI
- [ ] No state name (including `'ammend'`) is hardcoded in the supervisor panel display logic
- [ ] Adding or renaming a state in `workflow.toml` updates the supervisor panel automatically without requiring a UI code change

### Out of scope

- Changing how the agent work queue determines which tickets to surface (already driven by `actionable` in the server)
- Updating `ALL_WORKFLOW_STATES` in `SupervisorView.tsx` (used only for the filter dropdown, separate concern)
- Adding or modifying states in `workflow.toml`
- State colours or labels in the UI

### Approach

### 1. Add a server endpoint to expose workflow state config

**File:** apm-server/src/main.rs

Add `GET /api/workflow/states` to the protected router. The handler reads the loaded `Config`, iterates `config.workflow.states`, and serialises a JSON array with `id` and `actionable` fields for each state. Only these two fields are needed by the UI for this purpose.

### 2. Update supervisorUtils.ts — remove hardcoded list

**File:** apm-ui/src/lib/supervisorUtils.ts

- Delete the `SUPERVISOR_STATES` constant.
- Export a pure helper `supervisorStatesFromWorkflow(states: WorkflowState[]): string[]` that returns the `id` of each state where `actionable` includes `"supervisor"`.

### 3. Update SupervisorView.tsx — fetch and derive

**File:** apm-ui/src/components/supervisor/SupervisorView.tsx

- On component mount, call `GET /api/workflow/states` alongside the existing `/api/tickets` fetch.
- Pass the result to `supervisorStatesFromWorkflow` to compute the dynamic supervisor state list.
- Replace the `SUPERVISOR_STATES` import and every usage with this derived list wherever `visibleStates` is calculated.

### Order of steps

1. Server endpoint first (unblocks UI work).
2. `supervisorUtils.ts` helper (pure function, easy to unit-test in isolation).
3. `SupervisorView.tsx` integration.

### Constraints / gotchas

- The `new` state currently has no `actionable` entries in `workflow.toml`. If it should remain visible in the supervisor panel, `actionable = ["supervisor"]` must be added to the `new` state in `.apm/workflow.toml`. Confirm with the author before implementing; update `workflow.toml` as part of this ticket if yes.
- The new endpoint is purely additive — no existing endpoints change behaviour.

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-04-07T18:17Z | — | new | philippepascal |
| 2026-04-07T18:17Z | new | groomed | apm |
| 2026-04-07T18:22Z | groomed | in_design | philippepascal |