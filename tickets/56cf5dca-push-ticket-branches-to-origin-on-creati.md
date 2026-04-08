+++
id = "56cf5dca"
title = "Push ticket branches to origin on creation when aggressive sync is enabled"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
branch = "ticket/56cf5dca-push-ticket-branches-to-origin-on-creati"
created_at = "2026-04-08T15:40:56.947438Z"
updated_at = "2026-04-08T21:51:19.448573Z"
+++

## Spec

### Problem

When `apm new` creates a ticket, the branch is only created locally. In a multi-user setup, other collaborators and the server cannot see the ticket until the branch is pushed to origin. This breaks the collaborative workflow: a supervisor creates tickets and grooms them, but no one else sees them.

Additionally, state transitions fetch dependency branches from origin (`git fetch origin <branch>`), producing noisy `fatal: couldn't find remote ref` errors in server logs when those branches are local-only. Pushing on creation would eliminate this noise.

When `aggressive = true` in sync config, `apm new` should push the branch to origin immediately after creation. This matches the aggressive sync philosophy: keep local and remote in sync at all times.

**Current state:** Most of the implementation already exists. `ticket::create()` accepts `aggressive: bool`, the CLI handler (`apm/src/cmd/new.rs`) already derives `aggressive = config.sync.aggressive && !no_aggressive` and passes it through, and the push block already exists in `ticket.rs` (lines ~547–551). However, it calls `git::push_branch()` (no `--set-upstream`) rather than `git::push_branch_tracking()`. Additionally, no tests exercise the push path against an actual remote. `epic::create()` already calls `push_branch_tracking()` unconditionally on every creation.

The remaining work is: switch `ticket::create()` to use `push_branch_tracking`, and add tests that exercise the push path using a local bare-repo remote.

### Acceptance criteria

- [ ] `apm new` pushes the ticket branch to origin (with tracking) when `sync.aggressive = true`
- [ ] `apm epic new` pushes the epic branch to origin when `sync.aggressive = true` (already does so unconditionally — confirm by inspection, no code change needed)
- [ ] Push failure in `ticket::create()` is non-fatal: a warning is emitted but the command succeeds
- [ ] When `sync.aggressive = false`, `apm new` does not push the ticket branch
- [ ] After `apm new` runs with aggressive mode and a remote is configured, `git ls-remote origin <branch>` shows the new branch
- [ ] Tests cover the aggressive-push path using a local bare-repo remote
- [ ] Tests cover non-aggressive mode: no push attempted, command succeeds
- [ ] Tests cover push failure (no remote configured + aggressive=true): warning emitted, command succeeds

### Out of scope

- Auto-pushing on every commit within a worktree\n- Pushing on state transitions (already handled by completion strategies)\n- Gating epic::create() push on the aggressive flag (it always pushes; changing that behaviour is a separate decision)

### Approach

In `apm-core/src/ticket.rs` `create()`, after the branch is created and the initial commit is made, check `config.sync.aggressive`. If true, call `git::push_branch_tracking()` and handle errors as warnings. Same pattern in `apm-core/src/epic.rs` `create()` (verify it already pushes — if so, just confirm the aggressive gate). The config needs to be passed to or loaded within `ticket::create()`.

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-04-08T15:40Z | — | new | philippepascal |
| 2026-04-08T21:47Z | new | groomed | apm |
| 2026-04-08T21:51Z | groomed | in_design | philippepascal |