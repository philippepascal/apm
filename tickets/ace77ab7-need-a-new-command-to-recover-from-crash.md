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

- [ ] `apm workers recover <id>` on a ticket whose worker PID is dead and whose ticket state is not terminal/`worker_end` (i.e. shown as `crashed` by `apm workers`) transitions the ticket back to the state it was in immediately before the crashed worker started, and removes the stale `.apm-worker.pid` file
- [ ] `apm workers recover <id>` on a ticket whose worker PID is still alive fails with an error telling the caller to run `apm workers --kill <id>` first, and makes no changes to ticket state or files
- [ ] `apm workers recover <id>` on a ticket with no `.apm-worker.pid` file fails with a "nothing to recover" error and makes no changes
- [ ] `apm workers recover <id>` on a ticket whose current state has more than one possible predecessor (e.g. `in_design`, reachable from both `groomed` and `amend`) resolves the correct one from the ticket's `## History` table rather than guessing, and falls back to requiring an explicit `--to <state>` when History does not disambiguate
- [ ] `apm workers recover --all` recovers every ticket currently shown as `crashed` by `apm workers`, continues past individual failures, prints a per-ticket result line, and exits non-zero if any ticket failed
- [ ] `apm workers recover <id> --dry-run` prints the target state and the pid file that would be removed without changing ticket state or touching any files
- [ ] After a successful `apm workers recover <id>`, that ticket no longer appears as `crashed` in `apm workers`
- [ ] `apm workers recover <id>` prints a warning (but still proceeds) when the ticket's worktree has uncommitted changes, and never discards or modifies those files

### Out of scope

- Cleaning or discarding uncommitted/untracked files left in a crashed worker's worktree — that remains a manual step (or, once the ticket reaches a terminal state, `apm clean`)
- Automatically re-dispatching a new worker after recovery — `apm start`/`apm work` will pick the recovered ticket up on its next normal run
- Recovering from `merge_failed` — that state already has its own retry path (`merge_failed → implemented`)
- Any apm-server / web UI surface for this command — CLI-only, consistent with `apm workers diag`, which also has no web equivalent
- Changing how `apm workers` itself detects or labels a worker as `crashed` — that logic already exists and is reused as-is, not modified

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