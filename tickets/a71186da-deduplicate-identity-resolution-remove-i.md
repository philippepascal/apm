+++
id = "a71186da"
title = "Deduplicate identity resolution: remove identity.rs, use config.rs"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
branch = "ticket/a71186da-deduplicate-identity-resolution-remove-i"
created_at = "2026-04-07T22:30:44.747975Z"
updated_at = "2026-04-07T22:49:15.247022Z"
epic = "ac0fb648"
target_branch = "epic/ac0fb648-code-separation-and-reuse-cleanup"
+++

## Spec

### Problem

Two modules resolve the current user's identity with overlapping logic:

- `identity.rs::resolve_current_user()` (68 lines) — reads `APM_AGENT_NAME` env var, falls back to git config `user.name`
- `config.rs::resolve_identity()` (~50 lines within Config::load) — reads `.apm/local.toml` username, falls back to GitHub API, then git config

Both exist because identity resolution evolved: `identity.rs` was the original, `config.rs` added the richer version when git_host support landed. Neither calls the other. Callers must choose which to use, and the two can return different values for the same user depending on which fallback path triggers.

This creates a correctness risk: a user could be identified as "philippepascal" by one path and "Philippe Pascal" by the other, leading to inconsistent `agent` field values on tickets.

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
| 2026-04-07T22:30Z | — | new | philippepascal |
| 2026-04-07T22:43Z | new | groomed | apm |
| 2026-04-07T22:49Z | groomed | in_design | philippepascal |
