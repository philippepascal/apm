+++
id = "20eeacce"
title = "Epic worktrees nested under worktrees_base twice"
state = "specd"
priority = 0
effort = 2
risk = 2
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/20eeacce-epic-worktrees-nested-under-worktrees-ba"
created_at = "2026-04-24T06:29:13.730768Z"
updated_at = "2026-04-24T07:23:57.274937Z"
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

Two callsites in `apm-core/src/` still compute `worktrees_base` directly from `root` instead of first resolving the main worktree root. Both need the same one-liner fix that `worktree.rs:144` and `start.rs:453/622` already use.

**Fix 1 — `apm-core/src/git_util.rs`, `merge_into_default` (~line 967)**

Replace:
```rust
let worktrees_base = root.join(&config.worktrees.dir);
ensure_worktree(root, &worktrees_base, default_branch)?
```
With:
```rust
let main_root = main_worktree_root(root).unwrap_or_else(|| root.to_path_buf());
let worktrees_base = main_root.join(&config.worktrees.dir);
ensure_worktree(root, &worktrees_base, default_branch)?
```
`main_worktree_root` is already in scope (same file). The `root` passed to `ensure_worktree` stays as-is (it is the git command CWD, not the base directory).

**Fix 2 — `apm-core/src/init.rs`, `ensure_worktrees_dir` (~line 330)**

Replace:
```rust
let wt_dir = root.join(&config.worktrees.dir);
```
With:
```rust
let main_root = crate::git_util::main_worktree_root(root).unwrap_or_else(|| root.to_path_buf());
let wt_dir = main_root.join(&config.worktrees.dir);
```
`main_worktree_root` returns `None` when `root` is not inside a git repo or has no linked worktrees, so the `unwrap_or_else(|| root.to_path_buf())` fallback is safe for the normal `apm init` case where the user is at the main repo root.

**Manual cleanup (outside code change)**

Remove the double-nested directories that were created before the fix in the ticker repo:
- `ticker--worktrees/ticker--worktrees/epic-ad871030-ticker-wasm-crate`
- `ticker--worktrees/ticker--worktrees/epic-b28eec87-wasm-prep-refactors`

These are git-tracked linked worktrees; they must be removed with `git worktree remove --force <path>` (or `git worktree prune` if the directories no longer exist), not just `rm -rf`. This is a one-time manual operation documented in the ticket, not automated by the code change.

**No other files need to change.** `worktree.rs:145`, `start.rs:454`, and `start.rs:623` already use the correct pattern.

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-04-24T06:29Z | — | new | philippepascal |
| 2026-04-24T07:13Z | new | groomed | philippepascal |
| 2026-04-24T07:18Z | groomed | in_design | philippepascal |
| 2026-04-24T07:23Z | in_design | specd | claude-0424-0718-7fc8 |
