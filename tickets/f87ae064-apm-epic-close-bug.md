+++
id = "f87ae064"
title = "apm epic close bug"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/f87ae064-apm-epic-close-bug"
created_at = "2026-06-05T01:34:06.624276Z"
updated_at = "2026-06-12T07:52:46.175471Z"
+++

## Spec

### Problem

`apm epic close` guards against two unsafe conditions: an active worker process on a ticket in the epic, and an epic branch whose commits have not yet landed in the default branch. It does not check whether the epic's tickets are still in a non-terminal state.

When `apm epic list` shows "implemented" for an epic, every ticket has reached a state with `satisfies_deps = true` but one or more tickets have not yet transitioned to a terminal state (e.g. they remain in `implemented` rather than `closed`). Closing the epic in this condition deletes the branch while those tickets are left stranded in a non-terminal state — no pointer to the work remains, making them difficult to reason about or close afterwards.

### Acceptance criteria

- [ ] `apm epic close <id>` fails when the derived epic state is "implemented", printing an error that lists each non-terminal ticket by id, title, and current state
- [ ] The error message tells the user to close the listed tickets first and notes that `--force` bypasses the check
- [ ] `apm epic close <id> --force` proceeds when the derived state is "implemented" (consistent with how `--force` bypasses the existing guards)
- [ ] `apm epic close <id>` succeeds without the new error when the derived epic state is "done" (all tickets terminal)
- [ ] `apm epic close <id>` succeeds without the new error when the derived epic state is "empty" (no tickets in the epic)

### Out of scope

- Changing behaviour for "in_progress" epics beyond what existing live-worker and unmerged-branch guards already enforce
- Auto-closing non-terminal tickets as part of `apm epic close`
- Adding the same guard to `apm epic submit`

### Approach

How the implementation will work.

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-06-05T01:34Z | — | new | philippepascal |
| 2026-06-12T07:52Z | new | groomed | philippepascal |
| 2026-06-12T07:52Z | groomed | in_design | philippepascal |