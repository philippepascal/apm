+++
id = "dd412cd3"
title = "Implement apm epic close command"
state = "in_design"
priority = 6
effort = 0
risk = 0
author = "claude-0401-2145-a8f3"
agent = "27501"
branch = "ticket/dd412cd3-implement-apm-epic-close-command"
created_at = "2026-04-01T21:55:18.313179Z"
updated_at = "2026-04-02T00:47:54.397879Z"
+++

## Spec

### Problem

There is no command to create a PR from an epic branch to `main`. When an engineering team finishes all tickets in an epic, the epic branch must be merged to `main` as a coherent unit. Currently this requires running `gh pr create` manually, knowing the exact branch name and base branch.

`apm epic close <id>` should automate this: look up the epic branch by its short ID, verify that every ticket in the epic is in `implemented` or a later state, then run `gh pr create --base main --head epic/<id>-<slug>` and print the PR URL. The command does not merge — merging is left to human reviewers on GitHub.

Without this command the epic workflow is incomplete: tickets can be created (`apm new --epic`), listed (`apm epic list/show`), but never promoted to a PR as a group.

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