+++
id = "e22cea26"
title = "apm list --all only shows tickets that have branches."
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/e22cea26-apm-list-all-only-shows-tickets-that-hav"
created_at = "2026-06-10T02:49:43.077397Z"
updated_at = "2026-06-12T08:08:37.774713Z"
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

How the implementation will work.

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-06-10T02:49Z | — | new | philippepascal |
| 2026-06-12T07:52Z | new | groomed | philippepascal |
| 2026-06-12T08:08Z | groomed | in_design | philippepascal |