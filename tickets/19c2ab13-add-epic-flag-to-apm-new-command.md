+++
id = "19c2ab13"
title = "Add --epic flag to apm new command"
state = "in_design"
priority = 6
effort = 0
risk = 0
author = "claude-0401-2145-a8f3"
agent = "64496"
branch = "ticket/19c2ab13-add-epic-flag-to-apm-new-command"
created_at = "2026-04-01T21:55:26.992429Z"
updated_at = "2026-04-02T00:49:01.961954Z"
+++

## Spec

### Problem

Currently `apm new` always creates ticket branches from `main` (or the default branch) and writes no epic-related fields to frontmatter. For tickets that belong to an epic, the ticket branch must instead be created from the epic branch tip, and the frontmatter must carry `epic` and `target_branch` so that downstream commands (`apm start`, PR creation) know where to target. An optional `depends_on` field lets a ticket declare that it must not be dispatched until listed tickets are implemented.

The full design is in `docs/epics.md`. This ticket adds the `--epic <id>` flag (and `--depends-on`) to `apm new`. Without the flag, `apm new` behaviour is completely unchanged.

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
| 2026-04-02T00:49Z | groomed | in_design | philippepascal |