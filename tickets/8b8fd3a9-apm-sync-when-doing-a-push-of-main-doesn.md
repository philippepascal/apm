+++
id = "8b8fd3a9"
title = "apm sync, when doing a push of main, doesn't display push/hook outputs"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/8b8fd3a9-apm-sync-when-doing-a-push-of-main-doesn"
created_at = "2026-06-13T18:33:03.661566Z"
updated_at = "2026-06-14T05:57:41.484756Z"
+++

## Spec

### Problem

`apm sync` calls `push_branch` (defined in `apm-core/src/git_util.rs`) when pushing the default branch or ahead ticket/epic branches. `push_branch` delegates to the internal `run()` helper, which uses `Command::output()` — this captures both stdout and stderr of the git process. On success, both streams are discarded; on failure, only the captured stderr is surfaced as an error string. Remote push hooks (e.g. CI gate hooks, lint hooks, custom server-side checks) emit their messages through the git process's stderr, so those messages are silently swallowed every time the push succeeds.

Users who have hooks installed on the remote see nothing — no confirmation the hook ran, no warnings or status lines the hook produced. The fix is to stream git's output directly to the terminal rather than capturing it.

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
| 2026-06-13T18:33Z | — | new | philippepascal |
| 2026-06-14T05:57Z | new | groomed | philippepascal |
| 2026-06-14T05:57Z | groomed | in_design | philippepascal |