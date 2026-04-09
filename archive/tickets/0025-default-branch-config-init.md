+++
id = 25
title = "default-branch-config-init"
state = "closed"
priority = 5
effort = 3
risk = 2
author = "apm"
agent = "claude-0326-2333-75f3"
branch = "ticket/0025-default-branch-config-init"
created_at = "2026-03-26T23:43:13.236447Z"
updated_at = "2026-03-30T02:02:46.501095Z"
+++

## Spec

### Problem

apm should be able to configure what default branch to use. main is default, but what if users use master, or a development branch

### Acceptance criteria

- [ ] `ProjectConfig` has a `default_branch` field (string, defaults to `"main"`) parseable from `apm.toml`
- [ ] `apm init` detects the repo's current branch via `git symbolic-ref --short HEAD` and writes it as `default_branch` in the generated `apm.toml`
- [ ] `apm sync`'s auto-transition commits target `config.project.default_branch` instead of hardcoded `"main"`
- [ ] `apm verify --fix` commits to `config.project.default_branch` instead of hardcoded `"main"`
- [ ] `git::merged_into_main` accepts the default branch as a parameter and uses it instead of hardcoded `"main"`
- [ ] Repos with no `default_branch` in `apm.toml` default to `"main"` (backward-compatible)
- [ ] Integration test: generated config contains `default_branch` and `Config::load` parses it

### Out of scope

- Changing the default branch at runtime
- Supporting multiple default branches
- Renaming the `merged_into_main` function

### Amendment requests

<!-- Add amendment requests below -->

### Approach

1. Add `default_branch: String` to `ProjectConfig` in `apm-core/src/config.rs` with `#[serde(default = "default_branch_main")]` returning `"main".to_string()`
2. In `cmd/init.rs` `default_config()`, detect branch via `git symbolic-ref --short HEAD` (or `git rev-parse --abbrev-ref HEAD`), fall back to `"main"` on error, and include it in the generated TOML
3. Thread `config.project.default_branch` into `git::merged_into_main(root, branch)`, `cmd::sync::run`, and `cmd::verify::run` replacing the three hardcoded `"main"` literals
## History

| When | From | To | By |
|------|------|----|----|
| 2026-03-26T23:43Z | — | new | apm |
| 2026-03-27T05:36Z | new | specd | claude-0326-2222-8071 |
| 2026-03-27T06:05Z | specd | ammend | apm |
| 2026-03-27T06:08Z | ammend | in_progress | claude-0326-2222-8071 |
| 2026-03-27T06:36Z | claude-0326-2222-8071 | claude-0326-2333-75f3 | handoff |
| 2026-03-28T00:43Z | in_progress | implemented | claude-0326-2333-75f3 |
| 2026-03-28T00:49Z | implemented | accepted | apm sync |
| 2026-03-30T02:02Z | accepted | closed | apm-sync |