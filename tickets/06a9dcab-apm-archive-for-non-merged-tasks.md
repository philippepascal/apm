+++
id = "06a9dcab"
title = "apm archive for non merged tasks"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/06a9dcab-apm-archive-for-non-merged-tasks"
created_at = "2026-04-28T07:11:58.042694Z"
updated_at = "2026-04-28T07:31:23.624834Z"
+++

## Spec

### Problem

`apm archive --older-than N` reads ticket state exclusively from the default branch (e.g., `main`). For tickets whose `target_branch` is an epic rather than main, the ticket file on main may reflect an intermediate state (e.g., `implemented`) because the epic has not yet been merged into main. The ticket's authoritative state lives on its own `ticket/*` branch — which `apm show` correctly reads and reports as `closed`.

The result: archive incorrectly warns "non-terminal state 'implemented' — skipping" and refuses to archive a ticket that is genuinely closed. Users have to work around this by manually verifying and cannot rely on `apm archive` to clean up after epic-based workflows.

Root cause: `archive.rs` calls `git::read_from_branch(root, default_branch, rel_path)` (line 48) and then checks `terminal_states.contains(&t.frontmatter.state)` (line 65) against the default-branch version only. It never consults the ticket's own branch, even though the `branch` frontmatter field is always set and `git::read_from_branch` already supports local-then-remote fallback.

### Acceptance criteria

- [ ] `apm archive` archives a ticket whose state on the default branch is non-terminal but whose ticket branch (the `branch` frontmatter field) has a terminal state
- [ ] The file written to the archive directory contains content sourced from the ticket branch, not the stale default-branch version
- [ ] The `--older-than` filter uses `updated_at` from the ticket-branch content when the ticket-branch fallback is taken
- [ ] `apm archive` still skips (with a warning) a ticket that is non-terminal on both the default branch and its ticket branch
- [ ] `apm archive` still skips (with a warning) a ticket that is non-terminal on the default branch and has no `branch` frontmatter field
- [ ] `apm archive` still skips (with a warning) a ticket that is non-terminal on the default branch and whose ticket branch cannot be read
- [ ] Dry-run mode applies the same branch-fallback logic (a ticket eligible via the ticket branch appears in dry-run output)

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
| 2026-04-28T07:11Z | — | new | philippepascal |
| 2026-04-28T07:13Z | new | groomed | philippepascal |
| 2026-04-28T07:31Z | groomed | in_design | philippepascal |