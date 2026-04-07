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

- [ ] `apm epic set-max-workers <epic-id> <N>` writes `max_workers = N` into the `[epics."<epic-id>"]` table in `.apm/config.toml`
- [ ] `apm epic set-max-workers <epic-id> 0` (or `--unset`) removes the `max_workers` field from that table, restoring uncapped behaviour
- [ ] `apm epic show <epic-id>` prints the current `max_workers` limit when one is set
- [ ] `apm work` (without `--epic`) respects each epic's `max_workers` limit: it does not spawn a new worker for a ticket whose epic already has `max_workers` active workers
- [ ] `apm work --epic <id>` also respects the `max_workers` limit for that epic
- [ ] When a running worker finishes and a slot opens up, `apm work` spawns the next eligible ticket in that epic (normal pick-next behaviour resumes)
- [ ] Tickets with no epic, or whose epic has no `max_workers` set, are unaffected — they are still bounded only by `[agents] max_concurrent`
- [ ] Setting `max_workers` greater than `[agents] max_concurrent` is allowed but has no additional effect (the global cap still binds)
- [ ] `apm epic set-max-workers` with a non-existent epic ID prints an error and exits non-zero
- [ ] `apm epic set-max-workers` with a value ≤ 0 (other than the unset sentinel) prints an error and exits non-zero

### Out of scope

- Setting a global default `max_workers` that applies to all epics without an explicit override
- Per-ticket concurrency limits (only epic-level granularity is in scope)
- Dynamically adjusting `max_workers` while `apm work` is already running (takes effect on next loop iteration only; no hot-reload)
- Surfacing per-epic worker counts in `apm epic list`
- Any UI (apm-ui) changes
- Migrating or deprecating the existing `[agents] max_concurrent` global setting

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