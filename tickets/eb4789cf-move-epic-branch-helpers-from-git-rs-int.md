+++
id = "eb4789cf"
title = "Move epic branch helpers from git.rs into epic.rs"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/eb4789cf-move-epic-branch-helpers-from-git-rs-int"
created_at = "2026-04-12T06:04:33.586819Z"
updated_at = "2026-04-12T06:32:39.867887Z"
epic = "57bce963"
target_branch = "epic/57bce963-refactor-apm-core-module-structure"
depends_on = ["b28fe914"]
+++

## Spec

### Problem

`epic.rs` currently contains `derive_epic_state()` and `create()` but the epic branch discovery functions (`find_epic_branch`, `find_epic_branches`, `epic_branches`, `create_epic_branch`) live in `git.rs`. These are epic-domain operations that happen to call git commands, not general git utilities. They should live alongside the rest of the epic logic.

See [REFACTOR-CORE.md](../../REFACTOR-CORE.md) section 8 for the full plan.

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
| 2026-04-12T06:12Z | new | groomed | apm |
| 2026-04-12T06:32Z | groomed | in_design | philippepascal |
