+++
id = "a087593c"
title = "Make apm sync safe on main branch"
state = "in_progress"
priority = 0
effort = 4
risk = 5
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/a087593c-make-apm-sync-safe-on-main-branch"
created_at = "2026-04-17T18:32:29.530485Z"
updated_at = "2026-04-17T19:01:11.439056Z"
epic = "47375a6a"
target_branch = "epic/47375a6a-safer-apm-sync"
depends_on = ["5cf54181"]
+++

## Spec

### Problem

`apm sync` currently mishandles the default branch (`main`) in two ways:

1. **Blind push on inequality.** `push_default_branch` (`apm-core/src/git_util.rs:33`) pushes whenever local and `origin/<default>` SHAs differ — with no regard for direction. When origin is ahead (e.g. after the user merged an epic PR on GitHub), the push is rejected non-fast-forward and surfaces as a `warning: push main failed: …` line.
2. **No fast-forward.** After the fetch step, `origin/main` ref is updated locally, but the local `main` branch and main worktree are never fast-forwarded. Users have to run `git pull` manually before `apm sync` does anything useful post-merge.

Additionally, per the review decision captured in the design doc, **apm sync must not push anything automatically** — explicit pushes happen via `apm state <id> implemented` and equivalents. Sync's job on `main` is to (a) fetch, (b) fast-forward local when possible, (c) inform the user when local is ahead/diverged, and (d) never block on a push attempt that cannot succeed.

See `/Users/philippepascal/Documents/apm/apm-sync-scenarios.md` for the full scenario matrix, algorithm, and guidance strings. Implementers must add comments in the sync module explaining the local/remote classification states and why each maps to its action — the logic is not intuitive at a glance.

### Acceptance criteria

- [x] `apm sync` never calls `git push` on the default branch (default push path removed entirely)
- [x] When local `main` equals `origin/main`: sync prints nothing about main and makes no ref changes
- [x] When local `main` is strictly behind `origin/main` and the working tree does not conflict: local `main` is fast-forwarded via `git merge --ff-only origin/<default>` on the main worktree
- [x] When local `main` is behind but the FF would overwrite uncommitted local changes: sync prints the "main behind, FF blocked" guidance block and leaves the working tree untouched
- [x] When local `main` is strictly ahead of `origin/main`: sync prints a single info line ("main is ahead of origin/<default> by N commits — run `git push` when ready") and makes no network call
- [x] When local `main` and `origin/main` have diverged: sync prints the divergence guidance (rebase/merge choice) and does not modify local main or the working tree
- [x] When `origin/main` cannot be resolved (no remote, unreachable, fetch failed): main is skipped silently; any fetch failure surfaces as a single warning line from the existing fetch path
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

**1. Remove existing push.** In `apm/src/cmd/sync.rs`, delete the `git::push_default_branch(...)` call and its warning block (currently lines 13-15). In `apm-core/src/git_util.rs`, delete `push_default_branch` entirely — no other caller uses it (verify with a grep before deleting).

**2. Add classification helper.** In `apm-core/src/git_util.rs`, introduce:

```rust
// Classify a local branch relative to its origin counterpart.
// Direction note: `merge-base --is-ancestor A B` returns 0 iff A is reachable from B.
//   - local == remote       → Equal
//   - local ancestor-of remote (and not equal) → Behind (FF possible)
//   - remote ancestor-of local (and not equal) → Ahead
//   - neither ancestor       → Diverged
//   - remote ref missing     → NoRemote
pub enum BranchClass { Equal, Behind, Ahead, Diverged, NoRemote }

pub fn classify_branch(root: &Path, local: &str, remote: &str) -> BranchClass { ... }
```

Implemented with `git rev-parse` for SHA equality and `git merge-base --is-ancestor` for directed ancestry. Comments at every ancestor check spelling out which direction maps to which state.

**3. Add `sync_default_branch`.** In `apm-core/src/git_util.rs`:

```rust
// Bring local <default_branch> into sync with origin without ever pushing.
// Matrix:
//   Equal     → no-op
//   Behind    → git merge --ff-only origin/<default>; if it errors (dirty overlap), print guidance
//   Ahead     → info line only (no push — apm sync never pushes; push happens via apm state)
//   Diverged  → guidance
//   NoRemote  → silent skip
pub fn sync_default_branch(root: &Path, default: &str, warnings: &mut Vec<String>) { ... }
```

FF is executed against the main worktree by running `git merge --ff-only origin/<default>` with `root` as the working directory (main worktree is always on `main` per the project's hard rule). No checkout or ref surgery is needed.

**4. Wire into sync.rs.** Replace the removed push block with a call to `sync_default_branch`. Order in sync.rs becomes: fetch → `sync_local_ticket_refs` (unchanged in this ticket) → `sync_default_branch`.

**5. Guidance strings.** Ticket `5cf54181` defines the shared guidance module. For this ticket, consume the `main-behind-dirty-overlap` and `main-diverged-*` strings from that module (depends_on relationship is declared at the epic level). If ticket A lands before C is merged, inline the two strings temporarily with a `// TODO(5cf54181): move to sync_guidance` comment — prefer landing C first.

**6. Tests.** Add integration tests in `apm/tests/integration.rs`:
- `sync_main_equal_noop`
- `sync_main_behind_ff_clean`
- `sync_main_behind_ff_blocked_by_dirty_overlap` — stages a conflicting local change before sync, asserts guidance is printed and local `main` SHA is unchanged
- `sync_main_ahead_prints_info_no_push` — asserts no `git push` hits origin (use a bare-repo origin and verify its tip)
- `sync_main_diverged_prints_guidance` — creates divergent commits on both sides
- `sync_main_no_remote_skips` — sync works on a repo with no origin configured

**7. Comments.** Over and above inline comments in classify_branch, add a block comment at the top of `sync_default_branch` listing the matrix rows it covers. The user flagged this logic as "not intuitive" — err toward more comments, not fewer. Do not write comments explaining trivial code.

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-04-17T18:32Z | — | new | philippepascal |
| 2026-04-17T18:33Z | new | groomed | claude-0417-1645-sync1 |
| 2026-04-17T18:34Z | groomed | in_design | claude-0417-1645-sync1 |
| 2026-04-17T18:42Z | in_design | specd | claude-0417-1645-sync1 |
| 2026-04-17T18:48Z | specd | ready | apm |
| 2026-04-17T19:01Z | ready | in_progress | philippepascal |