+++
id = 4
title = "Implement apm sync (merge detection)"
state = "ready"
priority = 2
effort = 5
risk = 4
created = "2026-03-25"
updated = "2026-03-25"
+++

## Spec

### Problem

`apm sync` is the heartbeat of APM's event delivery model. It has two jobs:
(1) detect merged feature branches and fire `event:pr_all_merged` to move tickets
from `implemented` → `accepted`; (2) fetch from origin so local git state is current.
Currently it is a stub. Without it, tickets stay in `implemented` forever after a PR
is merged.

### Acceptance criteria

- [ ] `apm sync` runs `git fetch origin` to update remote tracking refs
- [ ] `apm sync` enumerates all tickets in `implemented` state
- [ ] For each, checks whether `frontmatter.branch` is present in `git branch -r --merged origin/main` output
- [ ] If merged: transitions ticket to `accepted`, commits frontmatter update to main, prints `#<id>: implemented → accepted (branch merged)`
- [ ] If not merged: no-op for that ticket
- [ ] `apm sync --offline` skips `git fetch origin` (for post-merge hook use)
- [ ] `apm sync --quiet` suppresses non-error output
- [ ] Sync is idempotent: running twice has no additional effect

### Out of scope

- Provider API polling (PR opened event, review state) — future ticket
- SQLite cache refresh — future ticket
- `apm sync` as a daemon or scheduled process

### Approach

In `cmd/sync.rs`, replace the stub:
1. Unless `--offline`: run `git fetch origin`
2. Load all tickets; filter to `implemented`
3. For each: run `git branch -r --merged origin/main`, check if `origin/<branch>` appears
4. On match: update frontmatter state + updated, append history, save, `git add`, accumulate
5. If any tickets updated: single `git commit -m "sync: accept merged tickets #<ids>"` + `git push origin main`

## History

| Date | Actor | Transition | Note |
|------|-------|------------|------|
| 2026-03-25 | manual | new → specd | |
| 2026-03-25 | manual | specd → ready | |
