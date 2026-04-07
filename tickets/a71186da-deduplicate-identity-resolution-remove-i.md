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

Two modules in `apm-core` independently resolve the current user's identity with overlapping logic that has drifted apart:

- `identity.rs::resolve_current_user()` (69 lines) — reads `.apm/local.toml` username, checks `git_host` config, calls `gh_username()` for GitHub, falls back to the literal string `"apm"`.
- `config.rs::resolve_identity()` (~37 lines) — reads `.apm/local.toml` username, checks `git_host` config, calls `gh_username()` then `fetch_authenticated_user()` with token resolution, falls back to `"unassigned"`.

The split happened because identity resolution evolved: `config.rs` added fuller GitHub API token support when `git_host` landed, but `identity.rs` was never removed or updated to delegate. Neither function calls the other.

Callers are split across the CLI and server layers. `apm new` and `apm list --mine` (in the `apm` crate) use `identity::resolve_current_user()`. The `apm-server` handlers (sync, POST /api/tickets, GET /api/me, queue access control) use `config::resolve_identity()`. This means a user who has no identity configured gets `author = "apm"` on tickets created via the CLI but `"unassigned"` when created via the server — the same user can appear under two different identities in the same project.

The desired state is a single identity resolution function (`config::resolve_identity()`) used by all callers. The `identity.rs` module and its `pub mod identity` declaration should be removed entirely.

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