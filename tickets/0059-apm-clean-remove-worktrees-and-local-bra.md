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

- [ ] `apm clean` iterates all tickets in terminal states (per `workflow.terminal_states` in `apm.toml`) and, for each: removes the permanent worktree if one exists, and deletes the local branch ref.
- [ ] `--dry-run` prints what would be removed without modifying anything.
- [ ] A ticket whose branch is not merged into the default branch is skipped with a warning (safety guard against premature cleanup).
- [ ] A worktree with uncommitted changes (dirty index or working tree) is skipped with a warning and its local branch is left intact.
- [ ] Each removed worktree or branch ref produces one line of output: `removed worktree <path>` or `removed branch <name>`.
- [ ] If there is nothing to clean, prints `Nothing to clean.` and exits 0.

### Out of scope



### Approach



## History

| When | From | To | By |
|------|------|----|----|
| 2026-03-29T19:12Z | — | new | claude-0329-1200-a1b2 |
| 2026-03-29T22:56Z | new | in_design | claude-spec-59 |