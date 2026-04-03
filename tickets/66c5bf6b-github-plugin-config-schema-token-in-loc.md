+++
id = "66c5bf6b"
title = "GitHub plugin: config schema, token in local.toml, API identity and collaborators sync"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "apm"
agent = "19555"
branch = "ticket/66c5bf6b-github-plugin-config-schema-token-in-loc"
created_at = "2026-04-02T20:54:29.742423Z"
updated_at = "2026-04-03T00:05:31.324777Z"
epic = "8db73240"
target_branch = "epic/8db73240-user-mgmt"
depends_on = ["4cec7a17"]
+++

## Spec

### Problem

When a repo is hosted on GitHub, APM currently requires users to manually
configure their identity in `.apm/local.toml` (`username`) and maintain the
`collaborators` list in `.apm/config.toml` by hand. This is error-prone and
creates drift whenever team membership changes on GitHub.

DESIGN-users.md (point 4) specifies an optional GitHub plugin that solves
both problems: the current user's identity is resolved via `GET /user` using a
stored token, and the collaborators list is synced from
`GET /repos/{owner}/{repo}/collaborators`. When the plugin is not configured,
the system falls back to the manual approach introduced by ticket 4cec7a17.

This ticket implements the plugin foundation: the `[git_host]` config schema,
`github_token` storage in `.apm/local.toml`, and the two API resolution paths
wired into `resolve_identity()` and a new `resolve_collaborators()` helper.

### Acceptance criteria

- [ ] `.apm/config.toml` containing `[git_host]` with `provider = "github"` and `repo = "owner/name"` parses into `Config` without error
- [ ] A config with no `[git_host]` section parses without error (plugin is optional)
- [ ] `LocalConfig` accepts an optional `github_token` field; a `local.toml` without it parses without error
- [ ] `resolve_identity()` returns the GitHub login when `[git_host]` is configured and a token is available (via `local.toml` or `GITHUB_TOKEN` env var)
- [ ] `resolve_identity()` falls back to the `local.toml` `username` field when the GitHub plugin is not configured
- [ ] `resolve_identity()` returns `"unassigned"` when neither GitHub plugin nor `local.toml` username is set
- [ ] `resolve_identity()` falls back gracefully (continues to `local.toml` / `"unassigned"`) when the GitHub API returns an error or is unreachable
- [ ] `resolve_collaborators()` returns the list of GitHub logins from the collaborators API when `[git_host]` is configured and a token is available
- [ ] `resolve_collaborators()` falls back to the static `collaborators` list from `config.toml` when the GitHub plugin is not configured
- [ ] `resolve_collaborators()` falls back gracefully to the static list when the GitHub API returns an error or is unreachable

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
| 2026-04-02T20:54Z | — | new | apm |
| 2026-04-02T23:23Z | new | groomed | apm |
| 2026-04-03T00:05Z | groomed | in_design | philippepascal |