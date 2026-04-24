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

- [ ] `apm state <epic-id> in_design` run from inside any linked worktree places the new epic worktree at `<worktrees_base>/epic-<id>-<slug>/`
- [ ] `apm state <epic-id> in_design` run from the main repo root places the new epic worktree at `<worktrees_base>/epic-<id>-<slug>/`
- [ ] `merge_into_default` invoked from inside a linked worktree creates the default-branch worktree at `<worktrees_base>/<default-branch>/`, not at `<worktrees_base>/<worktrees_base>/...`
- [ ] `apm init` / `ensure_worktrees_dir` run from inside a linked worktree creates `<worktrees_base>/` at the correct location (sibling of the main repo, not nested under the existing worktrees dir)
- [ ] Ticket (non-epic) worktrees continue to land at `<worktrees_base>/ticket-<id>-<slug>/` with no regression
- [ ] `cargo test` passes with no new failures after both callsite fixes

### Out of scope

- Automated cleanup of the already-created double-nested directories (manual `git worktree remove` steps are noted in the approach but not scripted)
- Changes to the `config.worktrees.dir` format or making it an absolute path
- Adding tests specifically for the worktree-path computation (verifying via `cargo test` is sufficient here)

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