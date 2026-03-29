+++
id = 59
title = "apm clean: remove worktrees and local branches for closed tickets"
state = "in_design"
priority = 3
effort = 0
risk = 0
author = "claude-0329-1200-a1b2"
branch = "ticket/0059-apm-clean-remove-worktrees-and-local-bra"
created_at = "2026-03-29T19:12:18.328861Z"
updated_at = "2026-03-29T22:56:54.841686Z"
+++

## Spec

### Problem

After tickets are closed and their PRs merged, the permanent git worktrees and local branch refs created by `apm start` / `apm worktrees --add` are never cleaned up. Over time this clutters `git worktree list` and `git branch --list ticket/*` with stale entries. There is no command to remove them in bulk.

### Acceptance criteria


### Out of scope



### Approach



## History

| When | From | To | By |
|------|------|----|----|
| 2026-03-29T19:12Z | — | new | claude-0329-1200-a1b2 |
| 2026-03-29T22:56Z | new | in_design | claude-spec-59 |