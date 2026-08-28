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

Add `apm workers recover` as a new subcommand next to the existing
`apm workers diag <id>` (see `WorkersCommand` in `apm/src/main.rs`, and
`run_diag` / `list` / `kill` in `apm/src/cmd/workers.rs`, which already
contains the crash-detection logic this command builds on).

#### CLI surface

In `apm/src/main.rs`, extend `WorkersCommand`:

```
Recover {
    id: Option<String>,       // required unless --all
    #[arg(long)] all: bool,
    #[arg(long)] dry_run: bool,
    #[arg(long)] to: Option<String>,  // explicit target state override
}
```

Wire it into `Command::Workers { .. }` dispatch alongside `Diag`.

#### Shared crash detection

Factor the crashed-worker scan currently inlined in
`apm/src/cmd/workers.rs::list()` (walk `worktree::list_ticket_worktrees`,
read `.apm-worker.pid`, check `worker::is_alive`, compare ticket state
against `ended_states` = states with `terminal || worker_end`) into a
helper, e.g. `crashed_workers(root, &config) -> Result<Vec<CrashedWorker>>`
with `CrashedWorker { ticket_id, state, pid_path, worktree }`. `list()` and
the new `recover --all` both call it — no duplicated scan logic.

#### Target-state resolution

For a single `id`, resolve the ticket via `worktree_for_ticket` (already in
`apm/src/util.rs`). Determine the recovery target state, in order:

1. If `--to <state>` was given, validate it's a real, non-terminal state in
   `config.workflow.states` and use it directly.
2. Otherwise parse the ticket body's `## History` markdown table (simple
   line-based split on `|`, skip the header/separator rows) and find the
   *last* row whose `To` column equals the ticket's current state; use that
   row's `From` column. This correctly distinguishes `groomed → in_design`
   from `amend → in_design`, and `ready → in_progress` from
   `fix → in_progress`.
3. If History has no matching row, fall back to config: collect every
   state with a `command:start` transition whose `to` equals the current
   state. If exactly one exists, use it.
4. If none of the above resolves a single target, bail with an error
   listing the candidate states and instructing the caller to pass
   `--to <state>` explicitly.

#### Recovery action

For each ticket to recover:

- If `.apm-worker.pid` is missing → bail "nothing to recover" (AC 3).
- If the PID is alive → bail, pointing at `apm workers --kill <id>` (AC 2).
- Check worktree cleanliness with `git_util::is_worktree_dirty` (already
  used in `clean.rs`); if dirty, print a warning but continue (AC 8) —
  mirror the wording of the existing `in_progress → ready` transition
  warning.
- `--dry-run`: print `id`, current state, resolved target state, and the
  pid file path that would be removed; do not call `state::transition` or
  touch any files.
- Otherwise call `apm_core::state::transition(root, id, target_state,
  false, /*force=*/ true)`. `force = true` is required because the
  rollback direction (e.g. `in_design → groomed`) is not a declared
  forward transition in `workflow.toml`; force already exists precisely to
  allow this kind of supervisor-directed correction and this command is
  the safe, guarded wrapper around it.
- On success, remove `.apm-worker.pid` from the ticket's worktree. Leave
  `.apm-worker.log` / `.apm-worker.summary.json` in place — they're useful
  for a postmortem via `apm workers diag <id>` and are already excluded
  from `apm clean`'s untracked-file sweep only once the ticket reaches a
  terminal state.

#### Batch mode (`--all`)

Call `crashed_workers()`, run the single-ticket recovery routine for each,
catch and report per-ticket errors without aborting the loop (mirror the
existing multi-id pattern in `apm/src/cmd/state.rs::run`), print one result
line per ticket, and exit non-zero if any ticket failed.

#### Tests

- `apm-core`: unit tests for the target-state resolver — History match,
  History/config disambiguation (groomed vs amend), config-fallback single
  candidate, ambiguous-with-no-`--to` error.
- `apm/tests/integration.rs`: end-to-end cases in a temp git repo —
  crashed `in_progress` ticket recovers to `ready` and its pid file is
  gone; crashed `in_design` ticket recovers to the correct History
  predecessor; live-PID ticket refuses; missing-pid-file ticket refuses;
  `--all` recovers two crashed tickets and reports a failure on a third
  without stopping; `--dry-run` changes nothing; recovered ticket no
  longer shows as `crashed` in `apm workers`.

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-08-28T00:51Z | — | new | philippepascal |
| 2026-08-28T07:13Z | new | groomed | philippepascal |
| 2026-08-28T07:26Z | groomed | in_design | philippepascal |