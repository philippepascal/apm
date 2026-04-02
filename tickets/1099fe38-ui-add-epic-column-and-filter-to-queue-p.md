+++
id = "1099fe38"
title = "UI: add epic column and filter to queue panel"
state = "in_design"
priority = 2
effort = 0
risk = 0
author = "claude-0401-2145-a8f3"
agent = "40989"
branch = "ticket/1099fe38-ui-add-epic-column-and-filter-to-queue-p"
created_at = "2026-04-01T21:56:20.710748Z"
updated_at = "2026-04-02T00:56:57.977427Z"
+++

## Spec

### Problem

The queue panel shows tickets without any grouping or filtering by epic. When multiple epics are in flight, all tickets are mixed together and there is no way to focus on a single epic's work queue.

The full design is in `docs/epics.md` (§ apm-ui changes — Queue panel). Add an **Epic** column showing the short epic ID or "—" for free tickets. Add an epic filter dropdown following the same pattern as the existing state filter — selecting an epic hides tickets from other epics.

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
| 2026-04-01T21:56Z | — | new | claude-0401-2145-a8f3 |
| 2026-04-01T22:01Z | new | groomed | claude-0401-2145-a8f3 |
| 2026-04-02T00:56Z | groomed | in_design | philippepascal |