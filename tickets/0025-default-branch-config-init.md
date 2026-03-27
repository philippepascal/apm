+++
id = 25
title = "default-branch-config-init"
state = "new"
priority = 5
effort = 3
risk = 2
author = "apm"
branch = "ticket/0025-default-branch-config-init"
created_at = "2026-03-26T23:43:13.236447Z"
updated_at = "2026-03-27T05:33:00.099668Z"
+++

## Spec

### Problem

APM hardcodes `"main"` in three places: `apm sync` commits auto-transitions
to `"main"`, `apm verify --fix` commits fixes to `"main"`, and
`git::merged_into_main` checks `git branch --merged main`. Repos that use
`master`, `trunk`, or any other default branch name will silently fail or
produce incorrect results.

`apm init` can detect the default branch at setup time and record it in
`apm.toml`, making these commands branch-agnostic.

### Acceptance criteria

- [ ] `apm.toml` has a `[project] default_branch` field (string, defaults to `"main"`)
- [ ] `apm init` detects the repo's current branch and writes it as `default_branch` in the generated config
- [ ] `apm sync` commits auto-transitions to the configured `default_branch`, not hardcoded `"main"`
- [ ] `apm verify --fix` commits to the configured `default_branch`
- [ ] `git::merged_into_main` uses the configured `default_branch` when checking merged branches
- [ ] Existing repos without `default_branch` in `apm.toml` default to `"main"` (backward-compatible)

### Out of scope

- Changing the default branch
- Multi-branch or trunk-based workflows beyond the single default branch
- Renaming the function `merged_into_main` (internal refactor not required)

### Approach

1. Add `default_branch: String` to `ProjectConfig` in `apm-core/src/config.rs` with `#[serde(default = "default_main")]`
2. In `apm init`, detect via `git symbolic-ref --short HEAD` (falls back to `"main"` on error) and write the result into the generated `apm.toml`
3. Pass `config.project.default_branch` into `git::merged_into_main`, `cmd::sync::run`, and `cmd::verify::run` in place of the hardcoded literal

## History

| When | From | To | By |
|------|------|----|----|
| 2026-03-26T23:43Z | — | new | apm |