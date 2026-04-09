+++
id = "ab531177"
title = "add an apm command to clean epics"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/ab531177-add-an-apm-command-to-clean-epics"
created_at = "2026-04-09T05:07:02.660761Z"
updated_at = "2026-04-09T05:18:26.521041Z"
+++

## Spec

### Problem

Epics accumulate over time as a project progresses. Once all tickets in an epic reach a terminal state (`derive_epic_state` returns `"done"`), the epic branch and its `.apm/epics.toml` entry serve no further purpose but remain in the repository indefinitely. There is currently no way to remove them short of manual `git branch -d` and hand-editing `.apm/epics.toml`.

This ticket adds `apm epic clean` — a subcommand that identifies all "done" epics, presents the list to the user, and deletes them (local branch + metadata entry) after confirmation. A `--yes` flag allows non-interactive use, and `--dry-run` lets users preview what would be removed without side effects.

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
| 2026-04-09T05:07Z | — | new | philippepascal |
| 2026-04-09T05:17Z | new | groomed | apm |
| 2026-04-09T05:18Z | groomed | in_design | philippepascal |