+++
id = "74f9a232"
title = "apm list should have a way to format output to use in pipes"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/74f9a232-apm-list-should-have-a-way-to-format-out"
created_at = "2026-06-11T01:10:29.686451Z"
updated_at = "2026-06-11T01:16:40.055105Z"
+++

## Spec

### Problem

`apm list` produces a human-readable table (columns: id, state, owner, base, title) plus footer blocks for stale warnings and recovery hints. This output is hostile to pipes — extracting ticket IDs requires something like `awk '{print $1}' | sed 's/^\*//'`, which breaks whenever the stale marker or column alignment changes.

Users need a machine-readable output mode so that `apm list` results can feed directly into shell scripts, other `apm` commands, and automation pipelines. The most important use case is a flat comma-separated list of ticket IDs.

### Acceptance criteria

- [ ] `apm list --format ids` prints a comma-separated list of ticket IDs on a single line (e.g. `74f9a232,3a1b2c3d`)
- [ ] `apm list --format ids` with no matching tickets prints an empty line and exits 0
- [ ] `apm list --format ids` respects all existing filter flags (`--state`, `--unassigned`, `--actionable`, `--mine`, `--author`, `--owner`, `--all`)
- [ ] `apm list --format ids` omits the stale-ticket footer, diverged-ticket warning, and recovery hint block
- [ ] `apm list --format json` prints a JSON array of objects, each containing the ticket's frontmatter fields
- [ ] `apm list --format json` with no matching tickets prints `[]`
- [ ] `apm list --format json` omits the stale-ticket footer, diverged-ticket warning, and recovery hint block
- [ ] `apm list` without `--format` produces identical output to the current behaviour
- [ ] `apm list --format <unknown>` exits non-zero with a message naming the supported values

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
| 2026-06-11T01:10Z | — | new | philippepascal |
| 2026-06-11T01:12Z | new | groomed | philippepascal |
| 2026-06-11T01:16Z | groomed | in_design | philippepascal |