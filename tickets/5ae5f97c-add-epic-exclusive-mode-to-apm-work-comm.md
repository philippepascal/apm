+++
id = "5ae5f97c"
title = "Add --epic exclusive mode to apm work command"
state = "in_design"
priority = 6
effort = 0
risk = 0
author = "claude-0401-2145-a8f3"
agent = "77035"
branch = "ticket/5ae5f97c-add-epic-exclusive-mode-to-apm-work-comm"
created_at = "2026-04-01T21:55:49.406819Z"
updated_at = "2026-04-02T00:49:18.843305Z"
+++

## Spec

### Problem

The `apm work` engine currently dispatches any ready ticket regardless of epic membership. When working an epic, the supervisor wants to focus the engine exclusively on that epic's tickets and ignore free tickets.

The full design is in `docs/epics.md` (§ `apm work` — Exclusive mode). Adding `--epic <id>` filters candidates to `frontmatter.epic == id` before the priority sort. A config shorthand `[work] epic = "ab12cd34"` implies exclusive mode. No other modes (balanced, per-epic limits) are supported — the spec explicitly cuts them for simplicity. Dependency ordering within the epic still applies.

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
| 2026-04-01T22:00Z | new | groomed | claude-0401-2145-a8f3 |
| 2026-04-02T00:49Z | groomed | in_design | philippepascal |