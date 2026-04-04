+++
id = "01dbdaad"
title = "Replace apm take with apm assign: set owner on any ticket"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "apm"
branch = "ticket/01dbdaad-replace-apm-take-with-apm-assign-set-own"
created_at = "2026-04-04T06:33:40.535848Z"
updated_at = "2026-04-04T07:06:39.505941Z"
epic = "8db73240"
target_branch = "epic/8db73240-user-mgmt"
depends_on = ["ffaad988"]
+++

## Spec

### Problem

`apm take` exists today as a "takeover" command — it writes a handoff entry to History and provisions a worktree, but it doesn't track the old or new owner in frontmatter (always logs "unknown"). It also only works for the current agent taking over, not for a supervisor assigning someone else.

With the `owner` field, `apm take` becomes redundant and underspecified. What's needed instead is `apm assign <id> <username>` — a supervisor action that sets the `owner` field on any ticket regardless of state. This replaces both the self-takeover use case (I want to own this ticket) and the delegation use case (supervisor assigns a ticket to someone). The old `apm take` command, its CLI entry, server endpoint (`/api/tickets/:id/take`), and `handoff()` function should be removed.

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
| 2026-04-04T06:33Z | — | new | apm |
| 2026-04-04T06:35Z | new | groomed | apm |
| 2026-04-04T07:06Z | groomed | in_design | philippepascal |
