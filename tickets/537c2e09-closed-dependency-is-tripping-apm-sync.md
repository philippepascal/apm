+++
id = "537c2e09"
title = "closed dependency is tripping apm sync"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/537c2e09-closed-dependency-is-tripping-apm-sync"
created_at = "2026-08-28T00:48:10.874316Z"
updated_at = "2026-08-28T07:19:26.968328Z"
+++

## Spec

### Problem

`apm sync` (and every other mutating `apm` command) can get permanently blocked when a
non-closed ticket's `depends_on` points at a ticket that is itself closed *and* whose
`ticket/*` branch has already been deleted (e.g. by `apm clean`). The reported symptom:

```
apm sync
  #cfd8d425: dep 8f85c68c not found
error: config has changed and validation is failing.
Mutating commands are blocked. Run apm validate to fix.
```

even though ticket `8f85c68c` is closed, clean, and its file is still present under
`tickets/`.

Root cause: `apm_core::ticket::load_all_from_git` discovers tickets exclusively by
enumerating live `ticket/*` branches — it never looks at the `tickets/` directory on the
default branch. `ticket::close` merges a closed ticket's file into the default branch
before `apm clean` deletes its branch, so the file legitimately survives there, but once
the branch is gone `load_all_from_git` can no longer see the ticket at all.

`apm/src/hash_trip.rs` runs a validation preflight (`validate_depends_on`, built on
`check_depends_on_rules`) before every non-exempt, non-read-only command, and it loads
its ticket list with a bare `load_all_from_git`. When a dependency has gone
branchless this way, `check_depends_on_rules` fails with `dep {id} not found`, which
`hash_trip::run` turns into `HashTripOutcome::Failed`. Because a failed hash-trip check
never writes the validation stamp, every subsequent mutating command (`apm sync`
included) re-runs the same check and fails again — the ticket queue is stuck until a
human intervenes, and the very command that should reconcile this state (`apm sync`)
can't run.

`apm/src/cmd/new.rs` has the identical bug: `apm new --depends-on <id>` loads tickets
via the same bare `load_all_from_git` before calling `check_depends_on_rules`, so
declaring a dependency on a closed-and-cleaned ticket at creation time fails the same
way.

Note that `apm/src/ctx.rs`'s `CmdContext::load` (used by `apm set`, `apm validate`, and
others) already works around this by merging in tickets found via
`apm_core::ticket::load_from_default_branch` — that fix just isn't applied everywhere
`check_depends_on_rules` / `validate_depends_on` is used.

### Acceptance criteria

Checkboxes; each one independently testable.

### Out of scope

Explicit list of what this ticket does not cover.

### Approach

How the implementation will work.

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-08-28T00:48Z | — | new | philippepascal |
| 2026-08-28T07:13Z | new | groomed | philippepascal |
| 2026-08-28T07:19Z | groomed | in_design | philippepascal |