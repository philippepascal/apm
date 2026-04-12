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

`ensure_amendment_section()` is currently defined in `apm-core/src/state.rs` as a `pub fn`. Its sole job is to insert a `### Amendment requests` section into a ticket body string when one is not already present — a pure document-formatting operation with no knowledge of state machine logic.

This placement is wrong. `state.rs` owns state machine transitions; `review.rs` owns spec-document-level operations (`split_body`, `extract_spec`, `normalize_amendments`, `apply_review`). The function is called once inside `transition()` when the new state is `"ammend"`, but it does not depend on any state module internals — it only needs a `&mut String`.

Moving it completes the cleanup started in ticket 4004f5dc and satisfies section 6 of REFACTOR-CORE.md: `review.rs` becomes the single home for all spec-document manipulation.

### Acceptance criteria

- [ ] `ensure_amendment_section` is defined in `apm-core/src/review.rs` and not in `apm-core/src/state.rs`
- [ ] `ensure_amendment_section` is `pub` in `review.rs` (accessible as `apm_core::review::ensure_amendment_section`)
- [ ] `state::transition()` calls `review::ensure_amendment_section` (not a local function) when transitioning to `"ammend"`
- [ ] `apm state <id> ammend` still inserts `### Amendment requests` into a ticket body that lacks the section
- [ ] `apm state <id> ammend` is a no-op on the amendment section when `### Amendment requests` is already present
- [ ] `review.rs` has tests for `ensure_amendment_section` covering: body already contains the section (no-op), body contains `### Out of scope` (inserts after the block), body contains `## History` but no `### Out of scope` (inserts before history), body contains neither (appends to end)
- [ ] All pre-existing tests in `state.rs` and `review.rs` continue to pass

### Out of scope

- No behavioral changes to `ensure_amendment_section` — the function logic is moved verbatim
- No changes to any other function in `state.rs` or `review.rs`
- No changes to the public API surface of `apm_core::state` beyond removing `ensure_amendment_section` from that namespace
- No changes to `apm/src/cmd/state.rs` or any CLI layer (the call stays inside `apm_core::state::transition`)
- Adding tests for other existing `review.rs` functions is out of scope

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