+++
id = "056b1ee1"
title = "Require epic quiescence in apm epic close"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/056b1ee1-require-epic-quiescence-in-apm-epic-clos"
created_at = "2026-04-27T20:29:06.958516Z"
updated_at = "2026-04-27T21:32:50.263787Z"
epic = "5ea30227"
target_branch = "epic/5ea30227-strategy-and-dependency-hardening"
depends_on = ["2973e208"]
+++

## Spec

### Problem

`apm epic close` currently gates on a state check: it refuses if any epic ticket is not in a `satisfies_deps: true` or `terminal` state. This check is too narrow — it does not account for live worker processes and does not use the shared quiescence definition established by ticket 2973e208.

The spec at `docs/strategy-and-dependencies.md` (§ 'Refresh and close: epic must be quiescent') requires the epic to be fully quiescent before the close PR is opened: no ticket may be in an active, non-terminal state, and no ticket may have a live worker process. Ticket 2973e208 adds `epic_is_quiescent()` in `apm-core/src/epic.rs` as the canonical helper for this check, used by both `apm refresh-epic` and `apm epic close`.

This ticket wires that helper into `run_close`, replacing the existing bespoke gate logic with a single call to `epic_is_quiescent()`.

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
| 2026-04-27T20:29Z | — | new | philippepascal |
| 2026-04-27T20:44Z | new | groomed | philippepascal |
| 2026-04-27T21:32Z | groomed | in_design | philippepascal |