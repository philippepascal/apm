+++
id = "a6367b87"
title = "Move ensure_amendment_section from state.rs to review.rs"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/a6367b87-move-ensure-amendment-section-from-state"
created_at = "2026-04-12T06:04:43.474175Z"
updated_at = "2026-04-12T06:45:02.710821Z"
epic = "57bce963"
target_branch = "epic/57bce963-refactor-apm-core-module-structure"
depends_on = ["4004f5dc"]
+++

## Spec

### Problem

`ensure_amendment_section()` lives in `state.rs` but it manipulates the spec document (adding/formatting the amendment request section). It belongs in `review.rs` alongside `split_body`, `extract_spec`, `normalize_amendments`, and `apply_review` — the module that owns all spec-document-level operations.

This is the final cleanup after trimming `state.rs` (4004f5dc).

See [REFACTOR-CORE.md](../../REFACTOR-CORE.md) section 6 for the full plan.

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
| 2026-04-12T06:45Z | groomed | in_design | philippepascal |
