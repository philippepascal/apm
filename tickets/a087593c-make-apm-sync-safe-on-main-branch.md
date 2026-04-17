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
| 2026-04-17T18:32Z | — | new | philippepascal |
| 2026-04-17T18:33Z | new | groomed | claude-0417-1645-sync1 |
| 2026-04-17T18:34Z | groomed | in_design | claude-0417-1645-sync1 |