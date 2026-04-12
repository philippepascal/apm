+++
id = "db874c60"
title = "Replace raw git calls in init.rs, start.rs, and worktree.rs with git_util helpers"
state = "groomed"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/db874c60-replace-raw-git-calls-in-init-rs-start-r"
created_at = "2026-04-12T17:29:31.936764Z"
updated_at = "2026-04-12T17:30:43.918442Z"
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
| 2026-04-12T17:29Z | — | new | philippepascal |
| 2026-04-12T17:30Z | new | groomed | apm |
