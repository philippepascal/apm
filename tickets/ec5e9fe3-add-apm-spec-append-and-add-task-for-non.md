+++
id = "ec5e9fe3"
title = "Add apm spec --append and --add-task for non-destructive section updates"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/ec5e9fe3-add-apm-spec-append-and-add-task-for-non"
created_at = "2026-04-27T22:17:27.580621Z"
updated_at = "2026-04-27T23:05:18.250272Z"
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

- [ ] **`--append` / `--append-file`**

- [ ] `apm spec <id> --append <text>` without `--section` exits non-zero with an error containing `"--append requires --section"`
- [ ] `apm spec <id> --section <name> --append <text>` appends the trimmed text after the existing section content, separated by a single newline
- [ ] When the target section is empty or absent, `--append` creates it with the new text (no leading newline)
- [ ] `apm spec <id> --section <name> --append-file <path>` reads the file at `<path>` and appends its contents to the section identically to `--append`
- [ ] `--append-file` without `--section` exits non-zero with an error containing `"--append-file requires --section"`
- [ ] Supplying both `--append` and `--set` (or `--set-file`) exits with a clap conflict error
- [ ] Supplying both `--append` and `--append-file` exits with a clap conflict error
- [ ] When config is active and the section has a defined type, `--append` applies `apply_section_type` formatting to the appended text before committing (consistent with `--set`)
- [ ] `--append` commits to the ticket branch with message `ticket(<id>): append to section <name>`
- [ ] When aggressive sync is enabled, `--append` pushes to origin after the commit; a push failure prints a warning but does not fail the command

- [ ] **`--add-task`**

- [ ] `apm spec <id> --add-task <text>` without `--section` exits non-zero with an error containing `"--add-task requires --section"`
- [ ] `apm spec <id> --section <name> --add-task <text>` appends `- [ ] <text>` to the named section
- [ ] When the target section is empty or absent, `--add-task` creates it with `- [ ] <text>` as its sole item
- [ ] When config is active and the named section has `type != "tasks"`, `--add-task` exits non-zero with an error that names the actual section type
- [ ] `--add-task` commits to the ticket branch with message `ticket(<id>): add task to <name>`
- [ ] When aggressive sync is enabled, `--add-task` pushes to origin after the commit; a push failure prints a warning but does not fail the command
- [ ] Supplying `--add-task` together with `--set`, `--set-file`, `--append`, or `--append-file` exits with a clap conflict error

### Out of scope

- Stdin input via `-` for `--append` (only `--set` supports stdin today; no regression, just not extended)\n- Type validation for `--add-task` when no `[ticket.sections]` config is present — no config means no type to validate; the item is appended unconditionally\n- Changes to existing flag behaviour: `--set`, `--set-file`, `--mark`, `--check` are untouched\n- Bulk append (appending to multiple sections in one invocation)\n- A blank-line separator variant for `--append` (single newline is the separator)

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
| 2026-04-27T23:05Z | groomed | in_design | philippepascal |