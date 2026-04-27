+++
id = "056b1ee1"
title = "Require epic quiescence in apm epic close"
state = "new"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/056b1ee1-require-epic-quiescence-in-apm-epic-clos"
created_at = "2026-04-27T20:29:06.958516Z"
updated_at = "2026-04-27T20:29:06.958516Z"
epic = "5ea30227"
target_branch = "epic/5ea30227-strategy-and-dependency-hardening"
depends_on = ["2973e208"]
+++

## Spec

### Problem

`apm epic close` currently opens a PR from the epic to the default branch without checking whether tickets in the epic are still being worked on. The spec at `docs/strategy-and-dependencies.md` (section 'Refresh and close: epic must be quiescent') requires the epic to be quiescent first: no ticket in `in_design`, `in_progress`, or with a live worker.

Reuse the `epic_is_quiescent()` helper added in ticket 2973e208 (refresh-epic). On non-quiescence, `apm epic close` must refuse with a clear message naming the offending tickets and their states.

See docs/strategy-and-dependencies.md, section 'Refresh and close: epic must be quiescent'.

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
