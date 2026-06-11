+++
id = "29cac0d9"
title = "apm state can take a comma separated list of ids"
state = "in_design"
priority = 5
effort = 2
risk = 1
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/29cac0d9-apm-state-can-take-a-comma-separated-lis"
created_at = "2026-06-11T06:35:11.889410Z"
updated_at = "2026-06-11T06:42:08.962983Z"
+++

## Spec

### Problem

`apm state <id> <state>` currently accepts a single ticket ID. When a supervisor or agent wants to batch-transition several tickets to the same state — e.g. marking a set of groomed tickets `ready` before dispatching workers — they must invoke the command once per ticket. This creates unnecessary friction in scripts and agent workflows.

The desired behaviour is that the ID argument accepts a comma-separated list (`apm state id1,id2,id3 ready`), transitions each ticket in turn, prints a result line per ticket, and exits non-zero if any transition failed.

### Acceptance criteria

- [ ] `apm state <id> <state>` with a single ID behaves identically to the current implementation (no regression).
- [ ] `apm state id1,id2 <state>` transitions both tickets and prints one `id: old → new` line per ticket.
- [ ] Whitespace around commas is trimmed: `apm state "id1, id2" <state>` works the same as `apm state id1,id2 <state>`.
- [ ] An empty or whitespace-only id argument is a no-op: the command prints nothing and exits 0.
- [ ] If one ticket in the list fails to transition, the command continues processing the remaining tickets.
- [ ] Each failed transition's error is printed to stderr immediately; after all tickets are processed, the command exits non-zero with a `{n} of {m} transitions failed` summary.
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
2. If exactly one token results, call `apm_core::state::transition` and handle output exactly as today (no behaviour change for the single-ID path).
3. If multiple tokens, iterate sequentially:
   - Call `apm_core::state::transition(root, token, new_state.clone(), no_aggressive, force)` for each.
   - On success: print `{out.id}: {out.old_state} → {out.new_state}`, then any `out.worktree_path`, `out.messages`, and `out.warnings`.
   - On error: push the error into a local `Vec<anyhow::Error>` and continue.
4. After the loop, if the error vec is non-empty, print each error to stderr and return the first error via `Err(...)`.

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

### Open questions


### Amendment requests

- [ ] Define behaviour for an empty/whitespace-only id argument (zero tokens after split+trim). This is the primary use case: 'apm state "$(apm list --format ids)" <state>' yields an empty id arg when no tickets match, since 'apm list --format ids' prints an empty line. Add an AC that an empty/whitespace-only id list is a no-op exiting 0 (do not error). Note this changes today's single-ID behaviour, where 'apm state "" <state>' currently errors with 'no ticket found'.
- [ ] Fix the error-reporting contract so the first error is not printed twice. main() returns anyhow::Result, so a returned Err is already printed by anyhow's handler (exit 1). Instead of printing each error in the loop AND returning the first error, print each per-ticket error to stderr in the loop, then return a summary via anyhow::bail!("{n} of {m} transitions failed") — preserves non-zero exit without duplicating the first error.
- [ ] Clarify why the single-token path is special-cased: it exists to preserve today's single-ID error output exactly (raw anyhow chain, no '0 of 1 failed' summary), satisfying the 'single ID behaves identically' AC. State this rationale in the Approach so a worker doesn't 'simplify' the branch away and change single-ID error formatting. Alternatively, drop the branch but make the loop's N=1 error path reproduce today's output byte-for-byte.

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