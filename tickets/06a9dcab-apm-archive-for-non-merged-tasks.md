+++
id = "06a9dcab"
title = "apm archive for non merged tasks"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/06a9dcab-apm-archive-for-non-merged-tasks"
created_at = "2026-04-28T07:11:58.042694Z"
updated_at = "2026-04-28T07:31:23.624834Z"
+++

## Spec

### Problem

apm archive --older-than 5d
warning: tickets/056b1ee1-require-epic-quiescence-in-apm-epic-clos.md is in non-terminal state 'implemented' — skipping

but

apm show 056b1ee1
From https://github.com/philippepascal/apm
 * branch              ticket/056b1ee1-require-epic-quiescence-in-apm-epic-clos -> FETCH_HEAD
056b1ee1 — Require epic quiescence in apm epic close
state:    closed
priority: 0  effort: 2  risk: 2
branch:   ticket/056b1ee1-require-epic-quiescence-in-apm-epic-clos
epic:         5ea30227
target_branch: epic/5ea30227-strategy-and-dependency-hardening
depends_on:   2973e208
owner:        philippepascal

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
| 2026-04-28T07:11Z | — | new | philippepascal |
| 2026-04-28T07:13Z | new | groomed | philippepascal |
| 2026-04-28T07:31Z | groomed | in_design | philippepascal |
