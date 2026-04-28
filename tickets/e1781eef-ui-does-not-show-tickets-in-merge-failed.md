+++
id = "e1781eef"
title = "UI does not show tickets in merge_failed state"
state = "groomed"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/e1781eef-ui-does-not-show-tickets-in-merge-failed"
created_at = "2026-04-28T22:26:52.277291Z"
updated_at = "2026-04-28T22:30:24.327084Z"
+++

## Spec

### Problem

`apm list` correctly shows tickets in state `merge_failed`, but the UI (apm-server / apm-ui supervisor board) does not surface them. A ticket can land in `merge_failed` and stay invisible to the supervisor in the UI until they happen to run `apm list` from the CLI.

**Concrete incident:** ticket 63f5e6d2 ("UI: epics filter fixes") merged failed because the main worktree had uncommitted local changes that would have been overwritten. State went to `merge_failed`. `apm list` showed it correctly:

```
63f5e6d2 [merge_failed] philippepascal main UI: epics filter fixes
```

The supervisor board in the UI did not show the ticket at all — neither as a flagged "needs attention" entry nor in any column.

**Likely cause:** the UI probably filters by a known set of states (e.g. `new`, `groomed`, `specd`, `ready`, `in_progress`, `implemented`, `ammend`, `question`, `blocked`, `closed`) and silently drops tickets in any other state. `merge_failed` was added by commit `a7bce26b` ("Add merge_failed state and catch merge errors in transition") but the UI's state list was not updated.

**What this ticket should do:**

1. Audit `apm-ui/src/` for hardcoded state lists / state-keyed colour maps / column-mapping logic that would exclude unknown states. Likely suspects: `SupervisorView.tsx`, swimlane / column definitions, ticket-card colour logic, `useLayoutStore` or similar filter state.
2. Make the UI render any state present in `workflow.toml`, including states the UI was not specifically authored for. Either (a) drop the hardcoded inclusion list entirely and render whatever the API returns, or (b) read state metadata from a `/api/workflow` endpoint.
3. Specifically: `merge_failed` tickets must be visible to the supervisor with a clear visual marker that the supervisor needs to act (it is the only path back to `implemented` or `in_progress`).
4. Also audit `apm-server` endpoints to ensure they do not strip / filter `merge_failed` from the response payload before the UI ever sees it.

**Out of scope:**

- Adding `merge_failed` to the user's project workflow.toml (separate operational concern; not a code bug).
- Designing the `/api/workflow` endpoint if approach (b) is chosen — that may merit its own ticket if it is non-trivial.
- Changes to `apm list` (already correct).
- Fixing the ticket itself that triggered this report (operational, not in scope).

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
