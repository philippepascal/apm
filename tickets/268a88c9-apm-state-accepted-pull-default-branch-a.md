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

When a ticket transitions from `implemented` to `accepted`, GitHub has already merged the closing PR (enforced by the `pr_all_closing_merged` precondition). At that point the local `main` branch is stale — it does not yet reflect the squash-merge that GitHub performed. Nothing in APM currently fetches or fast-forwards local `main` after acceptance.

The existing `completion = "merge"` strategy is not appropriate here: it merges the ticket branch locally with `--no-ff`, creating a redundant local merge commit on top of a squash-merge that GitHub already did. What is needed instead is a lightweight pull: fetch `origin/<default_branch>` and fast-forward the local default branch to match it.

This matters because engineers who immediately branch off `main` after accepting a ticket will be working from a stale base, risking conflicts and confusion about what has actually shipped.

### Acceptance criteria

- [ ] `apm state <id> accepted` fetches `origin/main` (or the configured default branch) from the remote
- [ ] After `apm state <id> accepted`, the local default branch is fast-forwarded to match `origin/<default_branch>`
- [ ] If the local default branch cannot be fast-forwarded (e.g. it has diverged), `apm state` prints a clear warning but does not fail — the state transition still completes
- [ ] `completion = "pull"` is accepted in `.apm/config.toml` transition definitions without a parse error
- [ ] `apm verify` lists the `implemented → accepted` transition as `completion: implemented → accepted = pull`
- [ ] The `implemented → accepted` transition in the project `.apm/config.toml` uses `completion = "pull"`

### Out of scope

- Removing or modifying the existing `completion = "merge"` strategy
- Deleting the ticket branch after acceptance (that belongs to the `closed` transition)
- Pushing the ticket branch to origin as part of acceptance (the PR is already merged)
- Handling cases where `git fetch` fails due to network issues (a warning and non-fatal continuation is sufficient)
- Any changes to the `implemented → closed` or `accepted → closed` transitions

### Approach

How the implementation will work.

### Open questions



### Amendment requests



## History

| When | From | To | By |
|------|------|----|----|
| 2026-04-01T06:10Z | — | new | philippepascal |
| 2026-04-01T06:12Z | new | in_design | philippepascal |