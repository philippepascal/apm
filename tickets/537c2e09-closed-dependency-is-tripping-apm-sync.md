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

- [ ] `apm sync` (and other mutating commands, e.g. `apm state`) do not print `dep <id> not found` and do not exit with "Mutating commands are blocked" when a non-closed ticket's `depends_on` references a ticket that is closed and whose `ticket/*` branch has already been deleted, but whose file still exists under `tickets/` on the default branch
- [ ] `apm validate` reports no `depends_on` issue for the same scenario
- [ ] `apm new --depends-on <id>` succeeds when `<id>` refers to a closed ticket whose branch has been deleted
- [ ] `depends_on` referencing an id that does not exist anywhere — no live branch and no file on the default branch — still produces a `dep <id> not found` error from all three entry points above
- [ ] `cargo test --workspace` passes, including a new regression test that reproduces the original bug (closed dependency, branch deleted, dependent ticket non-closed) and asserts the affected command no longer fails

### Out of scope

- `apm_core::context::build_dependency_bundle` (the worker-prompt dependency context builder) has the same shaped gap — it will print `*Ticket not found.*` for a closed, branchless dependency — but this is cosmetic prompt content, not a blocking failure. Leave it as-is; file a follow-up ticket if it turns out to matter.
- Changing what `apm clean` does with closed tickets' branches (it will keep deleting them).
- Any change to `apm archive` / `archive_dir` behaviour.
- Redesigning `hash_trip`'s stamp/re-validation mechanism itself (e.g. writing a stamp on failure) — the fix here is to stop the false positive at its source, not to change how failures are cached.

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