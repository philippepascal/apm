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
| 2026-06-10T02:49Z | — | new | philippepascal |
| 2026-06-12T07:52Z | new | groomed | philippepascal |
| 2026-06-12T08:08Z | groomed | in_design | philippepascal |