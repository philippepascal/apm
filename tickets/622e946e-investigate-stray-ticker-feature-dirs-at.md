+++
id = "622e946e"
title = "Investigate stray ticker-feature-* dirs at repos root"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/622e946e-investigate-stray-ticker-feature-dirs-at"
created_at = "2026-04-24T06:29:26.733092Z"
updated_at = "2026-04-24T07:19:38.416712Z"
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
| 2026-04-24T07:19Z | groomed | in_design | philippepascal |