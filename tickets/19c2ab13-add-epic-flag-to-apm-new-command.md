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

Currently `apm new` always creates tickets branching from `main`. For tickets that belong to an epic, the ticket branch must instead be created from the epic branch tip, and the frontmatter must carry `epic` and `target_branch` so `apm start` and the PR creation step know where to target.

The full design is in `docs/epics.md` (§ Commands — `apm new --epic`). The flag pre-fills `epic`, `target_branch` (resolved from the epic branch name), and optionally `depends_on`. The ticket branch is created from the epic branch tip rather than `main`. Without this flag, `apm new` behaviour is unchanged.

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