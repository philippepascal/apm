+++
id = "777afa3f"
title = "ticket branches need to be spawned from default branch"
state = "in_design"
priority = 0
effort = 2
risk = 2
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/777afa3f-ticket-branches-need-to-be-spawned-from-"
created_at = "2026-04-24T16:38:58.520269Z"
updated_at = "2026-04-24T16:55:06.159367Z"
+++

## Spec

### Problem

When APM creates ticket or epic worktree branches, the git base branch is hardcoded to `"main"` in several places. This means projects that use a different default branch (e.g. `master`, `develop`, `trunk`) will have their branches spawned from the wrong base, leading to incorrect diffs, merge conflicts, and broken CI pipelines.

The `default_branch` field already exists in `ProjectConfig` (loaded from `.apm/config.toml`, with `apm.toml` as a legacy fallback) and is correctly consumed in `start.rs` and `git_util.rs`. However, three locations in `epic.rs` and `new.rs` ignore it and hardcode `"main"` directly.

When `default_branch` is absent from `.apm/config.toml`, the field is populated at deserialisation time via `#[serde(default = "default_branch_main")]`, which returns `"main"`. There is no runtime git-remote detection; the value is always sourced from the config struct.

### Acceptance criteria

- [ ] When `default_branch = "develop"` is set in `.apm/config.toml`, a new ticket worktree branch is fetched and created from `origin/develop`
- [ ] When `default_branch = "develop"` is set in `.apm/config.toml`, a new epic worktree branch is fetched and created from `origin/develop`
- [ ] When `default_branch` is absent from `.apm/config.toml`, ticket and epic branches continue to be created from `origin/main` (existing behaviour unchanged)
- [ ] The fallback branch name in `apm/src/cmd/new.rs` `open_editor()` resolves to the configured default branch, not the literal string `"main"`

### Out of scope

- Changing the default value of `default_branch` (it remains `"main"`)
- Auto-detecting the default branch from the git remote (e.g. `git remote show origin`)
- Migrating or rebasing existing worktrees already created from the wrong base
- UI or server changes

### Approach

Three hardcoded `"main"` references need to be replaced with `config.project.default_branch`. The pattern to follow is already established in `apm-core/src/start.rs`. When `default_branch` is not set in `.apm/config.toml`, the serde default (`"main"`) is used automatically — no additional fallback logic is required in the changed code.

**1. `apm-core/src/epic.rs` — `create()` function (~lines 207, 221)**

- Replace `git_util::fetch_branch(root, "main")` with `git_util::fetch_branch(root, &config.project.default_branch)`
- Replace the `worktree add` call's `"origin/main"` argument with `&format!("origin/{}", config.project.default_branch)`
- Verify `config` is already in scope in this function; if not, thread it through from the call site following the same pattern as the adjacent fetch/merge calls in `start.rs`.

**2. `apm-core/src/epic.rs` — `create_epic_branch()` function (~lines 388–390)**

- Replace the `fetch origin main` arg with `&config.project.default_branch`
- Replace the fallback `git branch <branch> main` arg with `&config.project.default_branch`
- Same config-threading check applies.

**3. `apm/src/cmd/new.rs` — `open_editor()` function (~line 90)**

- Replace `.unwrap_or_else(|| "main".to_string())` with a lookup of `config.project.default_branch`
- Load the config at the top of the function (or accept it as a parameter) using the same loading call used elsewhere in the `cmd/` layer.

No new config fields are introduced. No behaviour changes for projects that already omit `default_branch` from `.apm/config.toml`.

### Open questions


### Amendment requests

- [x] correct configuration name: it's .apm/config.toml not apm.toml
- [x] specify how default branch is inferred if it's not specified in .apm/config.toml

### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-04-24T16:38Z | — | new | philippepascal |
| 2026-04-24T16:41Z | new | groomed | philippepascal |
| 2026-04-24T16:41Z | groomed | in_design | philippepascal |
| 2026-04-24T16:44Z | in_design | specd | claude-0424-1641-e800 |
| 2026-04-24T16:52Z | specd | ammend | philippepascal |
| 2026-04-24T16:52Z | ammend | in_design | philippepascal |