+++
id = "537c2e09"
title = "closed dependency is tripping apm sync"
state = "in_design"
priority = 0
effort = 2
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/537c2e09-closed-dependency-is-tripping-apm-sync"
created_at = "2026-08-28T00:48:10.874316Z"
updated_at = "2026-08-28T07:23:48.136075Z"
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

#### 1. Add a shared branchless-merge helper in `apm-core`

In `apm-core/src/ticket/ticket_util.rs`, add a function next to `load_all_from_git` /
`load_from_default_branch`:

```rust
/// Merge `tickets` (typically from `load_all_from_git`) with any tickets present
/// in `tickets_dir_rel` on `default_branch` but not already in the list — i.e.
/// tickets whose `ticket/*` branch no longer exists (e.g. a closed ticket after
/// `apm clean` deletes its branch) but whose file survives on the default branch
/// because `ticket::close` merges it there first. Existing entries win; the
/// result is re-sorted by `created_at`.
pub fn merge_branchless(
    root: &Path,
    tickets_dir_rel: &Path,
    default_branch: &str,
    mut tickets: Vec<Ticket>,
) -> Vec<Ticket> {
    let branchless = load_from_default_branch(root, tickets_dir_rel, default_branch)
        .unwrap_or_default();
    if !branchless.is_empty() {
        let seen: HashSet<String> = tickets.iter().map(|t| t.frontmatter.id.clone()).collect();
        for t in branchless {
            if !seen.contains(&t.frontmatter.id) {
                tickets.push(t);
            }
        }
        tickets.sort_by_key(|t| t.frontmatter.created_at);
    }
    tickets
}
```

This is a straight extraction of the merge logic that already lives inline in
`apm/src/ctx.rs::CmdContext::load` — no behaviour change there, just centralizing it so
the other two broken call sites can reuse it.

#### 2. Use it in `apm/src/ctx.rs`

Replace the inline branchless-merge block in `CmdContext::load` (the `let branchless =
...` through the `tickets.sort_by_key(...)` at the end of the `if !branchless.is_empty()`
block) with:

```rust
let tickets = apm_core::ticket::merge_branchless(
    root,
    &config.tickets.dir,
    &config.project.default_branch,
    tickets,
);
```

Keep everything else in `load` (the `aggressive` branch selecting
`load_all_from_git_classified` vs `load_all_from_git`) unchanged.

#### 3. Fix `apm/src/hash_trip.rs` (the actual `apm sync` blocker)

In `run()`, after loading `tickets` via `load_all_from_git` (line ~50-51), wrap the
result with the same helper before it's handed to `validate_config`/`validate_depends_on`:

```rust
let tickets = apm_core::ticket::merge_branchless(
    root,
    &config.tickets.dir,
    &config.project.default_branch,
    apm_core::ticket::load_all_from_git(root, &config.tickets.dir).unwrap_or_default(),
);
```

`hash_trip::run` doesn't fetch and shouldn't start doing so — reuse `load_all_from_git`
(not the classified/aggressive variant) here, same as today, just with the branchless
merge layered on top.

#### 4. Fix `apm/src/cmd/new.rs`

At line 66, wrap the same way before calling `check_depends_on_rules`:

```rust
let all_tickets = apm_core::ticket::merge_branchless(
    root,
    &config.tickets.dir,
    &config.project.default_branch,
    apm_core::ticket::load_all_from_git(root, &config.tickets.dir)?,
);
```

(`apm/src/cmd/set.rs` already goes through `CmdContext::load`, so it's fixed for free by
step 2 — no separate change needed there.)

#### 5. Tests

- `apm-core`: unit test for `merge_branchless` in `ticket_util.rs` — build a `tickets`
  vec loaded the normal way plus a ticket only reachable via a fake default-branch file
  read, and assert the branchless one is appended and dedup works when both a branch and
  a file exist for the same id (branch version wins, no duplicate).
- `apm-core/src/validate.rs` unit test (alongside the existing
  `validate_depends_on_closed_ticket_skipped` etc.): construct a dependent ticket
  (non-closed) whose `depends_on` references an id that is present only in a second
  `Ticket` list entry marked `state = "closed"` — confirm `validate_depends_on` on the
  merged list produces no violation, and confirm it still errors when the dep is absent
  from the list entirely (unaffected case, guards against a no-op fix).
- `apm/tests/e2e.rs`: since `hash_trip` is a private module of the `apm` binary (`mod
  hash_trip;` in `main.rs`, not re-exported via `apm/src/lib.rs`), reproduce the bug by
  spawning the real binary (the file already does this via `CARGO_BIN_EXE_apm`, see the
  `Command::new(APM)` calls). Sequence: `apm new` two tickets A and B; set B
  `depends_on = A`; close A and delete A's local `ticket/*` branch (via `git branch -D`,
  mirroring what `apm clean --remove-branches` does) while leaving A's file in `tickets/`
  on the default branch; then run `apm sync` (or any other mutating command) and assert
  exit code 0 with no "not found" / "Mutating commands are blocked" output. Also assert
  the true-orphan case (dep id with no branch and no file anywhere) still fails as today.

Run `cargo test --workspace` before finishing.

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-08-28T00:48Z | — | new | philippepascal |
| 2026-08-28T07:13Z | new | groomed | philippepascal |
| 2026-08-28T07:19Z | groomed | in_design | philippepascal |