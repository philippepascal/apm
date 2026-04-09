+++
id = 46
title = "apm work: headless ticket orchestration without a supervisor session"
state = "closed"
priority = 3
effort = 5
risk = 3
author = "claude-0328-c72b"
agent = "claude-0329-resume"
branch = "ticket/0046-apm-work-headless-ticket-orchestration-w"
created_at = "2026-03-28T19:42:39.548558Z"
updated_at = "2026-03-30T02:02:46.501095Z"
+++

## Spec

### Problem

`apm start --spawn <id>` (ticket #37) is a primitive that a Claude Code supervisor
session uses to hand individual tickets to background workers. It requires a running
supervisor to decide which tickets to start and in what order.

For fully automated runs — CI pipelines, overnight batch runs, or cases where the
user simply wants all actionable tickets worked without opening a Claude Code session —
there is no entry point. A standalone `apm work` command would fill this gap: call
`apm start --next` in a loop, spawning one worker per available ticket up to the
configured `agents.max_concurrent` limit, and exit when all workers have finished.

This ticket depends on #56 (`apm start --next`) being implemented first.

### Acceptance criteria

- [x] `apm work` calls `apm start --next --spawn` repeatedly until no more
  actionable tickets are found, respecting `agents.max_concurrent` from `apm.toml`
- [x] Actionable tickets include any state where `actionable = ["agent"]` (or
  includes `"agent"`) and the ticket has no assigned agent and a
  `trigger: command:start` transition exists — not just `ready`; this covers
  spec-writing (`new` → `in_design`) and implementation (`ready` → `in_progress`)
  as determined by `apm start --next` internally
- [x] `apm work` does not decide which type of agent to spawn — that is
  determined by the `instructions` field in the target state config, read by
  `apm start --next`
- [x] Concurrency is capped at `agents.max_concurrent`; additional tickets are
  queued and started as slots free up
- [x] `--skip-permissions` / `-P` flag passes through to all spawned workers
- [x] `apm work` blocks until all spawned workers have exited, then prints a
  summary (ticket id, final state)
- [x] `apm work --dry-run` prints which tickets would be started without
  spawning anything
- [x] `apm work` exits non-zero if any worker ended in a state other than
  `implemented` or `specd` (the expected terminal states for agent work)

### Out of scope

- Worker monitoring UI or live log tailing
- Automatic retry of blocked or failed workers
- Any changes to `apm start` itself
- Deciding what type of agent to spawn — that is `apm start --next`'s job

### Approach

`apm work` is a thin orchestration loop around `apm start --next --spawn`.

`apm/src/cmd/work.rs`:

```
loop:
    if active_workers < max_concurrent:
        result = apm start --next --spawn [-P]
        if no ticket found: break
        track child process
    wait for any child to exit, decrement active_workers
exit when queue empty and all children done
```

Rather than re-implementing `apm start --next` logic, `work.rs` spawns the
`apm start --next --spawn` subprocess and inspects its exit code:
- exit 0, output contains worktree path: worker launched, track pid
- exit 0, output "no actionable tickets": stop dispatching new workers
- exit non-zero: print warning, continue

After all workers exit, read each ticket's final state from its branch and
print the summary.

### Amendment requests

- [x] `apm work` is the delegator loop, not just a worker launcher for `ready`
  tickets. It should call `apm start --next` in a loop (a separate ticket),
  which internally finds the next actionable ticket across all states with
  `trigger: command:start` — including `new → in_design` (spec-writing) and
  `ready → in_progress` (implementation). Update the acceptance criteria to
  reflect this broader scope.
- [x] Remove the "out of scope" exclusion of spec-writing workers. Under the
  delegator model, `apm work` does not decide what kind of agent to spawn —
  `apm start --next` does, by reading the `instructions` property from the
  target state config.
- [x] The acceptance criteria mention `ready` state specifically; replace with
  "any state where `actionable = ['agent']` and `agent` is unset and a
  `trigger: command:start` transition exists."
- [x] Update the approach section now that `apm start --next` (separate ticket)
  exists as the primitive. `apm work` is a thin loop around it.

## History

| When | From | To | By |
|------|------|----|----|
| 2026-03-28T19:42Z | — | new | claude-0328-c72b |
| 2026-03-28T19:43Z | new | specd | claude-0328-c72b |
| 2026-03-29T19:11Z | specd | ammend | claude-0329-1200-a1b2 |
| 2026-03-29T20:39Z | ammend | in_design | claude-0329-main |
| 2026-03-29T20:42Z | in_design | specd | claude-0329-main |
| 2026-03-29T21:15Z | specd | ready | claude-0329-resume |
| 2026-03-29T21:15Z | ready | in_progress | claude-0329-resume |
| 2026-03-29T21:26Z | in_progress | implemented | claude-0329-resume |
| 2026-03-29T22:35Z | implemented | accepted | apm |
| 2026-03-30T02:02Z | accepted | closed | apm-sync |