+++
id = "aeacd066"
title = "Move branch_to_title and epic ID parsing to apm_core::epic"
state = "ready"
priority = 0
effort = 3
risk = 2
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/aeacd066-move-branch-to-title-and-epic-id-parsing"
created_at = "2026-04-12T09:02:36.908517Z"
updated_at = "2026-04-12T10:24:17.696434Z"
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

- Renaming or restructuring any other functions in apm_core::epic beyond the two new helpers\n- Moving any apm-server logic other than parse_epic_branch (e.g. EpicSummary, find_epic_branch wrappers)\n- Changing the behaviour of branch_to_title or epic_id_from_branch (pure refactor, no logic changes)\n- Adding branch_to_title or epic_id_from_branch to the public re-export of apm-core if it is not already done for other epic helpers\n- Fixing the apm-server Option handling beyond the minimal substitution needed to remove parse_epic_branch

### Approach

**Step 1 — Add helpers to `apm-core/src/epic.rs`**

Add two public functions (after `create_epic_branch` is a good location):

```rust
/// Convert an epic branch name to a display title.
/// `epic/57bce963-refactor-apm-core` -> `"Refactor Apm Core"`
pub fn branch_to_title(branch: &str) -> String {
    let rest = branch.trim_start_matches("epic/");
    let slug = match rest.find('-') {
        Some(pos) => &rest[pos + 1..],
        None => rest,
    };
    slug.split('-')
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                None => String::new(),
                Some(c) => c.to_uppercase().to_string() + chars.as_str(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

/// Extract the ID segment from an epic branch name.
/// `epic/57bce963-refactor-apm-core` -> `"57bce963"`
/// Strips the `epic/` prefix if present, then returns everything before the first `-`.
/// Returns the whole string if there is no `-`.
pub fn epic_id_from_branch(branch: &str) -> &str {
    let rest = branch.trim_start_matches("epic/");
    match rest.find('-') {
        Some(pos) => &rest[..pos],
        None => rest,
    }
}
```

Move the existing `branch_to_title` unit-test block from `apm/src/cmd/epic.rs` (lines 365-388) into `apm-core/src/epic.rs` under `#[cfg(test)]`. Add new tests for `epic_id_from_branch`:

- happy path: `"epic/57bce963-refactor-apm-core"` -> `"57bce963"`
- no `epic/` prefix: `"57bce963-refactor"` -> `"57bce963"`
- no dash at all: `"nodash"` -> `"nodash"`

**Step 2 — Update `apm/src/cmd/epic.rs`**

- Delete the `branch_to_title` fn definition (lines 342-363) and its test block (lines 365-388).
- Add `use apm_core::epic::{branch_to_title, epic_id_from_branch};` or use fully qualified paths.
- Replace every inline ID-parsing occurrence (found in `run_close`, `run_show`, `run_set`, lines ~75-77 and similar):

  Before:
  ```
  let after_prefix = epic_branch.trim_start_matches("epic/");
  let epic_id = after_prefix.split('-').next().unwrap_or("");
  ```
  After:
  ```
  let epic_id = epic_id_from_branch(&epic_branch);
  ```

**Step 3 — Update `apm/src/cmd/clean.rs`**

Replace all three inline ID-parsing blocks (lines ~189-191, ~216-218, ~248-250):

Before:
```
let after_prefix = branch.trim_start_matches("epic/");
let id_end = after_prefix.find('-').unwrap_or(after_prefix.len()).min(8);
let id = &after_prefix[..id_end];
```
After:
```
let id = apm_core::epic::epic_id_from_branch(branch);
```
Note: the `.min(8)` cap is redundant because epic IDs are always exactly 8 hex chars; the new helper is equivalent for all valid branch names.

Also replace the `crate::cmd::epic::branch_to_title(branch)` call (~line 219) with `apm_core::epic::branch_to_title(branch)`.

**Step 4 — Update `apm-server/src/main.rs`**

Delete `parse_epic_branch()` (lines 186-203). At each call site, replace the tuple return with two separate calls:

Before:
```
let (id, title) = parse_epic_branch(&branch).unwrap_or_default();
```
After:
```
let id    = apm_core::epic::epic_id_from_branch(&branch).to_string();
let title = apm_core::epic::branch_to_title(&branch);
```
Adjust error handling as needed based on how the None case was previously handled.

**Step 5 — Verify**

Run `cargo test` in the workspace root. The relocated tests in `apm-core` cover `branch_to_title`; new tests cover `epic_id_from_branch`. All three crates must compile and their tests must pass.

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-04-12T09:02Z | — | new | philippepascal |
| 2026-04-12T09:09Z | new | groomed | apm |
| 2026-04-12T09:14Z | groomed | in_design | philippepascal |
| 2026-04-12T09:18Z | in_design | specd | claude-0412-0915-2aa0 |
| 2026-04-12T10:24Z | specd | ready | apm |
