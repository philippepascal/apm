+++
id = "20eeacce"
title = "Epic worktrees nested under worktrees_base twice"
state = "groomed"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/20eeacce-epic-worktrees-nested-under-worktrees-ba"
created_at = "2026-04-24T06:29:13.730768Z"
updated_at = "2026-04-24T07:13:23.747829Z"
+++

## Spec

### Problem

Epic worktrees land at <worktrees_base>/<worktrees_base>/epic-<id>-<slug>/ instead of <worktrees_base>/epic-<id>-<slug>/. Example in ticker: /Users/philippepascal/repos/ticker--worktrees/ticker--worktrees/epic-ad871030-ticker-wasm-crate and /Users/philippepascal/repos/ticker--worktrees/ticker--worktrees/epic-b28eec87-wasm-prep-refactors. Ticket (non-epic) worktrees land correctly at /Users/philippepascal/repos/ticker--worktrees/ticket-<id>-<slug>/. Root cause: apm-core/src/git_util.rs:967-968 uses worktrees_base = root.join(config.worktrees.dir), but root can be a worktree path so .join("../ticker--worktrees") resolves to <worktree>/../ticker--worktrees = <worktrees_base>/ticker--worktrees. Compare apm-core/src/start.rs:453-454 which computes main_root = main_worktree_root(root).unwrap_or_else(|| root.to_path_buf()) first — the correct pattern. Expected: audit every callsite joining config.worktrees.dir (worktree.rs:145, git_util.rs:967, init.rs:330, start.rs:454, start.rs:623) and normalize to always go via main_worktree_root. Manual cleanup of existing nested directories needed after fix.

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
| 2026-04-24T06:29Z | — | new | philippepascal |
| 2026-04-24T07:13Z | new | groomed | philippepascal |
