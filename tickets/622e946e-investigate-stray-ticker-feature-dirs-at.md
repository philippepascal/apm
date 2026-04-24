+++
id = "622e946e"
title = "Investigate stray ticker-feature-* dirs at repos root"
state = "closed"
priority = 0
effort = 2
risk = 1
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/622e946e-investigate-stray-ticker-feature-dirs-at"
created_at = "2026-04-24T06:29:26.733092Z"
updated_at = "2026-04-24T08:01:43.403962Z"
+++

## Spec

### Problem

Three directories at `/Users/philippepascal/repos/` do not match the current apm worktree convention:

```
ticker-feature-1-export-xlsx      (created ~2026-03-24)
ticker-feature-3-grow-formula     (created ~2026-04-08)
ticker-feature-6-website-metrics  (created ~2026-03-25)
```

**What they are:** Legitimate git worktrees of `/Users/philippepascal/repos/ticker`, registered in that repo's worktree list. Each is checked out on a `feature/<n>-<slug>` branch. They were created under an older convention where worktrees were placed directly in `~/repos/` as `<project>-<branch-path-hyphenated>`. The current apm convention places worktrees under `<project>--worktrees/<branch-slug>` (double-dash separator, one level in).

**Why they are stray:** Current apm code (`apm-core/src/worktree.rs`, `ensure_worktree()`) constructs paths as `worktrees_base + branch_slug` where `worktrees_base` defaults to `../worktrees` relative to the main repo root. It cannot produce sibling paths at the `~/repos/` level. These worktrees pre-date or were created outside of current apm.

**Branch state:**
- `feature/1-export-xlsx` — pushed to `origin/feature/1-export-xlsx`
- `feature/3-grow-formula` — pushed to `origin/feature/3-grow-formula`
- `feature/6-website-metrics` — no remote tracking branch; tip commit `fce80c4` is shared with `feature/3-grow-formula`, suggesting its work was absorbed

No active development is happening in these worktrees (no apm tickets reference them, dates are weeks old). Leaving them wastes disk space and pollutes `git worktree list` output for the `ticker` project.

### Acceptance criteria

- [x] `git -C /Users/philippepascal/repos/ticker worktree list` no longer shows any `ticker-feature-*` entry
- [x] The three directories no longer exist under `/Users/philippepascal/repos/`
- [x] Work in `feature/1-export-xlsx` is confirmed present on `origin/feature/1-export-xlsx` before removal
- [x] Work in `feature/3-grow-formula` is confirmed present on `origin/feature/3-grow-formula` before removal
- [x] Work in `feature/6-website-metrics` is confirmed absorbed (tip commit `fce80c4` reachable from at least one remote branch) before removal
- [x] `git -C /Users/philippepascal/repos/ticker worktree prune` exits cleanly after removal

### Out of scope

- Merging or closing the `feature/*` branches on GitHub — separate ticker project concern
- Changing apm's worktree naming convention or adding a guard against placing worktrees outside `--worktrees/`
- Cleanup of any `ticker--worktrees/` directories (those follow current apm convention and are in scope for other tickets)
- Determining exactly how the old-convention worktrees were originally created — the source is not actionable; only cleanup matters

### Approach

All commands run against the ticker main repo. No apm code changes are required.

1. **Confirm feature/6 is absorbed** — run:
   ```
   git -C /Users/philippepascal/repos/ticker branch -r --contains fce80c4
   ```
   Expected output includes `origin/feature/3-grow-formula` or similar. If the commit is unreachable from any remote, push `feature/6-website-metrics` first.

2. **Remove the three worktrees** (one Bash call each; order does not matter):
   ```
   git -C /Users/philippepascal/repos/ticker worktree remove /Users/philippepascal/repos/ticker-feature-6-website-metrics
   git -C /Users/philippepascal/repos/ticker worktree remove /Users/philippepascal/repos/ticker-feature-1-export-xlsx
   git -C /Users/philippepascal/repos/ticker worktree remove /Users/philippepascal/repos/ticker-feature-3-grow-formula
   ```
   If git complains about uncommitted changes use `--force` only after confirming nothing untracked is needed.

3. **Prune** to remove stale worktree metadata:
   ```
   git -C /Users/philippepascal/repos/ticker worktree prune
   ```

4. **Verify** the list is clean:
   ```
   git -C /Users/philippepascal/repos/ticker worktree list
   ```
   Should show only the main worktree (`/Users/philippepascal/repos/ticker`) and any current `ticker--worktrees/` entries.

This ticket closes once the three directories are gone and `worktree list` is clean. No source code changes.

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-04-24T06:29Z | — | new | philippepascal |
| 2026-04-24T07:13Z | new | groomed | philippepascal |
| 2026-04-24T07:19Z | groomed | in_design | philippepascal |
| 2026-04-24T07:23Z | in_design | specd | claude-0424-0719-b4d0 |
| 2026-04-24T07:25Z | specd | ready | philippepascal |
| 2026-04-24T07:48Z | ready | in_progress | philippepascal |
| 2026-04-24T07:50Z | in_progress | implemented | claude-0424-0748-3d80 |
| 2026-04-24T08:01Z | implemented | closed | philippepascal(apm-sync) |
