+++
id = "15fac000"
title = "ammend ticket show in supervisor panel"
state = "in_progress"
priority = 0
effort = 4
risk = 3
author = "philippepascal"
branch = "ticket/15fac000-ammend-ticket-show-in-supervisor-panel"
created_at = "2026-04-07T18:17:11.036755Z"
updated_at = "2026-04-07T19:24:59.579141Z"
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

**Step 1 — Wrap `/api/tickets` response in an envelope** (`apm-server/src/main.rs`)

Add a new serialisable struct:
```rust
#[derive(serde::Serialize)]
struct TicketsEnvelope {
    tickets: Vec<TicketResponse>,
    supervisor_states: Vec<String>,
}
```

In `list_tickets`, after building `response: Vec<TicketResponse>`, compute `supervisor_states`:

- Always include `"new"` (structural — always visible to supervisor)
- For each non-terminal state in `cfg.workflow.states`, if `actionable` contains `"supervisor"`, include its `id`
- If config cannot be loaded, fall back to the static list `["new", "question", "specd", "blocked", "implemented"]`

Change the return type from `Json<Vec<TicketResponse>>` to `Json<TicketsEnvelope>` and return the envelope.

Server tests that parse `/api/tickets` as a plain array (search for `.uri("/api/tickets")` in the test section of `main.rs`) must be updated to deserialise the envelope and access `.tickets`.

**Step 2 — Remove hardcoded list from `supervisorUtils.ts`** (`apm-ui/src/lib/supervisorUtils.ts`)

- Remove the exported `SUPERVISOR_STATES` const and `SupervisorState` type.
- `groupBySupervisorState` is used by `WorkScreen.tsx` (agent view) and must remain. Update its signature to accept a `string[]` states parameter rather than iterating over the removed constant:
  ```ts
  export function groupBySupervisorState(states: string[], tickets: Ticket[]): [string, Ticket[]][] {
    return states
      .map((state): [string, Ticket[]] => [state, tickets.filter((t) => t.state === state)])
      .filter(([, group]) => group.length > 0)
  }
  ```

**Step 3 — Update `WorkScreen.tsx`** (`apm-ui/src/components/WorkScreen.tsx`)

Update the call to `groupBySupervisorState` to pass a local states list. `WorkScreen.tsx` can define its own hardcoded list as needed for the agent view — this is not the supervisor panel.

**Step 4 — Consume the envelope in `SupervisorView.tsx`** (`apm-ui/src/components/supervisor/SupervisorView.tsx`)

- Update `fetchTickets` return type to `Promise<{ tickets: Ticket[]; supervisor_states: string[] }>` and parse both fields from the JSON response.
- Derive the visible-state base list: `data?.supervisor_states ?? ['new', 'question', 'specd', 'blocked', 'implemented']`
- In the `visibleStates` memo, replace `[...SUPERVISOR_STATES]` with the derived list.
- Remove the `SUPERVISOR_STATES` import.

**Order:** Step 1 (server) → Step 2 (`supervisorUtils.ts`) → Step 3 (`WorkScreen.tsx`) → Step 4 (`SupervisorView.tsx`).

**Constraints / gotchas**

- The API envelope change is breaking for any consumer expecting a plain array from `/api/tickets`. All server-side tests hitting that route must be updated.
- `new` is included in `supervisor_states` unconditionally by the server. The `closed` (and any terminal state) is excluded unconditionally, even if `actionable` were set. The UI's existing `showClosed` toggle (appends `'closed'` to `visibleStates`) remains correct and unchanged.

### Step 1 — Wrap `/api/tickets` response in an envelope (`apm-server/src/main.rs`)

Add a new serialisable struct:
```rust
#[derive(serde::Serialize)]
struct TicketsEnvelope {
    tickets: Vec<TicketResponse>,
    supervisor_states: Vec<String>,
}
```

In `list_tickets`, after building `response: Vec<TicketResponse>`, compute `supervisor_states`:

- Always include `"new"` (structural — always visible to supervisor)
- For each non-terminal state in `cfg.workflow.states`, if `actionable` contains `"supervisor"`, include its `id`
- If config cannot be loaded, fall back to the static list `["new", "question", "specd", "blocked", "implemented"]`

Change the return type from `Json<Vec<TicketResponse>>` to `Json<TicketsEnvelope>` and return the envelope.

**Note on tests:** Server tests that parse `/api/tickets` as a plain array (e.g. `list_tickets_returns_200_json_array`, `list_tickets_excludes_closed_by_default`, etc.) must be updated to deserialise the envelope and access `.tickets`. Search for `.uri("/api/tickets")` in the test section of `main.rs` to find all affected tests.

### Step 2 — Remove hardcoded list from `supervisorUtils.ts` (`apm-ui/src/lib/supervisorUtils.ts`)

- Remove the exported `SUPERVISOR_STATES` const and `SupervisorState` type.
- `groupBySupervisorState` is used by `WorkScreen.tsx` (the agent view) and must remain. Update it to accept a `string[]` states parameter rather than iterating over the removed constant:
  ```ts
  export function groupBySupervisorState(states: string[], tickets: Ticket[]): [string, Ticket[]][] {
    return states
      .map((state): [string, Ticket[]] => [state, tickets.filter((t) => t.state === state)])
      .filter(([, group]) => group.length > 0)
  }
  ```
- Update the call site in `WorkScreen.tsx` to pass its own local states list (it can keep any hardcoded list it needs since it is the agent view, not the supervisor panel).

### Step 3 — Consume the envelope in `SupervisorView.tsx` (`apm-ui/src/components/supervisor/SupervisorView.tsx`)

- Update `fetchTickets` return type to `Promise<{ tickets: Ticket[]; supervisor_states: string[] }>` and read both fields from the JSON response.
- From the query result, destructure `data.tickets` (use as the tickets list) and `data.supervisor_states`.
- Derive the visible-state base list:
  ```ts
  const supervisorStates = data?.supervisor_states ?? ['new', 'question', 'specd', 'blocked', 'implemented']
  ```
- In the `visibleStates` memo, replace `[...SUPERVISOR_STATES]` with `[...supervisorStates]`.
- Remove the `SUPERVISOR_STATES` import.

### Order of steps

1. Server endpoint first (unblocks UI work).
2. `supervisorUtils.ts` helper (pure function, easy to unit-test in isolation).
3. `SupervisorView.tsx` integration.

### Constraints / gotchas

- The `new` state currently has no `actionable` entries in `workflow.toml`. If it should remain visible in the supervisor panel, `actionable = ["supervisor"]` must be added to the `new` state in `.apm/workflow.toml`. Confirm with the author before implementing; update `workflow.toml` as part of this ticket if yes.
- The new endpoint is purely additive — no existing endpoints change behaviour.

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

### Open questions


### Amendment requests

- [x] Do not add a new /api/workflow/states endpoint. Instead, add a supervisor_states field to the existing /api/tickets response envelope. The workflow config is static for the server lifetime — no need for a separate endpoint for a single consumer.
- [x] Hardcode new as always visible in the supervisor panel and terminal states (closed) as never visible. These are structural states native to apm, not configuration-bound. Only non-structural states should derive visibility from the actionable field in workflow.toml. Update the Approach, AC, and supervisorUtils logic accordingly.

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
| 2026-04-07T18:52Z | in_design | specd | claude-0407-1847-5190 |
| 2026-04-07T19:08Z | specd | ready | apm |
| 2026-04-07T19:24Z | ready | in_progress | philippepascal |
