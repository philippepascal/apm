+++
id = 46
title = "apm work: headless ticket orchestration without a supervisor session"
state = "ammend"
priority = 0
effort = 5
risk = 3
author = "claude-0328-c72b"
branch = "ticket/0046-apm-work-headless-ticket-orchestration-w"
created_at = "2026-03-28T19:42:39.548558Z"
updated_at = "2026-03-29T19:11:14.534410Z"
+++

## Spec

### Problem

`apm start --spawn <id>` (ticket #37) is a primitive that a Claude Code supervisor
session uses to hand individual tickets to background workers. It requires a running
supervisor to decide which tickets to start and in what order.

For fully automated runs — CI pipelines, overnight batch runs, or cases where the
user simply wants all ready tickets worked without opening a Claude Code session —
there is no entry point. A standalone `apm work` command would fill this gap: find
all actionable tickets, spawn a worker per ticket (up to the configured
`agents.max_concurrent` limit), and exit when all workers have finished.

This ticket is a placeholder. It depends on #37 (`apm start --spawn`) being
implemented and proven stable first.

### Acceptance criteria

- [ ] `apm work` finds all tickets in `ready` state and spawns one worker per ticket via the same mechanism as `apm start --spawn`
- [ ] Concurrency is capped at `agents.max_concurrent` from `apm.toml`; additional tickets are queued and started as slots free up
- [ ] `--skip-permissions` / `-P` flag passes through to all spawned workers (same semantics as `apm start --spawn -P`)
- [ ] `apm work` blocks until all spawned workers have exited, then prints a summary (ticket id, final state, log path)
- [ ] `apm work --dry-run` prints which tickets would be started without spawning anything
- [ ] `apm work` exits non-zero if any worker ended in a state other than `implemented`

### Out of scope

- Spec-writing workers (only `ready` tickets are eligible; `new` / `ammend` require supervisor judgment)
- Worker monitoring UI or live log tailing
- Automatic retry of blocked or failed workers
- Any changes to `apm start` itself

### Approach

_To be written once #37 is implemented and the worker model is validated in practice._

Likely: `apm/src/cmd/work.rs` wraps the same spawn logic extracted from
`start.rs` into a shared helper, loops over `ready` tickets respecting
`max_concurrent`, and waits for all child processes.

### Amendment requests

- [ ] `apm work` is the delegator loop, not just a worker launcher for `ready`
  tickets. It should call `apm start --next` in a loop (a separate ticket),
  which internally finds the next actionable ticket across all states with
  `trigger: command:start` — including `new → in_design` (spec-writing) and
  `ready → in_progress` (implementation). Update the acceptance criteria to
  reflect this broader scope.
- [ ] Remove the "out of scope" exclusion of spec-writing workers. Under the
  delegator model, `apm work` does not decide what kind of agent to spawn —
  `apm start --next` does, by reading the `instructions` property from the
  target state config.
- [ ] The acceptance criteria mention `ready` state specifically; replace with
  "any state where `actionable = ['agent']` and `agent` is unset and a
  `trigger: command:start` transition exists."
- [ ] Update the approach section now that `apm start --next` (separate ticket)
  exists as the primitive. `apm work` is a thin loop around it.

## History

| When | From | To | By |
|------|------|----|----|
| 2026-03-28T19:42Z | — | new | claude-0328-c72b |
| 2026-03-28T19:43Z | new | specd | claude-0328-c72b |
| 2026-03-29T19:11Z | specd | ammend | claude-0329-1200-a1b2 |
