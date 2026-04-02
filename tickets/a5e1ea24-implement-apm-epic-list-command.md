+++
id = "a5e1ea24"
title = "Implement apm epic list command"
state = "in_design"
priority = 6
effort = 0
risk = 0
author = "claude-0401-2145-a8f3"
agent = "philippepascal"
branch = "ticket/a5e1ea24-implement-apm-epic-list-command"
created_at = "2026-04-01T21:55:09.722953Z"
updated_at = "2026-04-02T00:47:06.221425Z"
+++

## Spec

### Problem

Once epic branches exist there is no way to see them or their status at a glance. Engineers and the supervisor need to know which epics are active, how many tickets are in each state, and whether an epic is done.

The full design is in `docs/epics.md` (§ Commands — `apm epic list`). Epic state is always derived — never stored — using these rules: no tickets → `empty`; any ticket `in_design` or `in_progress` → `in_progress`; all `implemented` or later → `implemented`; all `accepted`/`closed` → `done`; otherwise → `in_progress`.

The command lists all `epic/*` remote branches and for each shows: short ID, title (from slug), derived state, and per-state ticket counts (e.g. `2 in_progress, 1 ready, 3 implemented`).

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
| 2026-04-01T21:55Z | — | new | claude-0401-2145-a8f3 |
| 2026-04-01T21:59Z | new | groomed | claude-0401-2145-a8f3 |
| 2026-04-02T00:47Z | groomed | in_design | philippepascal |
