+++
id = "ba0ab334"
title = "apm clean: fix bad reconciliation advice for state mismatch"
state = "closed"
priority = 0
effort = 1
risk = 1
author = "philippepascal"
agent = "30043"
branch = "ticket/ba0ab334-apm-clean-fix-bad-reconciliation-advice-"
created_at = "2026-03-30T19:59:52.991650Z"
updated_at = "2026-03-31T05:05:22.246702Z"
+++

## Spec

### Problem

When `apm clean` detects a state mismatch — the ticket branch says one state but `main` has a different state — it prints:

```
warning: ticket/0049-... state mismatch — branch=closed main=in_progress — run `apm close 0049` to reconcile
```

This advice is wrong. If the branch already says `closed`, running `apm close` will fail with "ticket is already closed". The mismatch means the ticket was transitioned on its branch but that change was never propagated to `main` — typically because the branch was closed/merged without `apm sync` running.

The correct reconciliation is `apm sync`, not `apm close`. `apm sync` detects this exact case (ticket in terminal state on its branch, different state on main) and updates main accordingly.

The warning message should be corrected to suggest `apm sync` instead of `apm close`.

### Acceptance criteria

- [x] `apm clean` state-mismatch warning suggests `apm sync` instead of `apm close <id>`
- [x] The warning message still identifies the branch, the branch state, and the main state

### Out of scope

The `None` case (ticket not found on main at all) is a separate, distinct scenario not covered by this ticket.

### Approach

In `apm-core/src/clean.rs`, change the warning message at the `Some(ms) if ms != branch_state` arm (line ~149) from:

```
run `apm close {id}` to reconcile
```

to:

```
run `apm sync` to reconcile
```

The `None` arm (ticket not found on main) keeps its existing message unchanged — that case is different and may warrant a separate decision.

Update the integration test `clean_skips_state_mismatch_between_branch_and_main` to assert the new message text appears in stderr.

### Open questions



### Amendment requests



## History

| When | From | To | By |
|------|------|----|----|
| 2026-03-30T19:59Z | — | new | philippepascal |
| 2026-03-30T20:00Z | new | in_design | philippepascal |
| 2026-03-30T20:02Z | in_design | specd | claude-0330-2005-b4f2 |
| 2026-03-30T20:10Z | specd | ready | apm |
| 2026-03-30T20:11Z | ready | in_progress | philippepascal |
| 2026-03-30T20:14Z | in_progress | implemented | claude-0330-2011-7428 |
| 2026-03-30T20:31Z | implemented | accepted | apm-sync |
| 2026-03-31T05:05Z | accepted | closed | apm-sync |