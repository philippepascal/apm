+++
id = "06a9dcab"
title = "apm archive for non merged tasks"
state = "ready"
priority = 0
effort = 3
risk = 2
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/06a9dcab-apm-archive-for-non-merged-tasks"
created_at = "2026-04-28T07:11:58.042694Z"
updated_at = "2026-04-28T15:13:49.521957Z"
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

- Fetching from remote as part of `apm archive` (no network calls added, no `--aggressive` flag)
- Changing how the default branch is scanned to discover ticket files
- Handling tickets that exist on their ticket branch but are absent from the default branch entirely
- Syncing or updating the default-branch copy of the ticket file during archiving

### Approach

**Only file changed:** `apm-core/src/archive.rs`

Replace the non-terminal-state check block (currently lines 65-71) with logic that falls back to the ticket's own branch when the default-branch version is non-terminal. The existing `content` and `t` bindings need to be re-bound (shadowed) to the ticket-branch values when the fallback succeeds, because `content` is used later in `moves.push(...)` and `t.frontmatter.updated_at` is used in the `older_than` check.

After parsing `t` from the default branch (line 57), replace the current non-terminal check with a three-way match:
- If the default-branch state is terminal: proceed unchanged.
- If non-terminal and `t.frontmatter.branch` is Some: call `git::read_from_branch(root, ticket_branch, rel_path)`, re-parse into a new Ticket, and if that version has a terminal state, shadow both `t` and `content` with the ticket-branch values and fall through into the `older_than` / move logic. On read error or still-non-terminal state, emit the warning and continue.
- If non-terminal and no `branch` field: emit the warning and continue (existing behaviour, no change).

The `content` variable that feeds `moves.push(...)` must be the ticket-branch content when the fallback is taken, so the archived file reflects the closed state rather than the stale default-branch state.

No changes to `git_util.rs` are required; `read_from_branch` already handles local-then-remote fallback for any branch name.

Add a unit test covering the case where the default-branch ticket is non-terminal but the ticket-branch version is terminal. The test should verify that the ticket appears in moves (or dry_run_moves) and that warnings is empty.

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-04-28T07:11Z | — | new | philippepascal |
| 2026-04-28T07:13Z | new | groomed | philippepascal |
| 2026-04-28T07:31Z | groomed | in_design | philippepascal |
| 2026-04-28T07:36Z | in_design | specd | claude-0428-0731-3bf0 |
| 2026-04-28T15:13Z | specd | ready | philippepascal |
