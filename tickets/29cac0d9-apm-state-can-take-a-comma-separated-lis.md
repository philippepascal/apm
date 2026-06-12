+++
id = "29cac0d9"
title = "apm state can take a comma separated list of ids"
state = "in_progress"
priority = 5
effort = 2
risk = 1
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/29cac0d9-apm-state-can-take-a-comma-separated-lis"
created_at = "2026-06-11T06:35:11.889410Z"
updated_at = "2026-06-12T02:11:35.405685Z"
+++

## Spec

### Problem

`apm state <id> <state>` currently accepts a single ticket ID. When a supervisor or agent wants to batch-transition several tickets to the same state — e.g. marking a set of groomed tickets `ready` before dispatching workers — they must invoke the command once per ticket. This creates unnecessary friction in scripts and agent workflows.

The desired behaviour is that the ID argument accepts a comma-separated list (`apm state id1,id2,id3 ready`), transitions each ticket in turn, prints a result line per ticket, and exits non-zero if any transition failed.

### Acceptance criteria

- [x] `apm state <id> <state>` with a single ID behaves identically to the current implementation (no regression).
- [x] `apm state id1,id2 <state>` transitions both tickets and prints one `id: old → new` line per ticket.
- [x] Whitespace around commas is trimmed: `apm state "id1, id2" <state>` works the same as `apm state id1,id2 <state>`.
- [x] An empty or whitespace-only id argument is a no-op: the command prints nothing and exits 0.
- [x] If one ticket in the list fails to transition, the command continues processing the remaining tickets.
- [x] Each failed transition's error is printed to stderr immediately; after all tickets are processed, the command exits non-zero with a `{n} of {m} transitions failed` summary.
- [ ] The `id` argument description in `--help` output mentions comma-separated IDs.

### Out of scope

- Parallel execution of transitions (they run sequentially).
- Accepting IDs via stdin or a file flag.
- Batch transition to different target states per ticket (all IDs share the same target state).
- Changes to `apm-core/src/state::transition` — the core function stays single-ticket.

### Approach

All changes are in the CLI layer; `apm-core` is untouched.

#### `apm/src/cmd/state.rs`

Replace the current single-ticket body with a loop:

1. Split `id_arg` on `','`, trim whitespace from each token, discard empty tokens.
2. If zero tokens result, return `Ok(())` immediately — no output, exits 0. This handles the common scripting pattern `apm state "$(apm list --format ids)" <state>` when no tickets match.
3. If exactly one token results, call `apm_core::state::transition` and handle output and errors exactly as today, propagating errors with `?`. **Do not fold this into the multi-ticket loop.** The loop uses a summary error (`"{n} of {m} transitions failed"`), which would change the error output for the single-ID path and break the "single ID behaves identically" AC. The special case exists solely to preserve the current raw anyhow error chain for single-ticket calls.
4. If multiple tokens result, iterate sequentially:
   - Call `apm_core::state::transition(root, token, new_state.clone(), no_aggressive, force)` for each.
   - On success: print `{out.id}: {out.old_state} → {out.new_state}`, then any `out.worktree_path`, `out.messages`, and `out.warnings`.
   - On error: print the error to stderr immediately and increment a failure counter; continue to the next ticket.
5. After the loop, if any transitions failed, call `anyhow::bail!("{n} of {m} transitions failed")` where `n` is the failure count and `m` is the total ticket count. This produces a clean non-zero exit without duplicating the per-ticket error text already printed in step 4.

Signature of `run` does not change — `id_arg: &str` already accepts a comma-separated string from the CLI.

#### `apm/src/main.rs`

In the `State` variant of `Command`, update the `id` argument description from:

```
Ticket ID (8-char hex, 4+ char prefix, or plain integer)
```

to:

```
Ticket ID or comma-separated list of IDs (8-char hex, 4+ char prefix, or plain integer)
```

Also add an example line to the `long_about` string:

```
apm state 42,43 ready        # transition multiple tickets at once
```

#### `apm/tests/integration.rs`

Add one test `state_batch_transition`:

1. Create two tickets with `cmd::new::run`.
2. Call `cmd::state::run(dir.path(), "id1,id2", "specd".into(), false, true)` (force=true to bypass workflow rules in test).
3. Assert both ticket branch blobs contain `state = "specd"`.

Add one test `state_empty_id_noop`:

1. Call `cmd::state::run(dir.path(), "", "specd".into(), false, false)`.
2. Assert it returns `Ok(())` and produces no output.

### Open questions


### Amendment requests

- [x] Define behaviour for an empty/whitespace-only id argument (zero tokens after split+trim). This is the primary use case: 'apm state "$(apm list --format ids)" <state>' yields an empty id arg when no tickets match, since 'apm list --format ids' prints an empty line. Add an AC that an empty/whitespace-only id list is a no-op exiting 0 (do not error). Note this changes today's single-ID behaviour, where 'apm state "" <state>' currently errors with 'no ticket found'.
- [x] Fix the error-reporting contract so the first error is not printed twice. main() returns anyhow::Result, so a returned Err is already printed by anyhow's handler (exit 1). Instead of printing each error in the loop AND returning the first error, print each per-ticket error to stderr in the loop, then return a summary via anyhow::bail!("{n} of {m} transitions failed") — preserves non-zero exit without duplicating the first error.
- [x] Clarify why the single-token path is special-cased: it exists to preserve today's single-ID error output exactly (raw anyhow chain, no '0 of 1 failed' summary), satisfying the 'single ID behaves identically' AC. State this rationale in the Approach so a worker doesn't 'simplify' the branch away and change single-ID error formatting. Alternatively, drop the branch but make the loop's N=1 error path reproduce today's output byte-for-byte.

### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-06-11T06:35Z | — | new | philippepascal |
| 2026-06-11T06:35Z | new | groomed | philippepascal |
| 2026-06-11T06:35Z | groomed | in_design | philippepascal |
| 2026-06-11T06:38Z | in_design | specd | claude |
| 2026-06-11T06:40Z | specd | ammend | philippepascal |
| 2026-06-11T06:42Z | ammend | in_design | philippepascal |
| 2026-06-11T06:45Z | in_design | specd | claude |
| 2026-06-12T02:11Z | specd | ready | philippepascal |
| 2026-06-12T02:11Z | ready | in_progress | philippepascal |