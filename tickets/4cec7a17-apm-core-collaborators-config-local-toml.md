+++
id = "4cec7a17"
title = "apm-core: collaborators config, local.toml, and identity resolution"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "apm"
agent = "50192"
branch = "ticket/4cec7a17-apm-core-collaborators-config-local-toml"
created_at = "2026-04-02T20:53:47.546444Z"
updated_at = "2026-04-02T23:21:53.468894Z"
epic = "8db73240"
target_branch = "epic/8db73240-user-mgmt"
+++

## Spec

### Problem

There is no concept of stable human identity in apm-core. The `author` field is currently set to the `APM_AGENT_NAME` environment variable (an ephemeral agent session string like `claude-0402-1430-a3f9`) or the literal string `"apm"` for automated actions. There is no collaborators list in config, no per-machine identity file, and no resolution function. As a result, there is no reliable way to track which human created a ticket — only which short-lived agent process ran at the time.

The desired state (design doc points 1–3) is:
- `.apm/config.toml` carries a `collaborators` list under `[project]`
- `.apm/local.toml` (gitignored, per-machine) stores the local user's `username`
- A `resolve_identity(repo_root)` function returns the username from `local.toml` or `"unassigned"` as a fallback
- `apm new` uses the resolved identity as `author` instead of the agent env var
- `apm init` prompts for a username, writes `local.toml`, updates `collaborators`, and adds `local.toml` to `.gitignore`
- The `agent` field is dropped from frontmatter writes (silently tolerated on read for backward compatibility)

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
| 2026-04-02T20:53Z | — | new | apm |
| 2026-04-02T23:21Z | new | groomed | apm |
| 2026-04-02T23:21Z | groomed | in_design | philippepascal |