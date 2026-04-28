+++
id = "50649e84"
title = "Merge apm verify into apm validate; remove verify command"
state = "new"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/50649e84-merge-apm-verify-into-apm-validate-remov"
created_at = "2026-04-28T19:19:47.773209Z"
updated_at = "2026-04-28T19:19:47.773209Z"
+++

## Spec

### Problem

`apm verify` and `apm validate` are two commands with overlapping scope and near-synonymous names that consistently confuse users.

**Today's split** (logical vs filesystem correctness):

- `apm validate` (`apm-core/src/validate.rs`, `apm/src/cmd/validate.rs`) — config parses, state transitions reference known states, ticket branch-field matches actual branch name, no two tickets share the same branch, dependency rules across tickets (per active completion strategy). Flags: `--fix`, `--json`, `--config-only`. Refreshes the hash-trip stamp on success.
- `apm verify` (`apm-core/src/verify.rs`, `apm/src/cmd/verify.rs`) — unknown ticket states, ID/filename mismatches, missing branches on active tickets, merged-but-open branches, missing spec/history sections, frontmatter that does not match branch state, missing worktree for `in_design`/`in_progress` tickets. Flags: `--fix`, `--no-aggressive`.

**Why this is not worth keeping**:

1. Users who run either are usually troubleshooting and want the complete picture; the split forces them to know to run both.
2. There is no real cost difference at ticket scale — verify's filesystem checks are cheap. No "fast vs thorough" variant is justified.
3. The hash-trip on config/workflow change (`apm/src/hash_trip.rs`) runs validate but not verify, so config drift can leave dangling worktrees and we do not catch it. Merging closes this gap.

**What this ticket should change**:

Merge all of verify's checks into `apm validate`. Remove `apm verify` entirely — no deprecation alias. `apm validate` becomes the single integrity command.

Concrete edits:

- Move every check from `apm-core/src/verify.rs` into `apm-core/src/validate.rs`; delete `verify.rs`.
- Move CLI handler logic from `apm/src/cmd/verify.rs` into `apm/src/cmd/validate.rs`; delete `cmd/verify.rs`.
- Remove the `Verify` variant from the `Command` enum in `apm/src/main.rs`. Remove the dispatch arm.
- Update help text and `long_about` for `apm validate` to describe the combined check surface.
- Remove `Command::Verify` from `is_read_only_command` in `apm/src/hash_trip.rs` (added by ticket 6cf21715 / amendment to b10d957a). It is moot now that validate is exempt and subsumes verify.
- Combined `--fix` handles both classes of repair (validate's branch-field repairs and verify's filesystem repairs). Verify's `--fix` semantic of NOT auto-recreating missing worktrees (per ticket 6cf21715) must be preserved — the merged command reports the issue and exits non-zero, never silently re-provisions.
- `--config-only` continues to skip both ticket-integrity and filesystem checks (the union of the things `--config-only` previously skipped).
- Move/rename verify tests: `apm-core/tests/verify.rs` becomes part of validate's test surface, or merges into `apm-core/src/validate.rs`'s `#[cfg(test)]` block.
- Update `docs/commands.md`: remove the `apm verify` section, expand the `apm validate` section to describe the full check list.
- Update `README.md` if it references `apm verify` anywhere (search before editing).
- After this lands, the hash-trip automatically catches the gap that previously required a separate `apm verify` invocation — config or workflow change triggers the merged validate, which now also catches dangling worktrees and frontmatter drift.

**Out of scope**:

- Adding any new check beyond what validate and verify already do today.
- Renaming `apm validate` to anything else.
- Keeping `apm verify` as a deprecation alias — the user has explicitly opted out of the deprecation period.

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
| 2026-04-28T19:19Z | — | new | philippepascal |
