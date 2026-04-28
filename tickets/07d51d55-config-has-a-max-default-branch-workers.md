+++
id = "07d51d55"
title = "config has a max_default_branch_workers"
state = "in_design"
priority = 0
effort = 3
risk = 2
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/07d51d55-config-has-a-max-default-branch-workers"
created_at = "2026-04-28T06:56:57.028226Z"
updated_at = "2026-04-28T15:08:40.481898Z"
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

**`apm-core/src/config.rs`**

1. Add `max_workers_on_default: usize` to `AgentsConfig`, with a serde default function returning `1`:
   ```rust
   #[serde(default = "default_max_workers_on_default")]
   pub max_workers_on_default: usize,

   fn default_max_workers_on_default() -> usize { 1 }
   ```

2. Add a method alongside `blocked_epics()`:
   ```rust
   pub fn is_default_branch_blocked(&self, active_epic_ids: &[Option<String>]) -> bool {
       if self.agents.max_workers_on_default == 0 {
           return false;
       }
       let count = active_epic_ids.iter().filter(|e| e.is_none()).count();
       count >= self.agents.max_workers_on_default
   }
   ```

**`apm-core/src/init.rs` — `default_config()` (~line 280)**

In the `[agents]` section of the hardcoded config template, add both `max_workers_per_epic` and `max_workers_on_default` so new projects have them explicit from the start:

```toml
[agents]
max_concurrent = 3
max_workers_per_epic = 1
max_workers_on_default = 1
instructions = ".apm/agents.md"
```

**`apm-core/src/start.rs` — `run_next()` (~line 365)**

After the existing `blocked_epics()` call, compute and apply the default-branch limit:
```rust
let blocked = config.blocked_epics(&active_epic_ids);
let default_blocked = config.is_default_branch_blocked(&active_epic_ids);
let tickets: Vec<_> = all_tickets.into_iter()
    .filter(|t| match t.frontmatter.epic.as_deref() {
        Some(eid) => !blocked.iter().any(|b| b == eid),
        None => !default_blocked,
    })
    .collect();
```

**`apm-core/src/start.rs` — `spawn_next_worker()`**

Inspect whether `active_epic_ids` is already available inside this function or passed as a parameter. If it is available (recomputed internally), add the same `is_default_branch_blocked()` call and update the candidate filter identically. If it receives a pre-computed `blocked_epics: Vec<String>` only, add a `default_blocked: bool` parameter and thread the value through from the call site in `work.rs`.

**`apm/src/cmd/work.rs` — daemon spawn loop (~line 111)**

After computing `blocked_epics` from active workers' epic IDs, also call `config.is_default_branch_blocked()` with the same `active_epic_ids` slice, and pass the resulting bool to `spawn_next_worker()` (if that function requires it as a parameter per the step above).

**Tests**

Add unit tests in `config.rs` (or a dedicated test module) for `is_default_branch_blocked()` covering the four cases in the acceptance criteria. No integration-test changes are required unless existing tests assert on the current "non-epic tickets are never blocked" behaviour.

### Open questions


### Amendment requests

- [ ] apm init should set both max_workers_per_epic and max_workers_on_default to their default (1).

### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-04-28T06:56Z | — | new | philippepascal |
| 2026-04-28T07:13Z | new | groomed | philippepascal |
| 2026-04-28T07:27Z | groomed | in_design | philippepascal |
| 2026-04-28T07:31Z | in_design | specd | claude-0428-0727-2e28 |
| 2026-04-28T15:06Z | specd | ammend | philippepascal |
| 2026-04-28T15:08Z | ammend | in_design | philippepascal |