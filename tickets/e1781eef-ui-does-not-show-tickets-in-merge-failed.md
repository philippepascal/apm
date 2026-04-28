+++
id = "e1781eef"
title = "UI does not show tickets in merge_failed state"
state = "in_design"
priority = 0
effort = 3
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/e1781eef-ui-does-not-show-tickets-in-merge-failed"
created_at = "2026-04-28T22:26:52.277291Z"
updated_at = "2026-04-28T22:41:41.313451Z"
+++

## Spec

### Problem

The local `.apm/workflow.toml` does not include a `merge_failed` state entry. The state was added to `apm-core/src/default/workflow.toml` in commit `a7bce26b`, but the project-local config was never updated.

When `apm-server` handles `GET /api/tickets`, it builds `supervisor_states` by scanning only `cfg.workflow.states` — the locally loaded config. Because `merge_failed` is absent from that config, it is absent from `supervisor_states` in the API response (`apm-server/src/handlers/tickets.rs`, lines 52–57). `SupervisorView.tsx` uses `supervisorStates` (from the API) as the source of truth for which swimlane columns to render; no entry means no column is created. The tickets themselves ARE present in the payload (they are not terminal and therefore survive the `tickets.retain(…)` filter), but the UI creates no column to hold them, so they silently vanish from the board.

Two secondary gaps compound the problem:

- `ALL_WORKFLOW_STATES` in `SupervisorView.tsx` (lines 8–20) is a hardcoded list used exclusively for the state filter dropdown. `merge_failed` is not in it, so the supervisor cannot filter for that state to discover the hidden tickets.
- `stateColors.ts` has no entry for `merge_failed`. If the ticket did appear, it would render with the default gray rather than a visually prominent error colour.

The hardcoded fallback `supervisor_states` in `tickets.rs` (lines 41–44), used when config loading fails, also omits `merge_failed`, so the invisibility persists even in degraded mode.

### Acceptance criteria

- [ ] A ticket in `merge_failed` state appears as a swimlane column in the supervisor board when at least one such ticket exists, without any changes to `.apm/workflow.toml`.
- [ ] The `merge_failed` swimlane column uses the RED colour scheme (same as `blocked`), signalling that supervisor action is required.
- [ ] The state filter dropdown in SupervisorView lists `merge_failed` as a selectable option when at least one such ticket exists (dropdown is derived from `supervisorStates` rather than a hardcoded array).
- [ ] When workflow config fails to load, the server fallback still includes `merge_failed` in `supervisor_states`.
- [ ] Any other non-terminal ticket state present in the ticket list but absent from the workflow config is automatically surfaced as a swimlane column, with no further code changes required.
- [ ] The `/api/tickets` response payload includes tickets in `merge_failed` state (server does not strip them).

### Out of scope

- Adding `merge_failed` to the project's local `.apm/workflow.toml` (operational concern; not a code bug).
- Designing or implementing a `/api/workflow` endpoint.
- Changes to `apm list` (already works correctly).
- Fixing the operational ticket 63f5e6d2 that triggered this report.
- Changing the state machine logic in `apm-core/src/state.rs` (already correct — it correctly writes `merge_failed` on merge failure).

### Approach

Three files change.

---

### `apm-server/src/handlers/tickets.rs`

**Change 1 — Fallback list** (lines 41–44): add `"merge_failed"` so degraded mode still surfaces it:

```rust
let fallback_supervisor_states = || vec![
    "new".to_string(), "question".to_string(), "specd".to_string(),
    "blocked".to_string(), "implemented".to_string(), "merge_failed".to_string(),
];
```

**Change 2 — Catch-all pass**: make `supervisor_states` mutable in the destructure at line 45, then insert a scan immediately after the match block closes (after line 67) and before `tickets.retain(…)` (line 68). The scan appends any non-terminal ticket state not already in `supervisor_states`, making the board resilient to any future engine state not yet in the local config:

```rust
// Change the binding at line 45 to mut:
let (resolved_ids, terminal_ids, mut supervisor_states): (Vec<String>, Vec<String>, Vec<String>) = …;

// Insert after line 67, before line 68:
{
    let sup_set: std::collections::HashSet<&str> =
        supervisor_states.iter().map(|s| s.as_str()).collect();
    let term_set: std::collections::HashSet<&str> =
        terminal_ids.iter().map(|s| s.as_str()).collect();
    let mut seen = std::collections::HashSet::<String>::new();
    for t in &tickets {
        let s = t.frontmatter.state.clone();
        if !sup_set.contains(s.as_str()) && !term_set.contains(s.as_str()) && seen.insert(s.clone()) {
            supervisor_states.push(s);
        }
    }
}
```

Running this before `tickets.retain(…)` means the scan operates on the full, unfiltered ticket list.

---

### `apm-ui/src/lib/stateColors.ts`

Add `merge_failed: RED` to `STATE_COLORS` immediately after the existing `blocked: RED` entry (line 47):

```typescript
blocked: RED,
merge_failed: RED,
```

---

### `apm-ui/src/components/supervisor/SupervisorView.tsx`

1. Delete the `ALL_WORKFLOW_STATES` constant (lines 8–20).

2. Add a `dropdownStates` memo below the existing `visibleStates` memo (after line 102). It derives the filter-dropdown options from `supervisorStates` plus `'closed'`:

```typescript
const dropdownStates = useMemo(() => {
    return [...supervisorStates, 'closed']
}, [supervisorStates])
```

3. Replace `ALL_WORKFLOW_STATES.map(…)` at line 206 with `dropdownStates.map(…)`.

No changes required to `visibleStates`, `columns`, or any Swimlane component — they already derive correctly from `supervisorStates`. After the server fix the column appears automatically; after the UI fix the dropdown lists it.

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-04-28T22:26Z | — | new | philippepascal |
| 2026-04-28T22:30Z | new | groomed | philippepascal |
| 2026-04-28T22:35Z | groomed | in_design | philippepascal |