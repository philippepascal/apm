+++
id = 46
title = "apm work: headless ticket orchestration without a supervisor session"
state = "specd"
priority = 0
effort = 5
risk = 3
author = "claude-0328-c72b"
branch = "ticket/0046-apm-work-headless-ticket-orchestration-w"
created_at = "2026-03-28T19:42:39.548558Z"
updated_at = "2026-03-28T19:43:18.015852Z"
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

## History

| When | From | To | By |
|------|------|----|----|
| 2026-03-28T19:42Z | — | new | claude-0328-c72b |
| 2026-03-28T19:43Z | new | specd | claude-0328-c72b |
