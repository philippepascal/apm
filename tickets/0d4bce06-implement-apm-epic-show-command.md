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

- [ ] `apm epic show <id>` prints a header block with the epic title, branch name, and derived state
- [ ] `apm epic show <id>` prints a table of associated tickets, one row per ticket, showing: short ID, title, current state, assigned agent (or — if none), and `depends_on` entries (or — if none)
- [ ] Tickets with no `epic` frontmatter field set to the epic's ID are not shown in the table
- [ ] A 4-or-more character prefix that uniquely identifies one epic branch is accepted and resolves correctly
- [ ] An ambiguous prefix (matches more than one epic branch) exits non-zero and prints a list of the matching branch names
- [ ] An ID or prefix that matches no epic branch exits non-zero with a clear error message
- [ ] Derived state follows the rules in `docs/epics.md`: no tickets → `empty`; any ticket `in_design` or `in_progress` → `in_progress`; all tickets `implemented` or later → `implemented`; all tickets `accepted` or `closed` → `done`; otherwise → `in_progress`
- [ ] `apm epic show` with no argument prints usage and exits non-zero
- [ ] Adding `epic`, `target_branch`, and `depends_on` optional fields to `Frontmatter` does not break serialisation of any existing ticket that lacks those fields (they are omitted from output when `None`)

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