+++
id = "ef5197b2"
title = "Replace raw git calls in epic.rs with git_util helpers"
state = "in_design"
priority = 0
effort = 2
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/ef5197b2-replace-raw-git-calls-in-epic-rs-with-gi"
created_at = "2026-04-12T17:29:30.028375Z"
updated_at = "2026-04-12T17:43:53.169708Z"
epic = "6062f74f"
target_branch = "epic/6062f74f-consolidate-git-operations-into-git-util"
depends_on = ["061d0ac1"]
+++

## Spec

### Problem

epic.rs has 5 raw `Command::new("git")` calls (excluding test helpers) that should use git_util or worktree helpers:

1. `git fetch origin main` (line ~207) — `git_util::fetch_branch()` already exists, should use it
2. `git worktree add -b {branch} {path} origin/main` (line ~230) — `worktree::add_worktree()` already exists but with a different signature; either reuse or add a variant to git_util
3. `git add EPIC.md` (line ~253) — `git_util::stage_files()`
4. `git commit -m {msg}` (line ~262) — `git_util::commit()`
5. `git worktree remove --force {path}` (line ~272) — `worktree::remove_worktree()` already exists

Two of these (fetch_branch, remove_worktree) are direct replacements with existing helpers. The remaining three need the new helpers from the prerequisite ticket.

After this ticket, epic.rs should have zero raw git commands in production code (test helpers with `git init` etc. are acceptable).

Depends on the git_util helpers ticket landing first.

### Acceptance criteria

- [ ] `epic::create()` contains zero `Command::new("git")` calls in production code after the refactor\n- [ ] The fetch step calls `git_util::fetch_branch(root, "main")`\n- [ ] The worktree-add step calls `git_util::run(root, &["worktree", "add", "-b", &branch, &wt_path_str, "origin/main"])`\n- [ ] The stage step calls `git_util::stage_files(&wt_path, &["EPIC.md"])`\n- [ ] The commit step calls `git_util::commit(&wt_path, &commit_msg)`\n- [ ] The worktree-removal step calls `worktree::remove_worktree(root, &wt_path, true)` and its result is still ignored (non-fatal)\n- [ ] All existing `epic` integration tests pass with no changes to test helpers\n- [ ] `cargo clippy -p apm-core` reports no new warnings

### Out of scope

- Adding new git helpers beyond what is consumed here (those belong in the prerequisite ticket 061d0ac1)\n- Changing the behaviour of `epic::create()` — this is a pure refactor\n- Refactoring raw git calls in test helpers (e.g. `git init` setup in tests)\n- Touching any file outside `apm-core/src/epic.rs`\n- Modifying `worktree::add_worktree()` signature to accept a start-point argument

### Approach

All five replacements are in `epic::create()` in `apm-core/src/epic.rs`. No other files need to change.\n\nPrerequisite: ticket 061d0ac1 must be merged into the epic target branch first so that `git_util::stage_files` and `git_util::commit` are available.\n\n**Call 1 — fetch (lines ~207-217)**\nReplace the raw `Command::new("git") ... fetch origin main` block with:\n```rust\ngit_util::fetch_branch(root, "main")?;\n```\n`fetch_branch` already exists in `git_util.rs` and has the right signature: `(root: &Path, branch: &str) -> Result<()>`.\n\n**Call 2 — worktree add (lines ~230-247)**\nThe existing `worktree::add_worktree(root, wt_path, branch)` does not accept a start-point; it creates the branch from HEAD. The epic flow requires branching from `origin/main`, so use `git_util::run()` directly:\n```rust\ngit_util::run(root, &["worktree", "add", "-b", &branch, &wt_path.to_string_lossy(), "origin/main"])?;\n```\n\n**Call 3 — stage EPIC.md (lines ~253-259)**\nReplace with the helper from 061d0ac1:\n```rust\ngit_util::stage_files(&wt_path, &["EPIC.md"])?;\n```\n\n**Call 4 — commit (lines ~262-268)**\nReplace with the helper from 061d0ac1:\n```rust\ngit_util::commit(&wt_path, &commit_msg)?;\n```\n\n**Call 5 — worktree remove (lines ~272-275)**\nThe current code uses `let _ = ...` to ignore failures. Preserve that semantics:\n```rust\nlet _ = worktree::remove_worktree(root, &wt_path, true);\n```\n`remove_worktree` signature: `(root: &Path, wt_path: &Path, force: bool) -> Result<()>`.\n\nAdd/adjust `use` imports at the top of `epic.rs` to bring in `git_util` and `worktree` as needed; remove the `std::process::Command` import if it is no longer referenced outside of test helpers.

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-04-12T17:29Z | — | new | philippepascal |
| 2026-04-12T17:30Z | new | groomed | apm |
| 2026-04-12T17:41Z | groomed | in_design | philippepascal |