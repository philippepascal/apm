+++
id = "a6367b87"
title = "Move ensure_amendment_section from state.rs to review.rs"
state = "closed"
priority = 0
effort = 1
risk = 1
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/a6367b87-move-ensure-amendment-section-from-state"
created_at = "2026-04-12T06:04:43.474175Z"
updated_at = "2026-04-12T08:48:48.063795Z"
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

- [x] `ensure_amendment_section` is defined in `apm-core/src/review.rs` and not in `apm-core/src/state.rs`
- [x] `ensure_amendment_section` is `pub` in `review.rs` (accessible as `apm_core::review::ensure_amendment_section`)
- [x] `state::transition()` calls `review::ensure_amendment_section` (not a local function) when transitioning to `"ammend"`
- [x] `apm state <id> ammend` still inserts `### Amendment requests` into a ticket body that lacks the section
- [x] `apm state <id> ammend` is a no-op on the amendment section when `### Amendment requests` is already present
- [x] `review.rs` has tests for `ensure_amendment_section` covering: body already contains the section (no-op), body contains `### Out of scope` (inserts after the block), body contains `## History` but no `### Out of scope` (inserts before history), body contains neither (appends to end)
- [x] All pre-existing tests in `state.rs` and `review.rs` continue to pass

### Out of scope

- No behavioral changes to `ensure_amendment_section` — the function logic is moved verbatim
- No changes to any other function in `state.rs` or `review.rs`
- No changes to the public API surface of `apm_core::state` beyond removing `ensure_amendment_section` from that namespace
- No changes to `apm/src/cmd/state.rs` or any CLI layer (the call stays inside `apm_core::state::transition`)
- Adding tests for other existing `review.rs` functions is out of scope

### Approach

All changes are in `apm-core/src/`.

**`review.rs`**
- Copy `ensure_amendment_section` verbatim from `state.rs` and add it as a `pub fn` in `review.rs`. No new imports are needed — the function uses only `String` methods from std.
- Add a `#[cfg(test)]` block (or extend the existing one) with four test cases:
  - `already_has_section`: body contains `### Amendment requests` → function returns early, body unchanged
  - `inserts_after_out_of_scope`: body has `### Out of scope\n\n- x\n\n## History` → amendment block inserted between the out-of-scope block and `## History`
  - `inserts_before_history_no_out_of_scope`: body has `## History` but no `### Out of scope` → amendment block inserted immediately before `## History`
  - `appends_when_no_anchor`: body has neither anchor → amendment block appended at end

**`state.rs`**
- Remove the `ensure_amendment_section` function definition (currently at lines 321–338 on the worktree branch).
- Add `review` to the existing `use crate::` import group, or add `use crate::review;` as a separate line.
- Change the call site in `transition()` from `ensure_amendment_section(&mut t.body)` to `review::ensure_amendment_section(&mut t.body)`.

No other files change. The function is only called from within `state::transition()`; no external crate references it by path.

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-04-12T06:04Z | — | new | philippepascal |
| 2026-04-12T06:12Z | new | groomed | apm |
| 2026-04-12T06:45Z | groomed | in_design | philippepascal |
| 2026-04-12T06:48Z | in_design | specd | claude-0412-0645-ef88 |
| 2026-04-12T07:13Z | specd | ready | apm |
| 2026-04-12T08:03Z | ready | in_progress | philippepascal |
| 2026-04-12T08:06Z | in_progress | implemented | claude-0412-0803-cb48 |
| 2026-04-12T08:48Z | implemented | closed | philippepascal |
