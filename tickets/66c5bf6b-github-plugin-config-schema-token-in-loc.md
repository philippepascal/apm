+++
id = "66c5bf6b"
title = "GitHub plugin: config schema, token in local.toml, API identity and collaborators sync"
state = "new"
priority = 0
effort = 0
risk = 0
author = "apm"
branch = "ticket/66c5bf6b-github-plugin-config-schema-token-in-loc"
created_at = "2026-04-02T20:54:29.742423Z"
updated_at = "2026-04-02T20:54:29.742423Z"
epic = "8db73240"
target_branch = "epic/8db73240-user-mgmt"
depends_on = ["4cec7a17"]
+++

## Spec

### Problem

When a repo is hosted on GitHub, collaborator identity and the collaborators list could be resolved directly from the GitHub API, removing the need for manual configuration of `.apm/local.toml` and `collaborators` in config.toml. Without this plugin, teams on GitHub must manage identity configuration by hand. See `initial_specs/DESIGN-users.md` point 4.

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