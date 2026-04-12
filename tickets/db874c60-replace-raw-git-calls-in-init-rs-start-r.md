+++
id = "db874c60"
title = "Replace raw git calls in init.rs, start.rs, and worktree.rs with git_util helpers"
state = "specd"
priority = 0
effort = 3
risk = 2
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/db874c60-replace-raw-git-calls-in-init-rs-start-r"
created_at = "2026-04-12T17:29:31.936764Z"
updated_at = "2026-04-12T17:48:37.126630Z"
epic = "6062f74f"
target_branch = "epic/6062f74f-consolidate-git-operations-into-git-util"
depends_on = ["061d0ac1"]
+++

## Spec

### Problem

Three modules have raw git calls that should go through git_util:

**init.rs** (6 calls):
- `git symbolic-ref --short HEAD` — git_util already has `current_branch()` which does the same thing via `branch --show-current`; use it
- `git rev-parse HEAD` — git_util already has `has_commits()`; use it
- `git add .apm/config.toml ...` — `git_util::stage_files()`
- `git commit -m "apm: initialize project"` — `git_util::commit()`
- Test helpers (`git init`, `git config`) — acceptable to keep raw since they bootstrap repos for testing

**start.rs** (3 calls):
- `git config {key}` — `git_util::git_config_get()`
- `git rev-parse --verify origin/{merge_base}` — `git_util::local_branch_exists()` (or a ref-exists variant)
- `git merge {ref} --no-edit` — `git_util::merge_ref()`

**worktree.rs** (1 call):
- `git ls-files --error-unmatch {path}` — `git_util::is_file_tracked()`

After this ticket, none of these files should import `std::process::Command` for git operations. Test helpers in init.rs that run `git init` are exempt.

Depends on the git_util helpers ticket landing first.

### Acceptance criteria

- [ ] `init.rs::detect_default_branch` calls `git_util::current_branch()` instead of `git symbolic-ref --short HEAD`
- [ ] `init.rs::maybe_initial_commit` calls `git_util::has_commits()` instead of `git rev-parse HEAD`
- [ ] `init.rs::maybe_initial_commit` calls `git_util::stage_files()` instead of `git add`
- [ ] `init.rs::maybe_initial_commit` calls `git_util::commit()` instead of `git commit -m`
- [ ] `start.rs::git_config_value` delegates to `git_util::git_config_get()` (or is removed and call sites call `git_util::git_config_get` directly)
- [ ] The inline `rev-parse --verify origin/<merge_base>` check in `start.rs` is replaced by a call to an existing `git_util` helper (e.g. `remote_branch_tip`)
- [ ] The inline `git merge <ref> --no-edit` block in `start.rs` is replaced by `git_util::merge_ref()`
- [ ] `worktree.rs::is_tracked` delegates to `git_util::is_file_tracked()` instead of running `git ls-files --error-unmatch` directly
- [ ] `use std::process::Command` is absent from `worktree.rs` (no remaining usages in that file)
- [ ] `use std::process::Command` is absent from the non-test portion of `init.rs`; if the test helpers still require it the import is scoped to `#[cfg(test)]`
- [ ] `std::process::Command` is not used in `start.rs` for git operations
- [ ] All existing unit and integration tests pass

### Out of scope

- Adding new helpers to `git_util.rs` — covered by the dependency ticket 061d0ac1
- Replacing the `git init` / `git config user.*` calls inside `init.rs` test helpers — explicitly exempt per the problem statement
- Replacing raw `Command` calls that already live inside `git_util.rs` itself
- Any raw git calls in files other than `init.rs`, `start.rs`, and `worktree.rs`
- Behavioural changes — this is a pure refactor; observable outputs must stay identical

### Approach

This ticket is a straight substitution: swap each raw `Command` block for the corresponding `git_util` helper, adjust imports, and verify tests. All helpers come from ticket 061d0ac1 — do not start until that branch is merged into the epic branch.

### init.rs (`apm-core/src/init.rs`)

**Imports (line 3):** Remove `use std::process::Command;` from module level. If the `#[cfg(test)]` block still needs `Command` for its `git_init` helper, add `use std::process::Command;` inside the test module instead. Add `use crate::git_util;` (or import individual helpers).

**`detect_default_branch` (lines 177–188):** Replace the `Command::new("git").args(["symbolic-ref", "--short", "HEAD"])...` expression with:
```rust
crate::git_util::current_branch(root)
    .ok()
    .filter(|s| !s.is_empty())
    .unwrap_or_else(|| "main".to_string())
```
`current_branch` uses `branch --show-current`; detached-HEAD produces an empty string, same as the original, and the fallback to `"main"` is preserved.

**`maybe_initial_commit` (lines 320–346):**

- Lines 321–326 (has_commits check): Replace with:
  ```rust
  if crate::git_util::has_commits(root) {
      return Ok(());
  }
  ```
- Lines 332–335 (git add): Replace with:
  ```rust
  crate::git_util::stage_files(root, &[
      ".apm/config.toml", ".apm/workflow.toml", ".apm/ticket.toml", ".gitignore",
  ])?;
  ```
- Lines 337–344 (git commit): Replace with:
  ```rust
  if crate::git_util::commit(root, "apm: initialize project").is_ok() {
      messages.push("Created initial commit.".to_string());
  }
  ```
  The original silently ignored commit failure; `is_ok()` preserves that behaviour.

### start.rs (`apm-core/src/start.rs`)

**`git_config_value` (lines 62–72):** Replace the function body with:
```rust
fn git_config_value(root: &Path, key: &str) -> Option<String> {
    crate::git_util::git_config_get(root, key)
}
```
Alternatively delete the wrapper entirely and inline `crate::git_util::git_config_get(root, key)` at both call sites (lines ~93 and ~97).

**Remote-ref existence check (lines 263–274):** Replace the `rev-parse --verify origin/<merge_base>` block. `remote_branch_tip` already exists in `git_util` and returns `Option<String>` — use it to probe the remote-tracking ref:
```rust
let ref_to_merge = if crate::git_util::remote_branch_tip(&wt_display, merge_base).is_some() {
    format!("origin/{merge_base}")
} else {
    merge_base.to_string()
};
```
Note: rename the local variable from `merge_ref` to `ref_to_merge` to avoid a collision with the `git_util::merge_ref` function called immediately after.

**Inline merge block (lines 275–300):** Replace the entire `match Command::new("git").args(["merge", merge_ref, "--no-edit"])...` expression with:
```rust
let merge_message = crate::git_util::merge_ref(&wt_display, &ref_to_merge, &mut warnings);
```
`git_util::merge_ref` returns `Option<String>` — `Some(message)` on a real merge, `None` on already-up-to-date or failure (warning pushed into `warnings`). This matches the contract the surrounding code expects.

Ensure no `std::process::Command` usages remain in `start.rs`.

### worktree.rs (`apm-core/src/worktree.rs`)

**`is_tracked` (lines 122–131):** Replace the function body:
```rust
fn is_tracked(root: &Path, path: &str) -> bool {
    crate::git_util::is_file_tracked(root, path)
}
```
Remove `use std::process::Command;` (line 3) — no other production usages remain in this file.

### Verification

Run `cargo test -p apm-core`. The init integration tests exercise `maybe_initial_commit` against real repos and should pass unchanged. If any test explicitly constructs `Command` objects for git in non-test code, that is a sign something was missed.

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-04-12T17:29Z | — | new | philippepascal |
| 2026-04-12T17:30Z | new | groomed | apm |
| 2026-04-12T17:44Z | groomed | in_design | philippepascal |
| 2026-04-12T17:48Z | in_design | specd | claude-0412-1744-3858 |
