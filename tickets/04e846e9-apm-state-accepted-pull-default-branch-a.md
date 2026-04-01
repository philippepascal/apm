+++
id = "04e846e9"
title = "apm state accepted: pull default branch after PR merge"
state = "closed"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
branch = "ticket/04e846e9-apm-state-accepted-pull-default-branch-a"
created_at = "2026-04-01T06:04:14.712639Z"
updated_at = "2026-04-01T06:11:25.528600Z"
+++

## Spec

### Problem

When a ticket is accepted via `apm state <id> accepted`, the PR has already been merged on GitHub (the transition has a `pr_all_closing_merged` precondition). At this point, local main is stale — it does not reflect the merged changes. Nothing currently pulls or fast-forwards main after acceptance.

The fix requires new code in apm-core: a new CompletionStrategy variant (e.g. `PullDefault`) that fetches origin/<default_branch> and fast-forwards local main. This variant must be wired into the TOML config parser so transitions can opt into it with `completion = "pull"`. The `implemented → accepted` transition in .apm/config.toml should then set this.

The existing `completion = "merge"` strategy is wrong for this case — it would try to merge the ticket branch locally, which is redundant and potentially messy after a squash merge.

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
| 2026-04-01T06:04Z | — | new | philippepascal |
| 2026-04-01T06:08Z | new | in_design | philippepascal |
| 2026-04-01T06:11Z | in_design | closed | apm |
