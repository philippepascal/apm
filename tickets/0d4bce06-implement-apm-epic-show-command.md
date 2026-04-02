+++
id = "0d4bce06"
title = "Implement apm epic show command"
state = "in_design"
priority = 6
effort = 0
risk = 0
author = "claude-0401-2145-a8f3"
agent = "philippepascal"
branch = "ticket/0d4bce06-implement-apm-epic-show-command"
created_at = "2026-04-01T21:55:14.006927Z"
updated_at = "2026-04-02T00:47:26.271809Z"
+++

## Spec

### Problem

`apm epic list` gives aggregate counts; engineers and the supervisor also need to drill into a specific epic to see individual ticket status, assignees, and dependency relationships.

The full design is in `docs/epics.md` (§ Commands — `apm epic show`). The command accepts a short epic ID (or unambiguous prefix) and prints: title, branch name, derived state, and a table of tickets each showing ID, title, current state, assigned agent, and `depends_on` entries. The derived state logic is the same as in `apm epic list`.

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
