+++
id = "1f2ff09d"
title = "Clean up sync_guidance consistency nits"
state = "groomed"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/1f2ff09d-clean-up-sync-guidance-consistency-nits"
created_at = "2026-04-17T20:07:27.286560Z"
updated_at = "2026-04-17T20:09:12.356579Z"
epic = "47375a6a"
target_branch = "epic/47375a6a-safer-apm-sync"
depends_on = ["a087593c", "1339c81d", "5cf54181"]
+++

## Spec

### Problem

Three small consistency issues in the sync module landed with epic `47375a6a` (tickets `5cf54181`, `a087593c`, `1339c81d`). None is a correctness bug, but each rubs against the "single source of guidance wording" policy that ticket `5cf54181` established.

**1. Stale `TODO(5cf54181)` comments.** Four call sites in `apm-core/src/git_util.rs` carry a `// TODO(5cf54181): move to sync_guidance` comment immediately above a line that already does exactly that:

- `sync_default_branch` — Behind arm (uses `crate::sync_guidance::MAIN_BEHIND_DIRTY_OVERLAP`)
- `sync_default_branch` — Diverged arm (uses `MAIN_DIVERGED_DIRTY` / `MAIN_DIVERGED_CLEAN`)
- `sync_non_checked_out_refs` — Ahead arm (TODO, but this one is actually real — see item 2)
- `sync_non_checked_out_refs` — Diverged arm (uses `TICKET_OR_EPIC_DIVERGED`)

Three of the four are leftover scaffolding from pre-wiring the guidance module and should simply be deleted. The fourth (Ahead) is a real TODO resolved by item 2 below.

**2. Ahead info line is inlined, not in `sync_guidance`.** Two call sites emit an "X is ahead of origin by N commits" message via `format!(...)` directly:

- `sync_default_branch` — Ahead arm (`format!("{default} is ahead of {remote} by {count} commit{} — run `git push` when ready", ...)`)
- `sync_non_checked_out_refs` — Ahead arm (`format!("info: {branch} is ahead of origin — push when ready: git push origin {branch}")`)

The wording also diverges between the two sites. `sync_guidance.rs` has no `MAIN_AHEAD` or `TICKET_OR_EPIC_AHEAD` constant. Per the module's charter ("never scatter literal guidance strings through the sync flow — always reference a named constant"), both call sites should consume new constants from `sync_guidance.rs`.

**3. `sync_default_branch`'s Behind→FF fallback prints `MAIN_BEHIND_DIRTY_OVERLAP` on *any* `git merge --ff-only` failure, not only dirty-overlap.** In practice overlap is the only realistic failure mode for a strictly-behind FF, but the code assumes it. A one-line comment documenting the assumption is sufficient — rewriting the error handling to distinguish causes is not worth the complexity.

None of these affect behavior or test outcomes. They are code-hygiene fixes that keep the policy established by `5cf54181` honest.

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
| 2026-04-17T20:07Z | — | new | philippepascal |
| 2026-04-17T20:09Z | new | groomed | apm |
