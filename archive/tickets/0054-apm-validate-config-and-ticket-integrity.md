+++
id = 54
title = "apm validate: config and ticket integrity checker"
state = "closed"
priority = 3
effort = 3
risk = 1
author = "claude-0329-1200-a1b2"
agent = "claude-0329-impl-54"
branch = "ticket/0054-apm-validate-config-and-ticket-integrity"
created_at = "2026-03-29T19:11:40.144856Z"
updated_at = "2026-03-30T02:02:46.501095Z"
+++

## Spec

### Problem

APM currently has `apm verify`, which checks git and cache integrity (merged branches, state consistency, missing sections). What it does not do is validate that `apm.toml` itself is internally consistent, or that ticket files satisfy the structural rules derived from the config schema.

Concretely, a misconfigured `apm.toml` can silently break the entire workflow: a transition that references a non-existent target state, an `instructions` path that points to a missing file, or a `[ticket.sections]` entry with an invalid `type` value will not be caught until the affected command runs and fails with a cryptic error. Similarly, a ticket file whose `state` field names a state not in `apm.toml`, or whose required sections are empty past `specd`, cannot be detected by `apm verify` today because `apm verify` focuses on git state, not schema/content correctness.

`apm validate` fills this gap: a read-only, structured checker that reports schema violations in config and content violations in tickets, exits non-zero when errors are found, and optionally emits machine-readable JSON. It is the "lint" command for the APM project, separate from `apm verify`'s "fsck" role.

### Acceptance criteria

- [x] `apm validate` runs without arguments, reads `apm.toml` from the repo root, and prints a summary line `N tickets checked, N errors, N warnings`
- [x] Exits 0 when no errors are found; exits 1 when at least one error is found
- [x] Config check — all `to` targets in all transitions reference a state that exists in `[[workflow.states]]`; violations are reported as errors
- [x] Config check — every value in `preconditions` arrays is one of the known strings: `spec_not_empty`, `spec_has_acceptance_criteria`, `pr_exists`, `pr_all_closing_merged`; unknown values are reported as errors
- [x] Config check — every value in `side_effects` arrays is one of the known strings: `set_agent_null`; unknown values are reported as errors
- [x] Config check — no state has both `terminal = true` and at least one outgoing transition; violations are reported as errors
- [x] Config check — at least one non-terminal state exists; absence is reported as an error
- [x] Config check (gated on ticket #53) — each `[[ticket.sections]]` entry has a `type` value that is one of `"free"`, `"tasks"`, `"qa"`; invalid values are reported as errors
- [x] Config check (gated on ticket #53) — each `instructions` field on `[[workflow.states]]` that is set points to a file that exists relative to the repo root; missing files are reported as errors
- [x] Ticket check — each ticket file's `state` field names a state that exists in `[[workflow.states]]`; violations are reported as errors
- [x] Ticket check — each ticket file's `branch` field, when set, matches the canonical branch name `ticket/<id>-<slug>` that would be derived from the filename; mismatches are reported as errors
- [x] Ticket check (gated on ticket #53) — for tickets in `specd` or any later non-terminal state, each `[[ticket.sections]]` entry with `required = true` must have non-empty content in the ticket body; violations are reported as errors
- [x] `--json` flag outputs a JSON object `{"tickets_checked": N, "errors": [...], "warnings": [...]}` instead of human-readable lines; errors and warnings are objects with `"kind"`, `"subject"`, and `"message"` fields
- [x] `--fix` flag auto-repairs the one fixable error class: a `branch` field that is set but does not match the canonical name is rewritten to match and committed to the ticket branch; all other issues are reported only, never silently mutated
- [x] `apm validate` is listed in `apm --help` output
- [x] `cargo test --workspace` passes with at least: one test that validates a correct config passes; one test that detects a transition to a non-existent state; one test that detects a terminal state with outgoing transitions; one test that detects an unknown precondition; one test that detects a ticket with an unknown state

### Out of scope

- Checking `apm.toml` TOML syntax (the existing `Config::load` parse already errors on invalid TOML)
- Checking git object integrity or cache consistency (that is `apm verify`'s domain)
- Validating the content of `## History` rows
- Checking that `effort` and `risk` are non-zero for tickets past `in_design`
- Validating PR existence or merge status
- Repairing config errors — `--fix` only repairs ticket `branch` field mismatches
- Adding new precondition or side-effect strings beyond those already in the codebase; the known-values lists are derived from what `apm-core` currently evaluates

### Approach

Add a new `apm validate` subcommand alongside the existing commands in `apm/src/cmd/`. The implementation is a single pass over the config and a second pass over all tickets; no network calls, no git operations except reading ticket blobs.

**Command wiring** (`apm/src/main.rs`):
Add `Validate { fix: bool, json: bool }` to the `Command` enum and dispatch to `cmd::validate::run(root, fix, json)`.

**Config validation** (`apm/src/cmd/validate.rs`):
1. Collect all declared state IDs into a `HashSet<&str>`.
2. For each state, for each transition: check `to` is in the set.
3. For each state: if `terminal = true` and `transitions` is non-empty, error.
4. Check at least one state with `terminal = false` exists.
5. For each transition: check each precondition string against the known set; check each side-effect string against the known set.
6. If ticket #53 config fields are present (i.e., `config.ticket.sections` is non-empty): check each section's `type_` is a valid `SectionType` (this is enforced by serde deserialization but adds an explicit error message for the report); check each state's `instructions` path exists.

**Ticket validation** (`apm/src/cmd/validate.rs`, continued):
Load all tickets via `ticket::load_all_from_git`. For each ticket:
1. Check `state` is in the declared states set.
2. Check `branch` field, if set, matches `ticket/<id>-<slug>` derived from the filename.
3. If ticket #53 sections are configured and the ticket's state is at or after `specd` in the lifecycle: check each required section has non-empty content in the ticket body. The "at or after specd" check is done by walking the transition graph from `specd` to collect reachable non-terminal states.

**Output**:
- Default: print one line per error/warning, then the summary line.
- `--json`: serialize to JSON, print, exit.

**`--fix`**:
For each ticket where `branch` is set but wrong: rewrite `frontmatter.branch` to the canonical value, serialize, and call `git::commit_to_branch` (same pattern as `verify --fix`).

**Known preconditions** (static list in `validate.rs`):
`["spec_not_empty", "spec_has_acceptance_criteria", "pr_exists", "pr_all_closing_merged"]`

**Known side effects** (static list):
`["set_agent_null"]`

**Dependency**: the `instructions` and `[[ticket.sections]]` checks require ticket #53 (Config schema: ticket.sections, state instructions, transition completion and focus_section) to be implemented first. The checks are gated on `config.ticket.sections.is_empty()` and `state.instructions.is_some()` — if the fields are absent (pre-#53), those checks are silently skipped rather than failing.

## History

| When | From | To | By |
|------|------|----|----|
| 2026-03-29T19:11Z | — | new | claude-0329-1200-a1b2 |
| 2026-03-29T19:38Z | new | in_design | claude-0329-spec-54 |
| 2026-03-29T19:41Z | in_design | specd | claude-0329-spec-54 |
| 2026-03-29T19:42Z | specd | ready | claude-0329-1200-a1b2 |
| 2026-03-29T19:48Z | ready | in_progress | claude-0329-impl-54 |
| 2026-03-29T19:56Z | in_progress | implemented | claude-0329-impl-54 |
| 2026-03-29T20:19Z | implemented | accepted | claude-0329-main |
| 2026-03-30T02:02Z | accepted | closed | apm-sync |