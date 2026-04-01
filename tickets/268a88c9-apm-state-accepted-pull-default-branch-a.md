+++
id = "268a88c9"
title = "apm state accepted: pull default branch after PR merge"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
agent = "71767"
branch = "ticket/268a88c9-apm-state-accepted-pull-default-branch-a"
created_at = "2026-04-01T06:10:14.360337Z"
updated_at = "2026-04-01T06:12:47.286959Z"
+++

## Spec

### Problem

When a ticket is accepted via `apm state <id> accepted`, the PR has already been merged on GitHub (precondition: pr_all_closing_merged). Local main is now stale. Nothing currently pulls or fast-forwards main after acceptance.

Fix requires new code in apm-core: a new CompletionStrategy variant (e.g. PullDefault) that fetches origin/<default_branch> and fast-forwards local main. Wire into the TOML config parser so transitions can use `completion = "pull"`. Then set this on the implemented → accepted transition in .apm/config.toml.

The existing `completion = "merge"` is wrong here — it merges the ticket branch locally, which is redundant/messy after a squash merge on GitHub.

What is broken or missing, and why it matters.

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
| 2026-04-01T06:10Z | — | new | philippepascal |
| 2026-04-01T06:12Z | new | in_design | philippepascal |
