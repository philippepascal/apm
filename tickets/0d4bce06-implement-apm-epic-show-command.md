+++
id = "0d4bce06"
title = "Implement apm epic show command"
state = "in_design"
priority = 6
effort = 0
risk = 0
author = "claude-0401-2145-a8f3"
agent = "7772"
branch = "ticket/0d4bce06-implement-apm-epic-show-command"
created_at = "2026-04-01T21:55:14.006927Z"
updated_at = "2026-04-02T00:47:26.271809Z"
+++

## Spec

### Problem

Engineers and supervisors can see aggregate ticket counts via `apm epic list` (not yet implemented), but there is no way to drill into a specific epic to inspect individual ticket status, assignees, and dependency relationships. Without `apm epic show`, diagnosing blocked epics, tracking down the assigned agent for a specific ticket, or checking whether `depends_on` prerequisites have been met requires manual branch browsing.

The full design for this command is in `docs/epics.md` (§ Commands — `apm epic show`). The command accepts a short epic ID (or an unambiguous prefix) and prints: title, branch name, derived state, and a table of associated tickets with columns for ID, title, current state, assigned agent, and `depends_on` entries.

Two related pieces of infrastructure must land with this ticket because `apm epic show` depends on them and neither exists yet:
1. The `Frontmatter` struct does not have `epic`, `target_branch`, or `depends_on` fields; without the `epic` field there is no way to filter tickets by epic.
2. There is no CLI `epic` subcommand; the new `Epic { Show { ... } }` command variant and its dispatch must be added to `apm/src/main.rs`.

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