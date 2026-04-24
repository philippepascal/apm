+++
id = "622e946e"
title = "Investigate stray ticker-feature-* dirs at repos root"
state = "groomed"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/622e946e-investigate-stray-ticker-feature-dirs-at"
created_at = "2026-04-24T06:29:26.733092Z"
updated_at = "2026-04-24T07:13:29.807160Z"
+++

## Spec

### Problem

Three unexplained directories at /Users/philippepascal/repos/: ticker-feature-1-export-xlsx, ticker-feature-3-grow-formula, ticker-feature-6-website-metrics (all dated Mar 24, older than current apm worktree work). Naming does not match apm conventions (which expect ticket-<id>-<slug> or epic-<id>-<slug> under <project>--worktrees/). Likely NOT from current apm — but could be from an earlier version, a past supervisor-run git worktree add, or an unrelated tool. Expected: first identify the source. Starting points: run "git -C <dir> worktree list 2>&1" and "git -C <dir> log -n 3 --oneline" in each to see if they are legitimate git worktrees of some repo. If apm-related even from older version, document cleanup and verify current code does not produce these paths. If unrelated, close with a note. File-system hygiene only; not blocking any active work.

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
