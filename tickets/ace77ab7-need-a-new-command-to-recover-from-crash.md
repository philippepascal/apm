+++
id = "ace77ab7"
title = "need a new command to recover from crashed agents"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/ace77ab7-need-a-new-command-to-recover-from-crash"
created_at = "2026-08-28T00:51:22.995736Z"
updated_at = "2026-08-28T07:26:44.937595Z"
+++

## Spec

### Problem

Worker agents (spec-writer, coder) run inside per-ticket git worktrees and
record themselves in a `.apm-worker.pid` file when `apm start`/`apm work`
dispatches them. `apm workers` already detects when a worker has died
mid-task — it compares `.apm-worker.pid`'s PID against `ps` and, if the PID
is dead but the ticket's state is neither terminal nor `worker_end`, labels
the row `crashed`. That diagnosis is display-only: nothing acts on it.

The practical effect differs by state. A ticket stuck in `in_progress`
(coder crashed) can technically be un-stuck today via the existing
`in_progress → ready` manual transition, but the supervisor has to notice
the crash via `apm workers`, run `apm state <id> ready` by hand, and
separately remember to delete the stale `.apm-worker.pid` (nothing removes
it automatically, so the ticket keeps showing as `crashed` even after the
state is fixed). A ticket stuck in `in_design` (spec-writer crashed) has no
manual fallback transition at all — `in_design` only declares `specd` and
`question` as valid next states — so recovering it requires reaching for
`apm state <id> <state> --force`, which bypasses every transition guard and
can push the ticket into any state, not just a safe rollback.

This gets worse at scale: `apm work --daemon` can run several workers
concurrently, and a host restart or OOM event can crash all of them at
once, leaving a batch of tickets frozen with no single command to sweep
and recover them. We need a command that takes the existing crash
diagnosis and turns it into a safe, repeatable recovery action.

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
| 2026-08-28T00:51Z | — | new | philippepascal |
| 2026-08-28T07:13Z | new | groomed | philippepascal |
| 2026-08-28T07:26Z | groomed | in_design | philippepascal |