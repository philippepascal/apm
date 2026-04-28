+++
id = "07d51d55"
title = "config has a max_default_branch_workers"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/07d51d55-config-has-a-max-default-branch-workers"
created_at = "2026-04-28T06:56:57.028226Z"
updated_at = "2026-04-28T07:27:22.868510Z"
+++

## Spec

### Problem

Currently APM has two parallelism controls: `max_concurrent` (global ceiling across all workers) and `max_workers_per_epic` (cap per epic). Tickets that belong to no epic ("default branch" work) are never individually capped — they can collectively fill every available slot up to `max_concurrent`.

This is a problem when a project mixes epics (branch-isolated features) with standalone tickets. Without a cap, a burst of non-epic tickets can monopolise the worker pool and starve ongoing epic work, or vice versa.

The fix is a new `[agents]` config field, `max_workers_on_default`, that limits how many workers may simultaneously run on non-epic tickets. A value of `0` means "no limit beyond `max_concurrent`" (existing behaviour). The default is `1`, matching the existing conservative default for `max_workers_per_epic`.

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
| 2026-04-28T06:56Z | — | new | philippepascal |
| 2026-04-28T07:13Z | new | groomed | philippepascal |
| 2026-04-28T07:27Z | groomed | in_design | philippepascal |