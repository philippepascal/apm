+++
id = "e1781eef"
title = "UI does not show tickets in merge_failed state"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/e1781eef-ui-does-not-show-tickets-in-merge-failed"
created_at = "2026-04-28T22:26:52.277291Z"
updated_at = "2026-04-28T22:35:01.519626Z"
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
| 2026-04-28T22:26Z | — | new | philippepascal |
| 2026-04-28T22:30Z | new | groomed | philippepascal |
| 2026-04-28T22:35Z | groomed | in_design | philippepascal |