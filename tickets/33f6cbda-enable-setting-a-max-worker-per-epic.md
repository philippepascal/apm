+++
id = "33f6cbda"
title = "enable setting a max worker per epic"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
branch = "ticket/33f6cbda-enable-setting-a-max-worker-per-epic"
created_at = "2026-04-07T19:08:03.080608Z"
updated_at = "2026-04-07T19:19:28.676504Z"
+++

## Spec

### Problem

APM supports a global `[agents] max_concurrent` setting that caps the total number of simultaneously running workers. When using `apm work`, this global cap applies uniformly across all epics. There is currently no way to say "this epic should run at most N workers at the same time", which matters when:

- One epic is I/O-bound or API-rate-limited and flooding it with workers makes things worse.
- A project has multiple epics with different parallelism needs (e.g. a fast-moving feature epic vs. a careful refactor epic).
- A supervisor wants to throttle an in-flight epic without pausing all work.

The desired behaviour is: users can assign a `max_workers` ceiling to a specific epic, and `apm work` will not spawn more than that many concurrent workers for tickets belonging to that epic, regardless of the global `max_concurrent` limit.

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
| 2026-04-07T19:08Z | — | new | philippepascal |
| 2026-04-07T19:08Z | new | groomed | apm |
| 2026-04-07T19:19Z | groomed | in_design | philippepascal |