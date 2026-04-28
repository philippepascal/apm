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

- [ ] `[agents] max_workers_on_default` is accepted in `.apm/config.toml` without error
- [ ] When the field is absent, it defaults to `1`
- [ ] When set to `1`, a second non-epic ticket is not picked by `apm start --next` while one non-epic worker is already active
- [ ] When set to `2`, a third non-epic ticket is not picked while two non-epic workers are active
- [ ] When set to `0`, non-epic tickets are picked freely up to `max_concurrent` (no additional cap)
- [ ] Epic-linked tickets are unaffected: `max_workers_per_epic` still governs them independently
- [ ] The `apm work` daemon respects the limit in its spawn loop (non-epic slots are counted each iteration)
- [ ] A unit test covers: limit=1, 0 active non-epic workers → not blocked
- [ ] A unit test covers: limit=1, 1 active non-epic worker → blocked
- [ ] A unit test covers: limit=0, any number of active non-epic workers → not blocked
- [ ] A unit test covers: active workers are all epic-linked → non-epic slot is not blocked

### Out of scope

- Changes to `max_workers_per_epic` behaviour
- Changing the default value of `max_concurrent`
- Surfacing the default-branch worker count in `apm ps`, `apm show`, or other display commands
- Any UI or config validation beyond accepting the field and applying the limit

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