+++
id = "2973e208"
title = "Add apm refresh-epic command with epic quiescence check"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/2973e208-add-apm-refresh-epic-command-with-epic-q"
created_at = "2026-04-27T20:28:30.358011Z"
updated_at = "2026-04-27T21:07:40.360079Z"
epic = "5ea30227"
target_branch = "epic/5ea30227-strategy-and-dependency-hardening"
+++

## Spec

### Problem

Long-running epic branches drift from the default branch over time. There is no built-in APM command to pull default-branch updates into an epic branch. The spec at `docs/strategy-and-dependencies.md` (§ 'Refresh and close: epic must be quiescent') defines `apm refresh-epic <id>` as the supervisor-facing tool for this: it opens a PR from the default branch into the epic branch, which the supervisor reviews and merges so subsequent workers in the epic see the updated tip.

The command must refuse to run if any ticket in the epic is currently being worked on (i.e., in a state that is neither terminal nor `worker_end`, such as `in_design` or `in_progress`) or has a live worker process (alive `.apm-worker.pid`). This precondition is shared with `apm epic close` (ticket 056b1ee1), so the check must be extracted into a reusable `epic_is_quiescent()` helper in `apm-core`.

APM does not stop running workers; the supervisor is responsible for pausing the dispatcher and waiting for the active worker to complete before calling this command.

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
| 2026-04-27T20:28Z | — | new | philippepascal |
| 2026-04-27T20:43Z | new | groomed | philippepascal |
| 2026-04-27T21:07Z | groomed | in_design | philippepascal |