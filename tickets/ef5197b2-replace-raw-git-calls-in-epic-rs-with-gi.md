+++
id = "ef5197b2"
title = "Replace raw git calls in epic.rs with git_util helpers"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/ef5197b2-replace-raw-git-calls-in-epic-rs-with-gi"
created_at = "2026-04-12T17:29:30.028375Z"
updated_at = "2026-04-12T17:41:21.960808Z"
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

Explicit list of what this ticket does not cover.

### Approach

How the implementation will work.

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-04-12T17:29Z | — | new | philippepascal |
| 2026-04-12T17:30Z | new | groomed | apm |
| 2026-04-12T17:41Z | groomed | in_design | philippepascal |