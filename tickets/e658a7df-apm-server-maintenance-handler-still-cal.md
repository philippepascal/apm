+++
id = "e658a7df"
title = "apm-server maintenance handler still calls push_default_branch"
state = "groomed"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/e658a7df-apm-server-maintenance-handler-still-cal"
created_at = "2026-04-17T19:12:05.718252Z"
updated_at = "2026-04-17T19:59:45.150278Z"
+++

## Spec

### Problem

apm-server/src/handlers/maintenance.rs:27 calls apm_core::git::push_default_branch. The sync.rs push was removed by ticket a087593c, but this server-side handler was out of scope and still calls it. The function was kept in git_util.rs for this reason. A follow-up should decide whether the maintenance handler should also stop auto-pushing main, or be replaced with sync_default_branch semantics.

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
