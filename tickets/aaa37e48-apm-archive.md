+++
id = "aaa37e48"
title = "apm archive"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "apm"
branch = "ticket/aaa37e48-apm-archive"
created_at = "2026-04-03T00:33:18.924269Z"
updated_at = "2026-04-04T06:25:35.886928Z"
+++

## Spec

### Problem

As tickets are closed over time, the `tickets/` directory on `main` accumulates stale files indefinitely. While `apm list` hides terminal-state tickets by default, the files remain on disk and clutter the working directory for anyone browsing the repository. There is no automated way to sweep closed ticket files into a separate archive location.

This ticket adds `apm archive`, a command that moves closed ticket files from the active `tickets/` directory to a configurable archive directory on `main`. It also adds the `archive_dir` config key to `[tickets]` in `config.toml`, and extends the `apm show` fallback path so that archived tickets (whose per-ticket branch was later deleted by `apm clean --branches`) remain discoverable.

### Acceptance criteria

- [ ] `apm archive` errors with a clear message when `archive_dir` is not set in `[tickets]` config
- [ ] `apm archive` moves all terminal-state ticket files from `tickets/<id>-<slug>.md` to `<archive_dir>/<id>-<slug>.md` on the default branch in a single commit
- [ ] `apm archive --dry-run` prints the list of files that would be moved without modifying any branches
- [ ] `apm archive --older-than 30d` limits the batch to tickets whose `updated_at` is older than the threshold (same syntax as `apm clean --older-than`)
- [ ] `apm archive` skips ticket files that are not present in `tickets/` on the default branch and emits a per-ticket warning
- [ ] `apm archive` skips ticket files that are in a non-terminal state and emits a per-ticket warning
- [ ] `apm archive` prints a summary line: `archived N ticket(s)` (or `nothing to archive` when N = 0)
- [ ] `apm show <id>` succeeds for a ticket whose per-ticket branch has been deleted, when the ticket file exists in `archive_dir` on the default branch
- [ ] `[tickets] archive_dir = "archive/tickets"` in `config.toml` is accepted and loaded without error

### Out of scope

- Auto-archiving when a ticket transitions to a terminal state (i.e. no side effect on `apm state` or `apm close`) — that can be a follow-on ticket
- Restoring an archived ticket back to `tickets/` (no `apm unarchive`)
- Archiving epic branches or epic-related files
- Deleting the per-ticket git branch or worktree — that is `apm clean`'s job
- Remote branch pruning — handled by `apm clean --remote`
- Support for multiple archive directories or per-epic archive paths

### Approach

How the implementation will work.

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-04-03T00:33Z | — | new | apm |
| 2026-04-04T06:01Z | new | groomed | apm |
| 2026-04-04T06:25Z | groomed | in_design | philippepascal |