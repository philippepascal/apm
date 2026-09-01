+++
id = "bc8919b4"
title = "when dealing with epic, apm sync should be more advance"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/bc8919b4-when-dealing-with-epic-apm-sync-should-b"
created_at = "2026-09-01T18:00:23.105706Z"
updated_at = "2026-09-01T18:03:53.553660Z"
+++

## Spec

### Problem

`apm sync` already detects epics whose tickets are all in a terminal state (`Candidates.epic_submit_hints` in `apm-core/src/sync.rs`) and epics whose branch is already merged into the default branch but not yet deleted (`Candidates.epic_close_hints`). Today `apm/src/cmd/sync.rs` only *prints* these as passive text ("Epics ready to submit (apm epic submit <id>):" / "...to close..."). The operator has to remember to run `apm epic submit <id>` and `apm epic close <id>` by hand afterwards, once per epic, choosing the right submit method (merge / PR / auto) themselves.

Two gaps compound this. First, the hints are computed once, before `sync::apply` closes this run's tickets, so an epic that only becomes fully done as a direct result of tickets closing during *this* `apm sync` invocation is not reported until the *next* `apm sync` run. Second, there is no way to act on a hint in the moment — the operator must re-run a separate command per epic, even though `apm epic submit`/`apm epic close` already implement the exact decision needed (see `run_refresh_epic`'s existing `[1] Merge / [2] PR / [3] Auto / [4] Skip` menu, which this ticket reuses in spirit).

This affects operators running `apm sync` interactively at the end of a work session, which is precisely when several tickets in an epic tend to finish in the same pass.

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
| 2026-09-01T18:00Z | — | new | philippepascal |
| 2026-09-01T18:03Z | new | groomed | philippepascal |
| 2026-09-01T18:03Z | groomed | in_design | philippepascal |