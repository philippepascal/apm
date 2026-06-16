+++
id = "c8d5590d"
title = "correct mispelling ammend->amend in workflow and anywhere else it might be"
state = "ammend"
priority = 0
effort = 3
risk = 2
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/c8d5590d-correct-mispelling-ammend-amend-in-workf"
created_at = "2026-06-16T18:18:48.186548Z"
updated_at = "2026-06-16T19:29:16.231713Z"
+++

## Spec

### Problem

The workflow state intended to request spec or implementation revisions is named `ammend` throughout the codebase — in the workflow TOML, Rust source, tests, agent instructions, and documentation. The correct English spelling is `amend`. The misspelling propagated from the initial workflow definition and was copied into every layer that references the state by name.

Because the state ID is a bare string used in comparisons, config files, TOML fixtures, and user-facing help text, the misspelling appears in the interface agents and supervisors see every time they interact with this state. Fixing it corrects the language without changing any behaviour.

### Acceptance criteria

- [ ] `apm state <id> amend` transitions a ticket to the `amend` state
- [ ] `apm review <id> --to amend` transitions a ticket to the `amend` state
- [ ] `apm instructions` output contains `amend` (not `ammend`) in the state machine table
- [ ] `apm list` and `apm show` display `amend` for tickets in that state
- [ ] `cargo test --workspace` passes with no references to `ammend` remaining in source
- [ ] The default `workflow.toml` embedded in `apm-core` defines the state id as `amend`
- [ ] The project's `.apm/workflow.toml` defines the state id as `amend`
- [ ] Agent instruction files (`.apm/agents/` and `apm-core/src/default/agents/`) reference `amend` only

### Out of scope

- Archive ticket files under `archive/` — historical records, not live data
- Ticket branch names that happen to contain the word `ammend` in their slug (branch names are load-bearing and cannot be renamed)
- The title of this ticket itself (slug is frozen in the branch name)
- Any occurrences in `tickets/` Markdown files currently in states other than `amend` (none presently in that state, so no migration is needed)

### Approach

This is a pure string-rename across all active source files. No logic changes. No migration needed — `grep -rn 'state = "ammend"' tickets/` returns nothing, so no live tickets are in this state.

Run `grep -rn "ammend" . --include="*.rs" --include="*.toml" --include="*.md" | grep -v archive/ | grep -v ".git/"` after each step to confirm coverage.

#### Workflow TOML files (define the canonical state ID)

- **`apm-core/src/default/workflow.toml`** — rename `id = "ammend"` → `"amend"`, `label = "Ammend"` → `"Amend"`, both `to = "ammend"` → `"amend"`. This is the embedded default copied into new projects by `apm init`.
- **`.apm/workflow.toml`** — identical changes. This is the live project workflow.

#### Rust source files

- **`apm-core/src/state.rs`** — two string comparisons: `old_state == "ammend"` → `"amend"`, `new_state == "ammend"` → `"amend"`.
- **`apm-core/src/instructions.rs`** — two lines in `STATIC_STATE_MACHINE`: `"ammend"` → `"amend"` (both the state ID and the command example).
- **`apm-core/src/config.rs`** — two test fixture literals: `to = "ammend"` → `"amend"`, `id = "ammend"` → `"amend"`, and `label = "Ammend"` → `"Amend"`.
- **`apm-core/src/init.rs`** — three occurrences in `default_workflow_toml_is_valid` test: all `"ammend"` → `"amend"`.
- **`apm-core/src/epic.rs`** — four occurrences: state string in test fixture data, test function name `epic_is_quiescent_ammend_with_impl_history_blocks` → `epic_is_quiescent_amend_with_impl_history_blocks`, and two assertion message strings.
- **`apm/src/main.rs`** — one occurrence in help text: `--to ammend` → `--to amend`.
- **`apm/src/cmd/review.rs`** — one string comparison: `Some("ammend")` → `Some("amend")`.
- **`apm-server/src/workers.rs`** — one occurrence in an excluded-states array: `"ammend"` → `"amend"`.
- **`apm-server/src/main.rs`** — one occurrence in a test assertion string: `"ammend"` → `"amend"`.

#### Test files

- **`apm/tests/integration.rs`** — several occurrences across multiple tests: state strings in state arrays, TOML fixture literals, test function names (`state_ammend_inserts_amendment_section` → `state_amend_inserts_amendment_section`, `spawn_ammend_ticket_transitions_to_in_design` → `spawn_amend_ticket_transitions_to_in_design`, `review_ammend_normalises_plain_bullets_to_checkboxes` → `review_amend_normalises_plain_bullets_to_checkboxes`), and inline comments.
- **`apm/tests/e2e.rs`** — one TOML fixture literal: `id = "ammend"` → `"amend"`.

#### Agent instruction Markdown files

- **`.apm/agents/claude/apm.main-agent.md`** — three occurrences in transition examples and the "amend a ticket" section.
- **`.apm/agents/claude/apm.spec-writer.md`** — three occurrences in the handling-ammend section header and body.
- **`apm-core/src/default/agents/claude/apm.main-agent.md`** — same three as above (this is the embedded copy).
- **`apm-core/src/default/agents/claude/apm.spec-writer.md`** — same three as above (this is the embedded copy).

#### Documentation and README

- **`docs/commands.md`** — two occurrences in the `apm review` section.
- **`docs/agent-wrappers.md`** — two occurrences in the example workflow config and surrounding prose.
- **`README.md`** — three occurrences in the workflow diagram and narrative text.

#### Verification

After all changes, run:
1. `grep -rn "ammend" . --include="*.rs" --include="*.toml" --include="*.md" | grep -v archive/ | grep -v ".git/"` — must return zero hits (only the ticket's own branch-name slug is exempt, and that doesn't appear in source files).
2. `cargo test --workspace` — all tests must pass.

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-06-16T18:18Z | — | new | philippepascal |
| 2026-06-16T18:19Z | new | groomed | philippepascal |
| 2026-06-16T18:19Z | groomed | in_design | philippepascal |
| 2026-06-16T18:22Z | in_design | specd | claude |
| 2026-06-16T19:29Z | specd | ammend | philippepascal |
