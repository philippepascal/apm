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

When `aggressive = true` in sync config, `apm new` (and other branch-creating commands like `apm epic new`) should push the branch to origin immediately after creation. This matches the aggressive sync philosophy: keep local and remote in sync at all times.

### Acceptance criteria

- [ ] `apm new` pushes the ticket branch to origin when `sync.aggressive = true`
- [ ] `apm epic new` pushes the epic branch to origin when aggressive (already does this — verify)
- [ ] Push failure is non-fatal: warns but does not fail the command (supports offline work)
- [ ] When `sync.aggressive = false`, no push happens (current behavior preserved)
- [ ] State transition fetch errors no longer appear for freshly created tickets in aggressive mode
- [ ] Tests cover: aggressive push on create, non-aggressive skips push, push failure is warning

### Out of scope

Auto-pushing on every commit within a worktree. Pushing on state transitions (already handled by completion strategies).

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
