+++
id = "aeacd066"
title = "Move branch_to_title and epic ID parsing to apm_core::epic"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/aeacd066-move-branch-to-title-and-epic-id-parsing"
created_at = "2026-04-12T09:02:36.908517Z"
updated_at = "2026-04-12T09:14:58.898130Z"
epic = "1b029f52"
target_branch = "epic/1b029f52-refactor-apm-cli-code-organization"
+++

## Spec

### Problem

Several domain-logic helpers are defined in CLI command files (`apm/src/cmd/`) instead of in `apm-core` where they belong:

1. **`branch_to_title()`** in `apm/src/cmd/epic.rs` (lines ~343-363) — converts an epic branch name like `epic/57bce963-refactor-apm-core` to a display title `"Refactor Apm Core"`. This is epic-domain logic that `apm-server` also needs (it has its own inline version in `main.rs`). It belongs in `apm_core::epic`.

2. **Epic ID parsing from branch name** — the pattern `branch.trim_start_matches("epic/").split('-').next()` appears in `epic.rs` (lines 76-77) and `clean.rs` (lines 189, 216, 248). This should be a single helper in `apm_core::epic`, e.g., `fn epic_id_from_branch(branch: &str) -> &str`.

Moving these to `apm_core` eliminates duplication between `apm` and `apm-server` and puts domain logic in the library where it belongs.

### Acceptance criteria

- [ ] `apm_core::epic::branch_to_title(branch: &str) -> String` exists and converts `epic/<id>-some-slug` to `"Some Slug"`
- [ ] `apm_core::epic::epic_id_from_branch(branch: &str) -> &str` exists and returns the ID segment (before the first `-` after the `epic/` prefix)
- [ ] Both functions are exported from `apm_core` (`pub fn`)
- [ ] `apm/src/cmd/epic.rs` no longer defines its own `branch_to_title`; all call sites use `apm_core::epic::branch_to_title`
- [ ] `apm/src/cmd/epic.rs` no longer inlines the `split('-').next()` ID-parsing pattern; all call sites use `apm_core::epic::epic_id_from_branch`
- [ ] `apm/src/cmd/clean.rs` no longer inlines the `.trim_start_matches("epic/") … .find('-') … .min(8)` pattern; all three occurrences use `apm_core::epic::epic_id_from_branch`
- [ ] `apm-server/src/main.rs` no longer defines `parse_epic_branch`; its callers use `apm_core::epic::branch_to_title` and `apm_core::epic::epic_id_from_branch`
- [ ] Unit tests for `branch_to_title` (currently in `apm/src/cmd/epic.rs` lines 365–388) are moved into `apm_core/src/epic.rs`
- [ ] Unit tests for `epic_id_from_branch` covering the happy path and the no-dash edge case are added in `apm_core/src/epic.rs`
- [ ] `cargo test` passes across all three crates (`apm-core`, `apm`, `apm-server`)

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
| 2026-04-12T09:02Z | — | new | philippepascal |
| 2026-04-12T09:09Z | new | groomed | apm |
| 2026-04-12T09:14Z | groomed | in_design | philippepascal |