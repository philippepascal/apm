+++
id = "c8d5590d"
title = "correct mispelling ammend->amend in workflow and anywhere else it might be"
state = "in_progress"
priority = 0
effort = 3
risk = 2
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/c8d5590d-correct-mispelling-ammend-amend-in-workf"
created_at = "2026-06-16T18:18:48.186548Z"
updated_at = "2026-06-16T20:57:46.698192Z"
+++

## Spec

### Problem

The workflow state intended to request spec or implementation revisions is named `ammend` throughout the codebase — in the workflow TOML, Rust source, tests, agent instructions, and documentation. The correct English spelling is `amend`. The misspelling propagated from the initial workflow definition and was copied into every layer that references the state by name.

Because the state ID is a bare string used in comparisons, config files, TOML fixtures, and user-facing help text, the misspelling appears in the interface agents and supervisors see every time they interact with this state. Fixing it corrects the language without changing any behaviour.

### Acceptance criteria

- [x] `apm state <id> amend` transitions a ticket to the `amend` state
- [x] `apm review <id> --to amend` transitions a ticket to the `amend` state
- [x] `apm instructions` output contains `amend` (not `ammend`) in the state machine table
- [x] `apm list` and `apm show` display `amend` for tickets in that state
- [x] `cargo test --workspace` passes with no references to `ammend` remaining in source
- [x] The default `workflow.toml` embedded in `apm-core` defines the state id as `amend`
- [x] The project's `.apm/workflow.toml` defines the state id as `amend`
- [x] Agent instruction files (`.apm/agents/` and `apm-core/src/default/agents/`) reference `amend` only

### Out of scope

- Archive ticket files under `archive/` — historical records, not live data
- Ticket branch names that happen to contain the word `ammend` in their slug (branch names are load-bearing and cannot be renamed)
- The title of this ticket itself (slug is frozen in the branch name)
- Any occurrences in `tickets/` Markdown files currently in states other than `amend` (none presently in that state, so no migration is needed)

### Approach

This is a pure string-rename across all active source files. No logic changes. However, `apm-core/src/state.rs` hardcodes the state id as a string literal to gate two runtime behaviours (see Rust source files → state.rs below); this makes the rename a **breaking change for downstream repos** that have not yet updated their `workflow.toml`. No migration is needed for this project — `grep -rni 'state = "ammend"' tickets/` returns nothing, so no live tickets are in this state.

After each batch of changes, run the verification grep (see Verification) to confirm coverage before moving on.

#### Workflow TOML files (define the canonical state ID)

- **`apm-core/src/default/workflow.toml`** — rename `id = "ammend"` → `"amend"`, `label = "Ammend"` → `"Amend"`, both `to = "ammend"` → `"amend"`. This is the embedded default copied into new projects by `apm init`.
- **`.apm/workflow.toml`** — identical changes. This is the live project workflow.

#### Rust source files

- **`apm-core/src/state.rs`** — two string comparisons: `old_state == "ammend"` → `"amend"` (line 111), `new_state == "ammend"` → `"amend"` (line 166).

  > **Breaking change for downstream repos.** Line 111 gates the "all amendment requests must be checked before resubmitting to specd" validation; line 166 gates `ensure_amendment_section()` insertion on transition. Both comparisons match the state id read at runtime from `workflow.toml`. Any external repo whose `workflow.toml` still names the state `ammend` after upgrading the binary will continue to transition correctly but will silently lose these two behaviours. See ticket 68829abb (migration-docs) for the documentation update that should accompany this rename.

- **`apm-core/src/instructions.rs`** — two lines in `STATIC_STATE_MACHINE`: `"ammend"` → `"amend"` (both the state ID and the command example).
- **`apm-core/src/config.rs`** — test fixture literals: `to = "ammend"` → `"amend"`, `id = "ammend"` → `"amend"`, `label = "Ammend"` → `"Amend"`.
- **`apm-core/src/init.rs`** — occurrences in `default_workflow_toml_is_valid` test: all `"ammend"` and `"Ammend"` → corrected spellings.
- **`apm-core/src/epic.rs`** — eight occurrences across test fixtures and a test function; rely on the case-insensitive grep to locate all of them. Known locations: lines 591, 592, 613, 614 (`id`/`label` fixture pairs), 737 (function name `epic_is_quiescent_ammend_with_impl_history_blocks` → `epic_is_quiescent_amend_with_impl_history_blocks`), 747–748 (fixture state data), 753 (assertion message string).
- **`apm/src/main.rs`** — one occurrence in help text: `--to ammend` → `--to amend`.
- **`apm/src/cmd/review.rs`** — one string comparison: `Some("ammend")` → `Some("amend")`.
- **`apm-server/src/workers.rs`** — one occurrence in an excluded-states array: `"ammend"` → `"amend"`.
- **`apm-server/src/main.rs`** — one occurrence in a test assertion string: `"ammend"` → `"amend"`.

#### Test files

- **`apm/tests/integration.rs`** — multiple occurrences: state strings in state arrays, TOML fixture literals, test function names (`state_ammend_inserts_amendment_section` → `state_amend_inserts_amendment_section`, `spawn_ammend_ticket_transitions_to_in_design` → `spawn_amend_ticket_transitions_to_in_design`, `review_ammend_normalises_plain_bullets_to_checkboxes` → `review_amend_normalises_plain_bullets_to_checkboxes`), and inline comments.
- **`apm/tests/e2e.rs`** — one TOML fixture literal: `id = "ammend"` → `"amend"`.

#### Agent instruction Markdown files

- **`.apm/agents/claude/apm.main-agent.md`** — three occurrences in transition examples and the "amend a ticket" section.
- **`.apm/agents/claude/apm.spec-writer.md`** — three occurrences in the handling-ammend section header and body.
- **`.apm/agents/pi/apm.spec-writer.md`** — one occurrence at line 156: `## Ammend tickets` → `## Amend tickets`.
- **`apm-core/src/default/agents/claude/apm.main-agent.md`** — same three as the claude main-agent above (this is the embedded copy).
- **`apm-core/src/default/agents/claude/apm.spec-writer.md`** — same three as the claude spec-writer above (this is the embedded copy).

#### Documentation and README

- **`docs/commands.md`** — two occurrences in the `apm review` section.
- **`docs/agent-wrappers.md`** — two occurrences in the example workflow config and surrounding prose.
- **`README.md`** — three occurrences in the workflow diagram and narrative text.

#### Verification

After all changes, run:
1. `grep -rni "ammend" . --include="*.rs" --include="*.toml" --include="*.md" | grep -v archive/ | grep -v ".git/"` — must return zero hits. The `-i` flag catches both lowercase `ammend` and capitalized `Ammend`. (The ticket's own branch-name slug is exempt, but it does not appear in any source file.)
2. `cargo test --workspace` — all tests must pass.

### Open questions


### Amendment requests

- [x] Acknowledge the downstream back-compat breakage. apm-core/src/state.rs:111 and :166 match the bare literal "ammend" to gate BEHAVIOUR, not just display — line 111 gates the 'all amendment requests must be checked before resubmitting to specd' validation; line 166 gates ensure_amendment_section() insertion. Because the state id is read from workflow.toml at runtime, any external repo whose workflow.toml still says 'ammend' will keep transitioning but SILENTLY lose these two behaviours after upgrading the binary. The spec frames this as a pure string rename with no migration; it must call this out as a known breaking change for downstream repos (reference the existing migration-docs ticket 68829abb).
- [x] Missed occurrence: .apm/agents/pi/apm.spec-writer.md:156 contains '## Ammend tickets' and is NOT in the spec's file list (which only enumerates the claude agent files). Left unchanged, it makes the spec's own 'grep returns zero hits' verification AC fail. Add this file to the rename list.
- [x] Make the verification grep case-insensitive. The capitalized variants 'Ammend' (e.g. label = "Ammend" in epic.rs fixtures, the '## Ammend tickets' header, prose) will slip through a case-sensitive 'grep -rn ammend'. Either add -i to the verification command or enumerate the capitalized occurrences explicitly so the 'zero hits' gate is meaningful.
- [x] Fix the epic.rs occurrence count. The spec says four occurrences in apm-core/src/epic.rs; there are actually six lines (591, 592, 613, 614 — including two label = "Ammend" fixture lines; 737 fn name; 747, 748 fixture data; 753 assertion message). An implementer relying on the enumerated list rather than a grep would miss the 'Ammend' label casings. Update the count/list (or instruct to rely on a case-insensitive grep).

### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-06-16T18:18Z | — | new | philippepascal |
| 2026-06-16T18:19Z | new | groomed | philippepascal |
| 2026-06-16T18:19Z | groomed | in_design | philippepascal |
| 2026-06-16T18:22Z | in_design | specd | claude |
| 2026-06-16T19:29Z | specd | ammend | philippepascal |
| 2026-06-16T19:36Z | ammend | in_design | philippepascal |
| 2026-06-16T19:38Z | in_design | specd | claude |
| 2026-06-16T20:24Z | specd | ready | philippepascal |
| 2026-06-16T20:57Z | ready | in_progress | philippepascal |