+++
id = "e658a7df"
title = "apm-server maintenance handler still calls push_default_branch"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/e658a7df-apm-server-maintenance-handler-still-cal"
created_at = "2026-04-17T19:12:05.718252Z"
updated_at = "2026-04-17T20:00:07.481920Z"
+++

## Spec

### Problem

Ticket a087593c established the principle that apm must never automatically push the default branch. It removed `push_default_branch` from the CLI sync command and replaced it with `sync_default_branch`, a safe state-machine alternative (Equal / Behind / Ahead / Diverged / NoRemote) that fast-forwards when behind but never pushes.

The server-side maintenance handler (`apm-server/src/handlers/maintenance.rs`, line 27) was explicitly left out of that ticket's scope. It still calls `apm_core::git::push_default_branch`, making it the only remaining automatic pusher in the codebase. `push_default_branch` was retained in `git_util.rs` solely because this caller existed.

The desired behaviour is for the maintenance handler to follow the same safe-sync semantics as the CLI: fast-forward when behind, log a warning when ahead or diverged, and never push. Once the handler is updated, `push_default_branch` has no remaining callers and should be removed.

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
| 2026-04-17T19:12Z | — | new | philippepascal |
| 2026-04-17T19:59Z | new | groomed | apm |
| 2026-04-17T20:00Z | groomed | in_design | philippepascal |