+++
id = 4
title = "Implement apm sync (merge detection)"
state = "closed"
priority = 2
effort = 4
risk = 3
updated_at = "2026-03-27T00:06:00.192886Z"
+++

## Spec

### Problem

`apm sync` is the heartbeat of APM's event delivery model. It has two jobs:
(1) refresh the local ticket cache from all `ticket/*` remote branches; (2) detect
merged ticket branches and fire `event:pr_all_merged` to auto-transition tickets
from `implemented → accepted`. The current implementation does job (1) and prints
a hint for (2) but never actually fires the transition. Without it, tickets stay in
`implemented` forever after a PR is merged.

### Acceptance criteria

- [ ] `apm sync` runs `git fetch --all` to update remote tracking refs (unless `--offline`)
- [ ] Reads each `ticket/*` remote branch and writes the ticket file to the local cache
- [ ] For each ticket in `implemented` state whose branch appears in `git branch -r --merged origin/main`: transitions to `accepted`, appends history row
- [ ] Each such transition is committed to `main` via `git::commit_to_branch(root, "main", ...)` — this is the one APM-originated commit to main
- [ ] Prints `#<id>: implemented → accepted (branch merged)` for each transitioned ticket
- [ ] `apm sync --offline` skips `git fetch --all` (re-processes local branches only)
- [ ] `apm sync --quiet` suppresses non-error output
- [ ] Sync is idempotent: running twice has no additional effect

### Out of scope

- Provider API polling (PR opened event, review state) — future ticket
- SQLite cache refresh — future ticket
- `apm sync` as a daemon or scheduled process

### Approach

Replace the stub in `cmd/sync.rs`:
1. Unless `--offline`: call `git::fetch_all(root)`
2. Call `git::ticket_branches(root)` and populate local cache (already done)
3. Call `git::merged_into_main(root)` to get merged branches
4. For each merged branch: load the local ticket file; if state == `"implemented"`:
   - Update state, updated, append history; serialize
   - Call `git::commit_to_branch(root, "main", &rel_path, &content, &msg)`
   - Print the transition line (unless `--quiet`)
5. Add `--offline` and `--quiet` flags to the `sync` subcommand in `main.rs`

## History

| Date | Actor | Transition | Note |
|------|-------|------------|------|
| 2026-03-25 | manual | new → specd | |
| 2026-03-25 | manual | specd → ready | |
| 2026-03-26 | manual | ready → ready | Respec: actually fire transition via commit_to_branch; add --offline/--quiet |
| 2026-03-26 | manual | ready → specd | |
| 2026-03-26 | manual | specd → ready | |
| 2026-03-27T00:06Z | ready | closed | apm |