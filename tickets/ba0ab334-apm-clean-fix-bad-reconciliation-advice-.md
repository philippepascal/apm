+++
id = "ba0ab334"
title = "apm clean: fix bad reconciliation advice for state mismatch"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
agent = "51664"
branch = "ticket/ba0ab334-apm-clean-fix-bad-reconciliation-advice-"
created_at = "2026-03-30T19:59:52.991650Z"
updated_at = "2026-03-30T20:00:41.112157Z"
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


### Out of scope

Explicit list of what this ticket does not cover.

### Approach

How the implementation will work.

### Open questions



### Amendment requests



## History

| When | From | To | By |
|------|------|----|----|
| 2026-03-30T19:59Z | — | new | philippepascal |
| 2026-03-30T20:00Z | new | in_design | philippepascal |