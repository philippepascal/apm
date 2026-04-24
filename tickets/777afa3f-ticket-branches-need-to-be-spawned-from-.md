+++
id = "777afa3f"
title = "ticket branches need to be spawned from default branch"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/777afa3f-ticket-branches-need-to-be-spawned-from-"
created_at = "2026-04-24T16:38:58.520269Z"
updated_at = "2026-04-24T16:41:43.672003Z"
+++

## Spec

### Problem

When APM creates ticket or epic worktree branches, the git base branch is hardcoded to `"main"` in several places. This means projects that use a different default branch (e.g. `master`, `develop`, `trunk`) will have their branches spawned from the wrong base, leading to incorrect diffs, merge conflicts, and broken CI pipelines.

The `default_branch` field already exists in `ProjectConfig` (loaded from `apm.toml`, defaults to `"main"`) and is correctly consumed in `start.rs` and `git_util.rs`. However, three locations in `epic.rs` and `new.rs` ignore it and hardcode `"main"` directly.

### Acceptance criteria

- [ ] When `default_branch = "develop"` is set in `apm.toml`, a new ticket worktree branch is fetched and created from `origin/develop`
- [ ] When `default_branch = "develop"` is set in `apm.toml`, a new epic worktree branch is fetched and created from `origin/develop`
- [ ] When `default_branch` is absent from `apm.toml`, ticket and epic branches continue to be created from `origin/main` (existing behaviour unchanged)
- [ ] The fallback branch name in `apm/src/cmd/new.rs` `open_editor()` resolves to the configured default branch, not the literal string `"main"`

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
| 2026-04-24T16:38Z | — | new | philippepascal |
| 2026-04-24T16:41Z | new | groomed | philippepascal |
| 2026-04-24T16:41Z | groomed | in_design | philippepascal |