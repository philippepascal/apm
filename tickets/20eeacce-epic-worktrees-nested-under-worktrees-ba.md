+++
id = "20eeacce"
title = "Epic worktrees nested under worktrees_base twice"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/20eeacce-epic-worktrees-nested-under-worktrees-ba"
created_at = "2026-04-24T06:29:13.730768Z"
updated_at = "2026-04-24T07:18:42.739393Z"
+++

## Spec

### Problem

Epic worktrees are created at `<worktrees_base>/<worktrees_base>/epic-<id>-<slug>/` instead of `<worktrees_base>/epic-<id>-<slug>/`. Two examples observed in the ticker repo: `ticker--worktrees/ticker--worktrees/epic-ad871030-ticker-wasm-crate` and `ticker--worktrees/ticker--worktrees/epic-b28eec87-wasm-prep-refactors`. Ticket (non-epic) worktrees land correctly.

The double-nesting happens when `config.worktrees.dir` is a relative path such as `../ticker--worktrees` and the caller is already inside a linked worktree. In that case `root.join("../ticker--worktrees")` resolves to `<worktrees_base>/<project>--worktrees` — one level too deep.

Two callsites still use `root.join(&config.worktrees.dir)` directly without first resolving the main worktree root: `git_util.rs:967` (inside `merge_into_default`) and `init.rs:330` (inside `ensure_worktrees_dir`). The correct pattern — already used by `worktree.rs:144`, `start.rs:453`, and `start.rs:622` — calls `main_worktree_root(root)` first and joins against that result.

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
| 2026-04-24T07:18Z | groomed | in_design | philippepascal |