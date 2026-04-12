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

The `apm-core/src/` directory mixes Rust source files with five embedded template/config assets: `apm.agents.md`, `apm.spec-writer.md`, `apm.worker.md`, `ticket.toml`, and `workflow.toml`. These files are compiled into the binary via `include_str!()` in `init.rs` and written to the user's `.apm/` directory during `apm init`. Because they live at the same level as the `.rs` modules, scanning the source tree for code files requires mentally filtering out non-code assets.\n\nMoving these assets to `apm-core/src/default/` groups all embedded defaults in one place, making the source layout self-documenting: `src/*.rs` is code, `src/default/` is data.

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