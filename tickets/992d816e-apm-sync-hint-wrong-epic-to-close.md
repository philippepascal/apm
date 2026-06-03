+++
id = "992d816e"
title = "apm sync hint wrong epic to close"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/992d816e-apm-sync-hint-wrong-epic-to-close"
created_at = "2026-06-03T02:27:42.503993Z"
updated_at = "2026-06-03T06:34:40.733440Z"
+++

## Spec

### Problem

When `apm sync` computes which epics to list as "ready to close", it calls `is_branch_content_merged(root, default_branch, epic_branch)` for each epic branch. That function checks first whether `epic_branch` is a git ancestor of main (`git merge-base --is-ancestor epic main`). For any epic branch that was created from an old commit of main but never had development committed to it, the branch tip IS a literal ancestor of main — so the function returns `true` and the epic is added to `epic_close_hints`.

The result is that every stale, undeveloped epic branch (visible in `apm epic list` as `↓N clean`) is incorrectly listed as "Epics ready to close (apm epic close <id>)", while epics that have actual unmerged work (like the `done` epic whose branch is ahead of main) are correctly omitted. The user is prompted to run `apm epic close` on in-progress epics that have open tickets and no merged content.

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
| 2026-06-03T02:27Z | — | new | philippepascal |
| 2026-06-03T06:32Z | new | groomed | philippepascal |
| 2026-06-03T06:34Z | groomed | in_design | philippepascal |