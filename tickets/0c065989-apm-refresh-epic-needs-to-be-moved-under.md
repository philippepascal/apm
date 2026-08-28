+++
id = "0c065989"
title = "apm refresh-epic needs to be moved under apm epic refresh for consistency"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/0c065989-apm-refresh-epic-needs-to-be-moved-under"
created_at = "2026-08-28T00:50:30.208076Z"
updated_at = "2026-08-28T07:24:17.681175Z"
+++

## Spec

### Problem

`apm refresh-epic <id>` is a top-level command, even though every other
epic-scoped operation (`new`, `submit`, `close`, `list`, `show`, `set`) lives
under `apm epic <subcommand>`. This is the only epic operation that breaks the
`apm epic ...` convention, which makes the CLI surface harder to discover
(`apm epic --help` doesn't mention it) and inconsistent with the rest of the
command tree documented in `apm main.rs`'s help template and `apm help
commands`.

The command should be relocated to `apm epic refresh <id>` so all epic
operations are grouped under one subcommand namespace, with identical
behaviour and flags. No backward-compatible alias is kept for the old
top-level name, per the project's no-shim convention.

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
| 2026-08-28T00:50Z | — | new | philippepascal |
| 2026-08-28T07:13Z | new | groomed | philippepascal |
| 2026-08-28T07:24Z | groomed | in_design | philippepascal |