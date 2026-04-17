+++
id = "1f2ff09d"
title = "Clean up sync_guidance consistency nits"
state = "in_design"
priority = 0
effort = 2
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/1f2ff09d-clean-up-sync-guidance-consistency-nits"
created_at = "2026-04-17T20:07:27.286560Z"
updated_at = "2026-04-17T20:12:24.056305Z"
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

- [ ] `sync_guidance.rs` exports a `MAIN_AHEAD` pub const whose text includes the `<default>`, `<remote>`, `<count>`, and `<commits>` placeholders
- [ ] `sync_guidance.rs` exports a `TICKET_OR_EPIC_AHEAD` pub const whose text includes the `<slug>` placeholder
- [ ] `sync_default_branch` Ahead arm emits its message by substituting into `sync_guidance::MAIN_AHEAD` rather than via an inline `format!(...)`
- [ ] `sync_non_checked_out_refs` Ahead arm emits its message by substituting into `sync_guidance::TICKET_OR_EPIC_AHEAD` rather than via an inline `format!(...)`
- [ ] All four `// TODO(5cf54181): move to sync_guidance` comments are absent from `git_util.rs`
- [ ] `sync_default_branch` Behind arm carries a one-line comment documenting the assumption that any `git merge --ff-only` failure implies dirty overlap
- [ ] `cargo test` passes with no new failures

### Out of scope

- Rewriting the Behind arm error handling to distinguish dirty-overlap from other FF failure causes\n- Changes to any existing guidance constant wording\n- New tests (the guidance strings are not exercised by the current unit test suite)

### Approach

Two files change: `apm-core/src/sync_guidance.rs` (add two constants) and `apm-core/src/git_util.rs` (wire the constants, remove dead TODOs, add one comment).

**`apm-core/src/sync_guidance.rs`**

1. Update the module-level doc comment Placeholders list to include `<count>` (number of commits) and `<commits>` (the word "commit" or "commits", caller supplies).

2. Add `MAIN_AHEAD` after the existing `MAIN_DIVERGED_DIRTY` constant. Wording matches the existing inline message; the only change is extracting it to a named constant. Placeholders: `<default>`, `<remote>`, `<count>`, `<commits>`.

3. Add `TICKET_OR_EPIC_AHEAD` after `TICKET_OR_EPIC_DIVERGED`. Wording mirrors the existing inline string in the Ahead arm of `sync_non_checked_out_refs`. Placeholder: `<slug>`.

**`apm-core/src/git_util.rs`**

4. `sync_default_branch` Behind arm: delete the stale `// TODO(5cf54181): move to sync_guidance` comment. Add in its place a one-liner: `// Assumption: overlap is the only realistic failure mode for a strictly-behind FF merge; MAIN_BEHIND_DIRTY_OVERLAP covers any --ff-only error here.`

5. `sync_default_branch` Ahead arm: replace the inline `format!(...)` with chained `.replace()` calls on `crate::sync_guidance::MAIN_AHEAD`, substituting `<default>`, `<remote>`, `<count>` (count as string), and `<commits>` ("commit" when count == 1, "commits" otherwise).

6. `sync_default_branch` Diverged arm: delete the stale `// TODO(5cf54181): move to sync_guidance` comment.

7. `sync_non_checked_out_refs` Ahead arm: delete the `// TODO(5cf54181): move to sync_guidance` comment. Replace the inline `format!(...)` with `crate::sync_guidance::TICKET_OR_EPIC_AHEAD.replace("<slug>", &branch)`.

8. `sync_non_checked_out_refs` Diverged arm: delete the stale `// TODO(5cf54181): move to sync_guidance` comment.

No test changes needed. All changes are purely textual restructuring; no observable behaviour differs.

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-04-17T20:07Z | — | new | philippepascal |
| 2026-04-17T20:09Z | new | groomed | apm |
| 2026-04-17T20:09Z | groomed | in_design | philippepascal |