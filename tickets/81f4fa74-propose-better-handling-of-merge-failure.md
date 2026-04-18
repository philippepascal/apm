+++
id = "81f4fa74"
title = "propose better handling of merge failures by worker"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/81f4fa74-propose-better-handling-of-merge-failure"
created_at = "2026-04-18T07:37:39.058963Z"
updated_at = "2026-04-18T18:42:54.148458Z"
+++

## Spec

### Problem

When the worker transitions `in_progress → implemented`, the completion strategy runs `git merge --no-ff` into the default branch. If the merge fails (conflict, push error, etc.), the entire state transition fails with an error and the ticket remains in `in_progress`. The supervisor has no way to distinguish "worker is still implementing" from "worker finished but the merge blew up." The failure reason is only visible in the stderr of whoever ran `apm state`, which in an agent-driven workflow is ephemeral.

The desired behaviour is that merge failure is a first-class outcome: the ticket moves to a dedicated state, the failure reason is persisted in the ticket file, and the supervisor can act on it directly from `apm review` or `apm list` without needing to re-run commands or inspect git logs.

### Acceptance criteria

- [ ] When `git merge` fails during the `in_progress → implemented` transition, the ticket transitions to `merge_failed` instead of staying in `in_progress`
- [ ] The merge error message (stderr from git) is written to a `### Merge notes` section in the ticket file before the state is committed
- [ ] `merge_failed` is supervisor-actionable: it appears in `apm review` output and `apm list` under the supervisor role
- [ ] `apm show <id>` renders the `### Merge notes` section when the ticket is in `merge_failed` state
- [ ] From `merge_failed`, the supervisor can transition to `implemented` without triggering another merge attempt
- [ ] From `merge_failed`, the supervisor can transition back to `in_progress` (to let the worker retry)
- [ ] When the transition to `merge_failed` itself fails (e.g. cannot commit the ticket), the original merge error is still reported and the ticket is left in `in_progress` (no silent data loss)

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
| 2026-04-18T07:37Z | — | new | philippepascal |
| 2026-04-18T18:42Z | new | groomed | philippepascal |
| 2026-04-18T18:42Z | groomed | in_design | philippepascal |