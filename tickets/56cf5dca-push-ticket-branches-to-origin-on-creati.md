+++
id = "56cf5dca"
title = "Push ticket branches to origin on creation when aggressive sync is enabled"
state = "in_design"
priority = 0
effort = 2
risk = 0
author = "philippepascal"
branch = "ticket/56cf5dca-push-ticket-branches-to-origin-on-creati"
created_at = "2026-04-08T15:40:56.947438Z"
updated_at = "2026-04-08T21:55:23.968122Z"
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

**Step 1 — Switch to tracking push in `ticket::create()`**

File: `apm-core/src/ticket.rs`, lines ~547–551.

The push block currently reads:
```rust
if aggressive {
    if let Err(e) = crate::git::push_branch(root, &branch) {
        warnings.push(format!("warning: push failed: {e:#}"));
    }
}
```
Change `push_branch` to `push_branch_tracking`. No other changes needed in this function — the `aggressive` parameter is already wired from the caller, the warning pattern is already correct, and the config is already in scope via `config.sync.aggressive` (though the function receives the pre-computed bool, not the config directly — this is fine).

**Step 2 — Verify `epic::create()` (no code change)**

File: `apm-core/src/epic.rs`, line ~162. It already calls `push_branch_tracking()` unconditionally on every `apm epic new`. The acceptance criterion is satisfied as-is. Leave this function unchanged — gating epics on aggressive is out of scope and would be a behaviour change.

**Step 3 — Add tests with a real remote**

File: `apm-core/tests/ticket_create.rs`. Add a `setup_with_remote()` helper that:
1. Creates a bare repo with `git init --bare` in a second temp dir
2. Creates the working repo (reuse existing `setup()` logic)
3. Adds the bare repo as `origin` via `git remote add origin <path>`

Then add three new tests:

- `create_pushes_branch_when_aggressive`: call `ticket::create()` with `aggressive=true`, then verify the branch appears in origin via `git ls-remote origin <branch>` (or `git -C <bare_path> branch --list`). Assert it's non-empty.

- `create_no_push_when_not_aggressive`: call `ticket::create()` with `aggressive=false` (remote configured), verify the branch does NOT appear in origin.

- `create_push_failure_is_warning`: call `ticket::create()` with `aggressive=true` but NO remote configured (use the existing `setup()` which has no remote). Assert the call returns `Ok(...)` and `warnings` is non-empty.

Note: the existing `create_no_push_when_not_aggressive` test (lines ~161–185) already tests no-push without a remote. The new version should use a properly configured remote to make the distinction clear. Rename or keep both.

**Step 4 — No changes needed elsewhere**

- `apm/src/cmd/new.rs`: already correctly passes `config.sync.aggressive && !no_aggressive` — no change.
- `apm-server/src/main.rs`: hardcodes `aggressive=false` intentionally (server has no auth context to push). Leave it.
- `apm-core/src/git.rs`: `push_branch_tracking()` already exists and is correct.

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-04-08T15:40Z | — | new | philippepascal |
| 2026-04-08T21:47Z | new | groomed | apm |
| 2026-04-08T21:51Z | groomed | in_design | philippepascal |