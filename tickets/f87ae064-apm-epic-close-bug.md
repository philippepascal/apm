+++
id = "f87ae064"
title = "apm epic close bug"
state = "in_progress"
priority = 0
effort = 2
risk = 1
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/f87ae064-apm-epic-close-bug"
created_at = "2026-06-05T01:34:06.624276Z"
updated_at = "2026-06-12T23:03:18.480438Z"
+++

## Spec

### Problem

`apm epic close` guards against two unsafe conditions: an active worker process on a ticket in the epic, and an epic branch whose commits have not yet landed in the default branch. It does not check whether the epic's tickets are still in a non-terminal state.

When `apm epic list` shows "implemented" for an epic, every ticket has reached a state with `satisfies_deps = true` but one or more tickets have not yet transitioned to a terminal state (e.g. they remain in `implemented` rather than `closed`). Closing the epic in this condition deletes the branch while those tickets are left stranded in a non-terminal state — no pointer to the work remains, making them difficult to reason about or close afterwards.

### Acceptance criteria

- [x] `apm epic close <id>` fails when the derived epic state is "implemented", printing an error that lists each non-terminal ticket by id, title, and current state
- [ ] The error message tells the user to close the listed tickets first and notes that `--force` bypasses the check
- [ ] `apm epic close <id> --force` proceeds when the derived state is "implemented" (consistent with how `--force` bypasses the existing guards)
- [ ] `apm epic close <id>` succeeds without the new error when the derived epic state is "done" (all tickets terminal)
- [ ] `apm epic close <id>` succeeds without the new error when the derived epic state is "empty" (no tickets in the epic)

### Out of scope

- Changing behaviour for "in_progress" epics beyond what existing live-worker and unmerged-branch guards already enforce
- Auto-closing non-terminal tickets as part of `apm epic close`
- Adding the same guard to `apm epic submit`

### Approach

#### `apm/src/cmd/epic.rs` — `run_close`

`run_close` has an `if !force { ... }` block that loads `all_tickets` for the live-worker check. Append a second guard at the end of that same block (before it closes):

1. Filter `all_tickets` to tickets whose `epic` field equals `epic_id`.
2. Look up each ticket's `StateConfig` by matching `t.frontmatter.state` against `config.workflow.states`.
3. Call `apm_core::epic::derive_epic_state(&state_configs)` — this function is already public and handles the "implemented" / "done" / "in_progress" / "empty" distinction.
4. If the result is `"implemented"`, collect non-terminal tickets (those not in `config.terminal_state_ids()`), format each as `"  <id> — <title> (<state>)"`, and bail:

```
epic is in state 'implemented'; close these tickets first:
  <id> — <title> (<state>)
  ...
Use --force to close unconditionally.
```

No new `apm-core` function is required — `derive_epic_state` is already public and well-tested.

#### `apm/tests/integration.rs` — new test `epic_close_blocks_on_implemented_state`

1. `init_repo()` for an isolated temp repo.
2. `run_apm(p, &["epic", "new", "close guard test"])` — parse `epic_branch` from stdout; derive `epic_id` via `apm_core::epic::epic_id_from_branch`.
3. `create_ticket(p, "guard-ticket")` — get `(ticket_id, ticket_branch)`.
4. Build `ticket_path` with `ticket_rel_path(&ticket_branch)`.
5. Read ticket content via `branch_content`, replace `state = "new"` with `state = "implemented"` and append `epic = "<epic_id>"` on the next line, then commit the result directly to the ticket branch (`git checkout <ticket_branch>`, write, `git add`, `git commit`, `git checkout main`).
6. Assert `apm epic close <epic_id>` exits non-zero and its stderr contains `"implemented"`.
7. Assert `apm epic close <epic_id> --force` exits zero (bypasses all guards, including the merge-status check).

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-06-05T01:34Z | — | new | philippepascal |
| 2026-06-12T07:52Z | new | groomed | philippepascal |
| 2026-06-12T07:52Z | groomed | in_design | philippepascal |
| 2026-06-12T07:57Z | in_design | specd | claude |
| 2026-06-12T22:52Z | specd | ready | philippepascal |
| 2026-06-12T23:03Z | ready | in_progress | philippepascal |