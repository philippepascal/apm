+++
id = "de2588b4"
title = "Spawn paths consume apm prompt internally"
state = "new"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/de2588b4-spawn-paths-consume-apm-prompt-internall"
created_at = "2026-05-14T21:14:34.141790Z"
updated_at = "2026-05-14T21:21:29.757383Z"
depends_on = ["ba121f45"]
+++

## Spec

### Problem

Once `apm prompt` (ticket ba121f45) lands, integrate it into the three worker-spawn entry points:

1. `apm-core/src/start.rs::run` (handles `apm start --spawn <id>`)
2. `apm-core/src/start.rs::run_next` (handles `apm start --spawn --next` and `apm work` non-daemon)
3. `apm-core/src/start.rs::spawn_next_worker` (handles UI dispatch loop)

Each currently calls `resolve_system_prompt(...)` directly. Replace those call sites with a call to the new prompt-assembly function (or shell out to `apm prompt` and capture stdout — TBD in design).

Acceptance: all three spawn paths produce the same prompt as `apm prompt --ticket <id>` would print; running `apm prompt` then `apm start --spawn` for the same ticket shows identical prompts.

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