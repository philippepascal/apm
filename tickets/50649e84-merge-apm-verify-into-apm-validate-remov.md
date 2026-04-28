+++
id = "50649e84"
title = "Merge apm verify into apm validate; remove verify command"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/50649e84-merge-apm-verify-into-apm-validate-remov"
created_at = "2026-04-28T19:19:47.773209Z"
updated_at = "2026-04-28T19:33:30.069332Z"
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

- [ ] `apm verify` exits with an unrecognized-command error; no deprecation alias exists
- [ ] `apm validate` reports unknown ticket state values (state not in `config.workflow.states`)
- [ ] `apm validate` reports tickets whose filename numeric prefix does not match the frontmatter `id` field
- [ ] `apm validate` reports `in_progress` and `implemented` tickets that have no `branch` field
- [ ] `apm validate` reports tickets whose branch is already merged into the default branch but whose state is not closed
- [ ] `apm validate` reports tickets in `in_design` or `in_progress` states whose worktree directory is absent from disk
- [ ] `apm validate` reports tickets missing a `## Spec` section
- [ ] `apm validate` reports tickets missing a `## History` section
- [ ] `apm validate` still reports all previously-existing check categories: config parse errors, invalid state-transition targets, branch-field mismatches, duplicate branch assignments, and dependency-rule violations
- [ ] `apm validate --fix` auto-closes tickets whose branch is already merged (calls `ticket::close`); it does NOT recreate missing worktrees
- [ ] `apm validate --fix` continues to repair branch-field mismatches (existing behaviour)
- [ ] `apm validate --config-only` skips all per-ticket and filesystem checks, including the merged-branch and worktree checks brought over from verify
- [ ] `apm validate --json` includes issues from the full merged check set in its structured output
- [ ] A config or workflow change that triggers the hash-trip now catches dangling worktrees and frontmatter drift without a separate `apm verify` invocation
- [ ] `docs/commands.md` has no `apm verify` section; the `apm validate` section lists the complete merged check set
- [ ] `README.md` contains no reference to `apm verify`
- [ ] All three tests previously in `apm-core/tests/verify.rs` pass under the validate test surface

### Out of scope

- Adding any check that neither validate nor verify performed before\n- Renaming `apm validate` to a different command name\n- A deprecation alias for `apm verify` (explicitly opted out)\n- Changing hash-trip caching logic beyond removing `Command::Verify` from `is_read_only_command`\n- Changes to `--no-aggressive` flag behaviour\n- Performance optimisation of the merged check set

### Approach

Perform changes in the order below; each step can be compiled independently before moving to the next.

**Step 1 — Move verify logic into `apm-core/src/validate.rs`**

Copy the `verify_tickets()` function from `apm-core/src/verify.rs` verbatim into `apm-core/src/validate.rs` as a new `pub fn verify_tickets(root, config, tickets, merged) -> Vec<String>`. No rename needed — it becomes part of the validate public surface. Delete `apm-core/src/verify.rs`.

Remove `pub mod verify;` from `apm-core/src/lib.rs`.

**Step 2 — Migrate verify tests**

Move the three tests from `apm-core/tests/verify.rs` into the `#[cfg(test)]` block at the bottom of `apm-core/src/validate.rs`. Update the import: `use apm_core::verify::verify_tickets` → `use super::verify_tickets`. Delete `apm-core/tests/verify.rs`.

**Step 3 — Integrate verify checks into `apm/src/cmd/validate.rs`**

In `run()`, after loading `CmdContext`, and before the `--config-only` early-return path:

- Unless `config_only` is true, call `apm_core::git::merged_into_main(root, &ctx.config.project.default_branch).unwrap_or_default()` and collect into a `HashSet<String>`.
- Call `apm_core::validate::verify_tickets(root, &ctx.config, &ctx.tickets, &merged_set)` and append each result to the issue list with `kind = "integrity"` (or reuse an appropriate existing kind — whatever keeps the JSON schema consistent).
- In `--fix` mode, after existing branch-field repairs, run the merged-ticket close loop (same body as `cmd/verify.rs::apply_fixes`). Do not add any worktree-recreation logic.
- The informational lines that `cmd/verify.rs` prints (completion-strategy report, logging path) are not carried over — they are not part of an integrity check.

Do not alter `--config-only` or `--no-aggressive` semantics. `--config-only` already exits before ticket iteration; the `merged_into_main` call must also be behind the `!config_only` guard. `--no-aggressive` is already threaded into `CmdContext::load` and controls the fetch; the `merged_into_main` call happens after load, so it respects the same flag implicitly.

Delete `apm/src/cmd/verify.rs`.

**Step 4 — Remove the `Verify` command from `apm/src/main.rs`**

- Delete the `Verify { fix, no_aggressive }` variant and its `#[command]` doc block from the `Command` enum.
- Delete the dispatch arm `Command::Verify { .. } => cmd::verify::run(...)`.
- Remove `mod verify;` from `mod cmd { ... }` (or wherever the module is declared in main.rs).

**Step 5 — Update `apm/src/hash_trip.rs`**

- Remove `| super::Command::Verify { .. }` from `is_read_only_command()`.
- Delete the `verify_is_read_only` test in `hash_trip.rs`.

**Step 6 — Update `docs/commands.md`**

- Delete the `apm verify` section (~lines 1066–1102).
- Expand the `apm validate` section to list all checks now performed. Group them clearly:
  - *Config checks*: config parse errors, invalid state-transition targets, git-host provider required for pr/merge transitions, instruction file paths, warning-only checks (e.g. missing Docker).
  - *Ticket integrity checks* (previously validate): branch-field mismatches, duplicate branch assignments, dependency-rule violations per completion strategy.
  - *Ticket integrity checks* (previously verify): unknown state values, ID/filename mismatches, missing branch for in_progress/implemented, merged-but-open branches, missing `## Spec`/`## History` sections, document-structure validation errors, missing worktrees for in_design/in_progress.
- Update `--fix` description to cover both repair classes.
- Update `--config-only` description to note it skips all per-ticket and filesystem checks.

**Step 7 — Update `README.md`**

Search for `apm verify`. Replace the two-line block that references both commands with a single entry for `apm validate` covering all integrity checks.

**Constraints**

- `verify_tickets` must NOT auto-close or auto-recreate; it only returns `Vec<String>`. The fix logic lives exclusively in `cmd/validate.rs::apply_fixes`.
- The `--fix` flag must not silently re-provision missing worktrees. The worktree-missing issue is reported and the command exits non-zero; the implementer must verify no worktree-creation code path is introduced anywhere in this diff.
- Backward compatibility: no behaviour changes to `--config-only`, `--json`, `--no-aggressive`, or existing validate checks.

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-04-28T19:19Z | — | new | philippepascal |
| 2026-04-28T19:32Z | new | groomed | philippepascal |
| 2026-04-28T19:33Z | groomed | in_design | philippepascal |