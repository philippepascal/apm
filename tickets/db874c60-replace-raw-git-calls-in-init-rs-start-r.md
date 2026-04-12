+++
id = "db874c60"
title = "Replace raw git calls in init.rs, start.rs, and worktree.rs with git_util helpers"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/db874c60-replace-raw-git-calls-in-init-rs-start-r"
created_at = "2026-04-12T17:29:31.936764Z"
updated_at = "2026-04-12T17:44:25.861153Z"
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

How the implementation will work.

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-04-12T17:29Z | — | new | philippepascal |
| 2026-04-12T17:30Z | new | groomed | apm |
| 2026-04-12T17:44Z | groomed | in_design | philippepascal |