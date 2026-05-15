+++
id = "de2588b4"
title = "Spawn paths consume apm prompt internally"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/de2588b4-spawn-paths-consume-apm-prompt-internall"
created_at = "2026-05-14T21:14:34.141790Z"
updated_at = "2026-05-15T01:39:42.342332Z"
depends_on = ["ba121f45"]
+++

## Spec

### Problem

The three worker-spawn entry points in `apm-core/src/start.rs` each call `resolve_system_prompt(...)` directly. Once ticket ba121f45 lands it renames that function to `build_system_prompt`, adds a per-agent file at Level 0 of the cascade, and exposes `apm prompt <id>` as a CLI that calls the same function. After ba121f45 merges, the three call sites must reference `build_system_prompt`; any site still calling `resolve_system_prompt` will fail to compile.

The secondary concern is parity: `apm prompt <id>` is designed (per ba121f45 Step 2) to resolve the ticket's triggering transition and invoke `build_system_prompt` with the same cascade as the spawn paths. For that guarantee to hold the spawn paths must call `build_system_prompt` through the same argument-construction logic, not a parallel copy. This ticket ensures that the rename and any parity gap are addressed in one place after ba121f45 is merged.

### Acceptance criteria

- [ ] After this ticket merges, `apm-core` compiles without referencing `resolve_system_prompt` anywhere outside of test history or comments
- [ ] For any ticket in a spawnable state, `apm start --spawn <id>` passes the same system-prompt string to the worker subprocess as `apm prompt <id>` prints to stdout
- [ ] For any ticket picked up by `run_next`, the system prompt written to the temp file equals the output of `apm prompt <id>` for that ticket
- [ ] For any ticket dispatched by `spawn_next_worker`, the system prompt written to the temp file equals the output of `apm prompt <id>` for that ticket
- [ ] If `build_system_prompt` returns an error (e.g. a missing instructions file), each spawn path exits non-zero and surfaces the error message unchanged
- [ ] All existing unit tests that previously referenced `resolve_system_prompt` by name pass after being updated to reference `build_system_prompt`

### Out of scope

- Adding or changing the `build_system_prompt` function itself (ba121f45)
- Adding the `apm prompt` CLI command (ba121f45)
- Changing the priority cascade or per-agent file Level 0 logic (ba121f45)
- Shelling out to `apm prompt` as a subprocess — the spawn paths call `build_system_prompt` directly as a library function
- Changes to argument-construction code in the spawn paths beyond the function-name substitution
- Modifying any spawn-path behavior other than the system-prompt call

### Approach

How the implementation will work.

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-05-14T21:14Z | — | new | philippe|philippepascal |
| 2026-05-14T21:22Z | new | groomed | philippe |
| 2026-05-15T01:39Z | groomed | in_design | philippe |