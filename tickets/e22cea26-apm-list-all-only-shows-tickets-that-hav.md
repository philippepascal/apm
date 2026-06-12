+++
id = "e22cea26"
title = "apm list --all only shows tickets that have branches."
state = "in_design"
priority = 0
effort = 3
risk = 2
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/e22cea26-apm-list-all-only-shows-tickets-that-hav"
created_at = "2026-06-10T02:49:43.077397Z"
updated_at = "2026-06-12T08:12:54.276739Z"
+++

## Spec

### Problem

`apm list --all` sources tickets exclusively from `ticket/` branches (local and remote). Once a ticket's branch is deleted — typically after GitHub merges the PR and auto-deletes the branch — the ticket file remains in the `tickets/` directory on the default branch but has no corresponding `ticket/` branch. Those tickets are invisible to every `apm list` invocation, including `--all`.

The desired behaviour is that `apm list --all` also surfaces tickets whose file is present in `tickets/` on the default branch but whose `ticket/` branch no longer exists. Archived tickets (moved to a separate `archive_dir`) are out of scope and should remain excluded.

### Acceptance criteria

- [ ] `apm list --all` shows a ticket whose `ticket/` branch has been deleted but whose `.md` file is present in `tickets/` on the default branch.
- [ ] `apm list --all` continues to show tickets that have an active `ticket/` branch.
- [ ] `apm list` (without `--all`) hides tickets from the default branch that are in a terminal state, consistent with the existing terminal-state hiding behaviour.
- [ ] When the same ticket is found both on a `ticket/` branch and in `tickets/` on the default branch, it appears exactly once in the output.
- [ ] `apm list --all` produces no error when the `tickets/` directory does not exist on the default branch.
- [ ] Ticket files in `archive_dir` (when configured) are not included in `apm list --all`.

### Out of scope

- Archived tickets (files in `archive_dir`) — they are intentionally excluded from `apm list`.
- `apm show` finding tickets without branches — existing fallback in `state.rs` already handles this.
- Visual indication in `apm list` output that a ticket has no live branch.
- `apm next`, `apm start`, and other commands that call `load_all_from_git` — those callers do not need branchless tickets for their purposes.

### Approach

#### New function in `apm-core/src/ticket/ticket_util.rs`

Add `pub fn load_from_default_branch(root: &Path, tickets_dir_rel: &Path, default_branch: &str) -> Result<Vec<Ticket>>`:

1. Call `crate::git::list_files_on_branch(root, default_branch, &tickets_dir_rel.to_string_lossy())` to get relative paths of all files in `tickets/` on the default branch. Ignore errors (returns `Ok(vec![])` on missing tree).
2. Filter paths to those ending in `.md`.
3. For each path, read via `crate::git::read_from_branch(root, default_branch, &rel_path)` and parse via `Ticket::parse`. Skip silently on error.
4. Return the collected list.

Because `ticket.rs` does `pub use ticket_util::*`, the function is automatically exported as `apm_core::ticket::load_from_default_branch` with no module changes needed.

#### Merge into `CmdContext::load` in `apm/src/ctx.rs`

After the existing `tickets` load (both the aggressive and non-aggressive branches), add:

1. Call `apm_core::ticket::load_from_default_branch(root, &config.tickets.dir, &config.project.default_branch)?`.
2. Collect the IDs already present in `tickets` into a `HashSet<&str>`.
3. Extend `tickets` with branchless tickets whose ID is not in the set.
4. Re-sort `tickets` by `created_at` (the existing sort order).

No other callers of `load_all_from_git` change — they do not go through `CmdContext` and do not need branchless tickets.

#### Tests

- Unit test in `ticket_util.rs`: create a temp git repo, commit a ticket file to the default branch without creating a `ticket/` branch, call `load_from_default_branch`, assert the ticket is returned.
- Unit test for dedup: load a ticket from both a branch and the default branch, assert it appears once in the merged result.
- The existing `list_filtered` tests for terminal-state hiding apply unchanged; terminal tickets from the default branch are hidden by the same `terminal_ok` filter.

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-06-10T02:49Z | — | new | philippepascal |
| 2026-06-12T07:52Z | new | groomed | philippepascal |
| 2026-06-12T08:08Z | groomed | in_design | philippepascal |