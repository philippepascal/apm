+++
id = "c36a4bf6"
title = "Move embedded assets to src/default/"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/c36a4bf6-move-embedded-assets-to-src-default"
created_at = "2026-04-12T06:04:13.294338Z"
updated_at = "2026-04-12T06:12:48.194890Z"
epic = "57bce963"
target_branch = "epic/57bce963-refactor-apm-core-module-structure"
+++

## Spec

### Problem

The `apm-core/src/` directory mixes Rust source files with embedded template/config assets (`ticket.toml`, `workflow.toml`, `apm.worker.md`, `apm.spec-writer.md`, `apm.agents.md`). These files are included via `include_str!()` in `init.rs` but sit at the same level as code modules, making the source tree harder to scan.

See [REFACTOR-CORE.md](../../REFACTOR-CORE.md) section 1 for the full plan.

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
| 2026-04-12T06:04Z | — | new | philippepascal |
| 2026-04-12T06:11Z | new | groomed | apm |
| 2026-04-12T06:12Z | groomed | in_design | philippepascal |
