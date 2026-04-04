+++
id = "63140ca3"
title = "apm state: push to origin after merge_into_default completes"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "apm"
branch = "ticket/63140ca3-apm-state-push-to-origin-after-merge-int"
created_at = "2026-04-04T02:20:43.522276Z"
updated_at = "2026-04-04T06:38:48.457743Z"
+++

## Spec

### Problem

When `apm state <id> implemented` runs and the workflow's completion strategy is `merge` (or `pr_or_epic_merge` with a target branch set), `state.rs` calls `merge_into_default`, which merges the ticket branch into the default branch locally but never pushes the result to origin.

After the merge the local default branch is ahead of `origin/<default_branch>`. No other contributor or CI system sees the merged commit until someone manually runs `git push origin <default_branch>`. This defeats the purpose of the automated merge path and leaves the repo in an inconsistent state.

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
| 2026-04-04T02:20Z | — | new | apm |
| 2026-04-04T06:02Z | new | groomed | apm |
| 2026-04-04T06:38Z | groomed | in_design | philippepascal |