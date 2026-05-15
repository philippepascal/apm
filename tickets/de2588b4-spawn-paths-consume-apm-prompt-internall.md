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
| 2026-05-14T21:14Z | — | new | philippe|philippepascal |
| 2026-05-14T21:22Z | new | groomed | philippe |
| 2026-05-15T01:39Z | groomed | in_design | philippe |