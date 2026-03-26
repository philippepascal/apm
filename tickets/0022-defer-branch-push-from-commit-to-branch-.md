+++
id = 22
title = "Defer branch push from commit_to_branch to apm sync"
state = "implemented"
priority = 8
effort = 2
risk = 2
branch = "ticket/0022-defer-branch-push-from-commit-to-branch-"
created = "2026-03-26"
updated = "2026-03-26"
+++

## Spec

### Problem

`commit_to_branch` pushes to `origin/<branch>` after every commit (line 155 of
`apm-core/src/git.rs`). This makes every `apm state`, `apm set`, and `apm new`
call incur a live network round-trip to the remote. With moderate GitHub latency,
each command takes 1–3 seconds instead of being near-instant.

The spec does not require an immediate push for ticket branches — `apm sync` is
the designed mechanism for propagating state to the remote. The push in
`commit_to_branch` is a shortcut that trades correctness for convenience, but
the cost (noticeable slowness on every command) outweighs the benefit.

### Acceptance criteria

- [ ] `commit_to_branch` no longer pushes to origin after committing to a ticket
  branch (remove the `git push origin <branch>` call in `try_worktree_commit`)
- [ ] `apm sync` is responsible for pushing all local ticket branches that have
  unpushed commits — it should push each `ticket/*` branch that has commits
  not present on `origin/<branch>`
- [ ] `apm state`, `apm set`, `apm new` complete without any network call (fast)
- [ ] `initial_specs/SPEC.md` (or whichever section covers sync/push behaviour)
  is updated to document that ticket branch pushes are deferred to `apm sync`
- [ ] All existing tests continue to pass

### Out of scope

- Changing push behaviour for `apm/meta` branch (that push is intentional and
  required for the optimistic-lock protocol)
- Offline-mode flag for `apm sync` (tracked in #17)

### Approach

1. In `try_worktree_commit` in `apm-core/src/git.rs`: remove the
   `let _ = run(root, &["push", "origin", branch]);` line
2. In `apm/src/cmd/sync.rs`: after existing merge detection, iterate local
   `ticket/*` branches and push any that are ahead of their remote tracking
   branch (use `git rev-list --count origin/<b>..<b>` > 0 as the check; skip
   branches with no remote counterpart if origin is absent)
3. Update `initial_specs/SPEC.md` to note that ticket branch pushes are batched
   by `apm sync`, not immediate

## History

| Date | Actor | Transition | Note |
|------|-------|------------|------|
| 2026-03-26 | agent | new → ready | |
