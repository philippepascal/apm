+++
id = "2973e208"
title = "Add apm refresh-epic command with epic quiescence check"
state = "new"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/2973e208-add-apm-refresh-epic-command-with-epic-q"
created_at = "2026-04-27T20:28:30.358011Z"
updated_at = "2026-04-27T20:28:30.358011Z"
epic = "5ea30227"
target_branch = "epic/5ea30227-strategy-and-dependency-hardening"
+++

## Spec

### Problem

There is no built-in way to pull default-branch updates into a long-running epic branch. The spec at `docs/strategy-and-dependencies.md` (section 'Refresh and close: epic must be quiescent') defines `apm refresh-epic <id>` as the supervisor-facing tool for this.

Implementation:
- New `apm refresh-epic <id>` subcommand
- Open a PR from the default branch into `epic/<id>-<slug>` via `gh pr create --base epic/<id>-<slug> --head <default>`
- PR title and body auto-generated from the diff range (commits on default not yet in the epic)
- Refuse the operation if the epic is not quiescent: any ticket in the epic is in `in_design`, `in_progress`, or has a live worker. Define a shared `epic_is_quiescent()` helper to be reused by `apm epic close` (separate ticket).

The supervisor reviews and merges the PR manually. APM does not stop running workers; it only enforces the precondition.

See docs/strategy-and-dependencies.md, section 'Refresh and close: epic must be quiescent'.

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
| 2026-04-27T20:28Z | — | new | philippepascal |
