+++
id = "5a4ad4bd"
title = "apm work improvement"
state = "in_design"
priority = 0
effort = 2
risk = 2
author = "apm"
agent = "93102"
branch = "ticket/5a4ad4bd-apm-work-improvement"
created_at = "2026-03-30T19:21:34.679718Z"
updated_at = "2026-03-30T19:27:45.034164Z"
+++

## Spec

### Problem

The `apm work` dispatch loop sets a permanent `no_more` flag the first time `spawn_next_worker` returns `None` (no actionable ticket found). Once that flag is set, the loop stops trying to spawn new workers and merely waits for existing workers to drain. This means that if a running worker finishes and its ticket transitions unblock another ticket—making it newly actionable—the loop will never pick it up. The result: `apm work` under-utilises available worker slots after the initial burst, and newly-unblocked tickets are silently ignored until a completely new `apm work` invocation is run by the user.

### Acceptance criteria


### Out of scope

Explicit list of what this ticket does not cover.

### Approach

How the implementation will work.

### Open questions



### Amendment requests



## History

| When | From | To | By |
|------|------|----|----|
| 2026-03-30T19:21Z | — | new | apm |
| 2026-03-30T19:23Z | new | in_design | philippepascal |