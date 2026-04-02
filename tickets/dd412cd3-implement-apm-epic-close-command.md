+++
id = "dd412cd3"
title = "Implement apm epic close command"
state = "in_design"
priority = 6
effort = 0
risk = 0
author = "claude-0401-2145-a8f3"
agent = "philippepascal"
branch = "ticket/dd412cd3-implement-apm-epic-close-command"
created_at = "2026-04-01T21:55:18.313179Z"
updated_at = "2026-04-02T00:47:54.397879Z"
+++

## Spec

### Problem

When all tickets in an epic are implemented, the epic branch must be merged to `main` as a single coherent unit. There is currently no command to initiate this — engineers would have to create the PR manually.

The full design is in `docs/epics.md` (§ Commands — `apm epic close`). The command runs `gh pr create` from the epic branch targeting `main`. It does not merge — merging requires human approval as usual. The command should refuse (with a clear error) if not all tickets are `implemented` or later, since merging a partial epic would leave incomplete work on main.

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
