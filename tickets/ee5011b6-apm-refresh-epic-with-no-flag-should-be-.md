+++
id = "ee5011b6"
title = "apm refresh-epic with no flag should be interactive and propose several merge action"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/ee5011b6-apm-refresh-epic-with-no-flag-should-be-"
created_at = "2026-06-16T18:06:05.166190Z"
updated_at = "2026-06-16T18:09:33.758773Z"
+++

## Spec

### Problem

When `apm refresh-epic <id>` is run with no action flag (`--merge`, `--pr`, or `--auto`), the command prints a one-line status message and exits without doing anything. On an interactive terminal this is unhelpful: the user already knows there are commits to pull in, and now must re-type the command with the right flag to act on that information.

The fix is to turn the no-flag path into an interactive prompt when stdout is a terminal. The command should show the same status it already computes, then offer a numbered menu of the same actions the flags expose, read the user's choice, and execute it. Non-interactive callers (pipes, headless agents) keep the current print-and-exit behaviour so no scripted usage breaks.

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
| 2026-06-16T18:06Z | — | new | philippepascal |
| 2026-06-16T18:09Z | new | groomed | philippepascal |
| 2026-06-16T18:09Z | groomed | in_design | philippepascal |