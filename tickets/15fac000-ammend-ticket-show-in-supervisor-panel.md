+++
id = "15fac000"
title = "ammend ticket show in supervisor panel"
state = "in_design"
priority = 0
effort = 4
risk = 2
author = "philippepascal"
branch = "ticket/15fac000-ammend-ticket-show-in-supervisor-panel"
created_at = "2026-04-07T18:17:11.036755Z"
updated_at = "2026-04-07T18:47:20.412768Z"
+++

## Spec

### Problem

Tickets in the `ammend` state currently appear in the supervisor panel in the APM UI. This is wrong — `ammend` is a state where a spec-writer agent must act (it has `actionable = ["agent"]` in `workflow.toml`), so it belongs in the agent work queue, not in the supervisor's attention queue.

The root cause is that `apm-ui/src/lib/supervisorUtils.ts` hardcodes a `SUPERVISOR_STATES` array that explicitly names `'ammend'`. The supervisor panel uses this array to decide what to render, with no reference to `workflow.toml`. Any future workflow changes (new states, renamed states, changed `actionable` actors) require manual UI edits or risk the same bug recurring.

The desired behaviour is that the supervisor panel derives its visible-state list from `workflow.toml` configuration surfaced by the server, with two structural exceptions: `new` is always visible (it has no `actionable` entries but supervisors must act on it), and terminal states (e.g. `closed`) are never visible. All other states show in the supervisor panel only when `actionable` includes `"supervisor"`. The `ammend` state (`actionable = ["agent"]`) is then excluded automatically.

### Acceptance criteria

- [ ] `ammend` tickets do not appear in the supervisor panel
- [ ] `new`, `question`, `specd`, `blocked`, and `implemented` tickets continue to appear in the supervisor panel
- [ ] `new` tickets always appear in the supervisor panel regardless of its `actionable` value in `workflow.toml`
- [ ] Terminal-state tickets (e.g. `closed`) never appear in the supervisor panel regardless of `actionable`
- [ ] `GET /api/tickets` returns a `{ tickets: [...], supervisor_states: [...] }` envelope instead of a plain array
- [ ] `supervisor_states` in the envelope lists state ids that are either `"new"` (hardcoded) or have `actionable` containing `"supervisor"` and are not terminal
- [ ] No state name other than `"new"` is hardcoded in the supervisor panel display logic
- [ ] Adding or renaming a non-structural state in `workflow.toml` automatically updates the supervisor panel without a UI code change

### Out of scope

- Adding a new `/api/workflow/states` endpoint (superseded by envelope field on `/api/tickets`)
- Changing the `actionable` field of `new` in `workflow.toml` (visibility of `new` is structural, not config-driven)
- Changing how the agent work queue determines which tickets to surface (already driven by `actionable` in the server)
- Updating `ALL_WORKFLOW_STATES` in `SupervisorView.tsx` (used only for the filter dropdown, separate concern)
- Updating `groupBySupervisorState` in `supervisorUtils.ts` or `WorkScreen.tsx` (agent view, separate concern)
- Adding or modifying other states in `workflow.toml`
- State colours or labels in the UI

### Approach

**Step 1 — Server endpoint** (`apm-server/src/main.rs`)

Add `GET /api/workflow/states` to the protected router. The handler reads the loaded `Config`, iterates `config.workflow.states`, and serialises a JSON array with `id` and `actionable` fields for each state. Only these two fields are needed by the UI.

**Step 2 — supervisorUtils.ts** (`apm-ui/src/lib/supervisorUtils.ts`)

- Delete the `SUPERVISOR_STATES` constant.
- Export a pure helper `supervisorStatesFromWorkflow(states: WorkflowState[]): string[]` that returns the `id` of each state where `actionable` includes `"supervisor"`.

**Step 3 — SupervisorView.tsx** (`apm-ui/src/components/supervisor/SupervisorView.tsx`)

- On component mount, call `GET /api/workflow/states` alongside the existing `/api/tickets` fetch.
- Pass the result to `supervisorStatesFromWorkflow` to compute the dynamic supervisor state list.
- Replace the `SUPERVISOR_STATES` import and every usage with this derived list wherever `visibleStates` is calculated.

Steps must be done in order (server endpoint unblocks the UI work).

**Gotcha — `new` state:** `new` currently has no `actionable` entries in `workflow.toml`, so it would fall off the supervisor panel under this scheme. If it should remain visible, add `actionable = ["supervisor"]` to the `new` state in `.apm/workflow.toml` as part of this ticket. Confirm with the author first.

The new endpoint is purely additive — no existing endpoints change.

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

- [ ] Do not add a new /api/workflow/states endpoint. Instead, add a supervisor_states field to the existing /api/tickets response envelope. The workflow config is static for the server lifetime — no need for a separate endpoint for a single consumer.
- [ ] Hardcode new as always visible in the supervisor panel and terminal states (closed) as never visible. These are structural states native to apm, not configuration-bound. Only non-structural states should derive visibility from the actionable field in workflow.toml. Update the Approach, AC, and supervisorUtils logic accordingly.

### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-04-07T18:17Z | — | new | philippepascal |
| 2026-04-07T18:17Z | new | groomed | apm |
| 2026-04-07T18:22Z | groomed | in_design | philippepascal |
| 2026-04-07T18:27Z | in_design | specd | claude-0407-1822-e230 |
| 2026-04-07T18:40Z | specd | ammend | claude-0407-review |
| 2026-04-07T18:47Z | ammend | in_design | philippepascal |