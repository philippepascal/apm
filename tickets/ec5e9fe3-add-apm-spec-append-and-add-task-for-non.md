+++
id = "ec5e9fe3"
title = "Add apm spec --append and --add-task for non-destructive section updates"
state = "groomed"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/ec5e9fe3-add-apm-spec-append-and-add-task-for-non"
created_at = "2026-04-27T22:17:27.580621Z"
updated_at = "2026-04-27T22:55:14.081682Z"
+++

## Spec

### Problem

`apm spec <id> --section <name> --set` and `--set-file` replace the entire section content. This is destructive for sections that accumulate over time as a decision record — specifically `Amendment requests` and `Open questions` (`.apm/agents.md`: "Do not delete answered questions or checked amendment items — they are the decision record"). The only non-destructive writer today is `--mark`, which is read-only on items it doesn't match. There is no constructive complement.

Real incident: during ticket 941e57fa amendment, calling `apm spec --set-file` to write new amendment requests erased a previously-checked amendment item from a prior round. Recovery required reading the prior commit on the ticket branch and re-stitching the content manually.

Proposed additions to `apm spec`:

1. **`--append <text>`** and **`--append-file <path>`** — generic appenders. Append the given content to the existing section with a newline separator. Works for any section type (`free`, `qa`, `tasks`). Auto-commits to the ticket branch like `--set`.

2. **`--add-task <text>`** — typed sugar for sections with `type = "tasks"` (Acceptance criteria, Amendment requests). Appends `- [ ] <text>` to the list. Errors out cleanly if invoked on a non-tasks section, catching writer mistakes early.

Behavior on missing section: `--set` today creates the section if absent; the appenders should match that behavior so they're drop-in safe.

Implementation lives in `apm-core/src/spec.rs` (where `set_section` already lives) and the CLI handler in `apm/src/cmd/spec.rs`.

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
| 2026-04-27T22:17Z | — | new | philippepascal |
| 2026-04-27T22:55Z | new | groomed | philippepascal |
