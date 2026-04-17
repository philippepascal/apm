+++
id = "a087593c"
title = "Make apm sync safe on main branch"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/a087593c-make-apm-sync-safe-on-main-branch"
created_at = "2026-04-17T18:32:29.530485Z"
updated_at = "2026-04-17T18:34:07.974525Z"
epic = "47375a6a"
target_branch = "epic/47375a6a-safer-apm-sync"
+++

## Spec

### Problem

`apm sync` currently mishandles the default branch (`main`) in two ways:

1. **Blind push on inequality.** `push_default_branch` (`apm-core/src/git_util.rs:33`) pushes whenever local and `origin/<default>` SHAs differ — with no regard for direction. When origin is ahead (e.g. after the user merged an epic PR on GitHub), the push is rejected non-fast-forward and surfaces as a `warning: push main failed: …` line.
2. **No fast-forward.** After the fetch step, `origin/main` ref is updated locally, but the local `main` branch and main worktree are never fast-forwarded. Users have to run `git pull` manually before `apm sync` does anything useful post-merge.

Additionally, per the review decision captured in the design doc, **apm sync must not push anything automatically** — explicit pushes happen via `apm state <id> implemented` and equivalents. Sync's job on `main` is to (a) fetch, (b) fast-forward local when possible, (c) inform the user when local is ahead/diverged, and (d) never block on a push attempt that cannot succeed.

See `/Users/philippepascal/Documents/apm/apm-sync-scenarios.md` for the full scenario matrix, algorithm, and guidance strings. Implementers must add comments in the sync module explaining the local/remote classification states and why each maps to its action — the logic is not intuitive at a glance.

### Acceptance criteria

- [ ] `apm sync` never calls `git push` on the default branch (default push path removed entirely)
- [ ] When local `main` equals `origin/main`: sync prints nothing about main and makes no ref changes
- [ ] When local `main` is strictly behind `origin/main` and the working tree does not conflict: local `main` is fast-forwarded via `git merge --ff-only origin/<default>` on the main worktree
- [ ] When local `main` is behind but the FF would overwrite uncommitted local changes: sync prints the "main behind, FF blocked" guidance block and leaves the working tree untouched
- [ ] When local `main` is strictly ahead of `origin/main`: sync prints a single info line ("main is ahead of origin/<default> by N commits — run `git push` when ready") and makes no network call
- [ ] When local `main` and `origin/main` have diverged: sync prints the divergence guidance (rebase/merge choice) and does not modify local main or the working tree
- [ ] When `origin/main` cannot be resolved (no remote, unreachable, fetch failed): main is skipped silently; any fetch failure surfaces as a single warning line from the existing fetch path
- [ ] The sync module has block comments documenting the Equal/Ahead/Behind/Diverged/NoRemote classification and why each maps to its action
- [ ] Integration tests in `apm/tests/integration.rs` using temp git repos cover: equal, behind-FF-clean, behind-FF-blocked-by-dirty, ahead, diverged, and no-remote cases
- [ ] `cargo test --workspace` passes

### Out of scope

- Non-checked-out `ticket/*` and `epic/*` ref handling (ticket `1339c81d`)
- Mid-merge / mid-rebase / mid-cherry-pick detection and shared guidance strings module (ticket `5cf54181`)
- Any form of automatic `git push` from `apm sync` for any branch
- Changes to `apm state <id> implemented` or other state-transition push behavior
- `--offline` flag semantics (unchanged)
- Pre-emptively computing dirty-tree overlap — rely on `git merge --ff-only`'s native error and fall through to guidance
- Renaming `default_branch` config or any other config shape changes

### Approach

How the implementation will work.

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-04-17T18:32Z | — | new | philippepascal |
| 2026-04-17T18:33Z | new | groomed | claude-0417-1645-sync1 |
| 2026-04-17T18:34Z | groomed | in_design | claude-0417-1645-sync1 |