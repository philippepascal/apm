+++
id = "a5e1ea24"
title = "Implement apm epic list command"
state = "in_design"
priority = 6
effort = 0
risk = 0
author = "claude-0401-2145-a8f3"
agent = "87256"
branch = "ticket/a5e1ea24-implement-apm-epic-list-command"
created_at = "2026-04-01T21:55:09.722953Z"
updated_at = "2026-04-02T00:47:06.221425Z"
+++

## Spec

### Problem

Once epic branches exist there is no way to see them or their status at a glance. Engineers and the supervisor need to know which epics are active, how many tickets are in each state, and whether an epic is done.

The full design is in `docs/epics.md` (§ Commands — `apm epic list`). Epic state is always derived — never stored — using these rules: no tickets → `empty`; any ticket `in_design` or `in_progress` → `in_progress`; all `implemented` or later → `implemented`; all `accepted`/`closed` → `done`; otherwise → `in_progress`.

The command lists all `epic/*` remote branches and for each shows: short ID, title (from slug), derived state, and per-state ticket counts (e.g. `2 in_progress, 1 ready, 3 implemented`).

### Acceptance criteria

- [ ] `apm epic list` outputs one line per `epic/*` remote branch
- [ ] Each line shows the 8-char ID, the humanized title (hyphens → spaces, title-cased), derived state, and non-zero per-state ticket counts
- [ ] When no `epic/*` branches exist, the command exits 0 with no output
- [ ] Derived state is `empty` when no tickets reference the epic ID
- [ ] Derived state is `in_progress` when at least one ticket is in state `in_design` or `in_progress`
- [ ] Derived state is `implemented` when all tickets are in state `implemented` or a later non-terminal state (but not all accepted/closed)
- [ ] Derived state is `done` when all tickets are in state `accepted` or `closed`
- [ ] Derived state falls back to `in_progress` for any other mix of states
- [ ] Ticket counts omit states with a zero count (e.g. `2 in_progress, 3 implemented`, not `2 in_progress, 0 ready, 3 implemented`)
- [ ] The command respects the aggressive-fetch setting (same behaviour as `apm list`)

### Out of scope

- `apm epic new`, `apm epic show`, `apm epic close` commands
- Adding the `target_branch` or `depends_on` fields to `Frontmatter`
- `depends_on` scheduling / engine loop changes
- apm-server epic API routes
- apm-ui epic UI additions
- `apm new --epic` flag
- `apm work --epic` exclusive mode

### Approach

How the implementation will work.

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-04-01T21:55Z | — | new | claude-0401-2145-a8f3 |
| 2026-04-01T21:59Z | new | groomed | claude-0401-2145-a8f3 |
| 2026-04-02T00:47Z | groomed | in_design | philippepascal |