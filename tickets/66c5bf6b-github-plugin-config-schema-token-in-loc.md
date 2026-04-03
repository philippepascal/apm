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
| 2026-04-02T20:54Z | — | new | apm |
| 2026-04-02T23:23Z | new | groomed | apm |
| 2026-04-03T00:05Z | groomed | in_design | philippepascal |