+++
id = "a595add3"
title = "Split config.toml into focused files for state machine, ticket structure, and project settings"
state = "closed"
priority = 9
effort = 4
risk = 2
author = "claude-0401-2145-a8f3"
agent = "73872"
branch = "ticket/a595add3-split-config-toml-into-focused-files-for"
created_at = "2026-04-01T22:27:35.511052Z"
updated_at = "2026-04-01T23:07:09.230079Z"
+++

## Spec

### Problem

The single `.apm/config.toml` file mixes three unrelated concerns in one 324-line file:

1. **Project identity and infrastructure** — `[project]`, `[tickets]`, `[worktrees]`, `[[repos.code]]`, `[provider]`, `[agents]`, `[sync]`, `[logging]`, `[workers]`
2. **State machine** — `[workflow]`, `[[workflow.states]]` (11 states with transitions, ~200 lines), `[workflow.prioritization]`
3. **Ticket body structure** — `[[ticket.sections]]` (7 section definitions)

As the project evolves, this creates practical problems:
- Navigating the file is difficult; a reader wanting to tweak a transition must scroll past or around ticket sections and project settings.
- Swapping or customising the state machine requires touching the same file that holds unrelated project settings.
- Code reviews for workflow changes are noisy because unrelated settings appear in the diff context.

The desired state is three focused files under `.apm/`:
- `config.toml` — project identity and infrastructure settings (already the primary config file; retains its name but sheds the workflow and ticket-section content)
- `workflow.toml` — the state machine: `[workflow]`, `[[workflow.states]]`, `[workflow.prioritization]`
- `ticket.toml` — ticket body structure: `[[ticket.sections]]`

The `Config::load()` function in `apm-core/src/config.rs` must be updated to read all three files and merge them into the existing `Config` struct. Backward compatibility with projects that still keep everything in one `config.toml` must be preserved.

### Acceptance criteria

- [x] `apm` loads a project whose `.apm/` directory contains three separate files (`config.toml`, `workflow.toml`, `ticket.toml`) without error
- [x] `apm` loads a project whose `.apm/config.toml` still contains `[workflow]` and `[[ticket.sections]]` (legacy monolithic layout) without error
- [x] When both `workflow.toml` and a `[workflow]` block in `config.toml` exist, the content from `workflow.toml` takes precedence but show a warning in apm validate
- [x] When both `ticket.toml` and `[[ticket.sections]]` in `config.toml` exist, the content from `ticket.toml` takes precedence but show a warning in apm validate
- [x] `apm init` on a new project creates `.apm/config.toml`, `.apm/workflow.toml`, and `.apm/ticket.toml` as separate files
- [x] The live `.apm/config.toml` in this repository no longer contains `[[workflow.states]]` or `[[ticket.sections]]` blocks
- [x] `cargo test --workspace` passes

### Out of scope

- Changing the schema or content of any config section (states, transitions, ticket sections remain identical)
- Providing a CLI command or migration tool to split an existing monolithic `config.toml` — existing projects keep working as-is; only new `apm init` produces the split layout
- Validating that the three files do not contain conflicting or duplicate keys beyond the precedence rule stated in acceptance criteria
- Moving `[tickets]` (the `dir` path) out of `config.toml` — it stays in the project settings file
- Any UI or documentation changes beyond updating the `apm.agents.md` reference if it mentions `apm.toml`

### Approach

**1. Update `Config::load()` in `apm-core/src/config.rs`**

Replace the current single-file read with a merge of up to three files:

- Load primary config (project/infra settings) from `.apm/config.toml` (or legacy `apm.toml`) into `Config` as today; `workflow` and `ticket` default to empty if absent
- If `.apm/workflow.toml` exists, parse it into a thin wrapper `struct WorkflowFile { workflow: WorkflowConfig }` and replace `config.workflow`
- If `.apm/ticket.toml` exists, parse it into a thin wrapper `struct TicketFile { ticket: TicketConfig }` and replace `config.ticket`

Add two private wrapper structs in the same file:

    struct WorkflowFile { workflow: WorkflowConfig }
    struct TicketFile { ticket: TicketConfig }  // ticket defaults to empty

No callers change -- they all call `Config::load(root)` and receive the same `Config` type.

**2. Split the live `.apm/config.toml`**

- Move `[[workflow.states]]` blocks and `[workflow]` / `[workflow.prioritization]` into a new `.apm/workflow.toml`
- Move `[[ticket.sections]]` entries into a new `.apm/ticket.toml`
- Leave `[project]`, `[tickets]`, `[worktrees]`, `[[repos.code]]`, `[provider]`, `[agents]`, `[sync]`, `[logging]` in `.apm/config.toml`

**3. Update `apm init` in `apm-core/src/init.rs`**

- Split `default_config()` into three functions: `default_config()`, `default_workflow_toml()`, `default_ticket_toml()`
- `setup()` creates all three files under `.apm/` (skip each if already exists)
- `maybe_initial_commit()` stages all three: `git add .apm/config.toml .apm/workflow.toml .apm/ticket.toml`
- `default_config()` retains project/infra content only (no workflow or ticket-section content)
- `default_workflow_toml()` produces the standard states + prioritization block
- `default_ticket_toml()` produces the seven standard `[[ticket.sections]]` entries

**4. Update integration tests in `apm/tests/integration.rs`**

Tests that write an inline `apm.toml` with workflow content continue to work (backward-compat path). Tests that assert on `.apm/config.toml` content for workflow strings (e.g. `[[workflow.states]]`) must read `.apm/workflow.toml` instead.

Lines to audit: approx 157, 171, 173, 186, 423 -- check whether each assertion targets workflow, ticket-section, or project content and redirect to the correct file.

**5. Order of changes**

1. `apm-core/src/config.rs` -- add wrapper structs and update `load()`
2. `apm-core/src/init.rs` -- split `default_config()`, update `setup()` and `maybe_initial_commit()`
3. Split live `.apm/config.toml` into three files
4. Fix any integration tests broken by the new init layout
5. Run `cargo test --workspace`

### Open questions



### Amendment requests
## History

| When | From | To | By |
|------|------|----|----|
| 2026-04-01T22:27Z | — | new | claude-0401-2145-a8f3 |
| 2026-04-01T22:28Z | new | groomed | claude-0401-2145-a8f3 |
| 2026-04-01T22:35Z | groomed | in_design | philippepascal |
| 2026-04-01T22:40Z | in_design | specd | claude-0401-2200-spec1 |
| 2026-04-01T22:46Z | specd | ready | apm |
| 2026-04-01T22:46Z | ready | in_progress | philippepascal |
| 2026-04-01T22:53Z | in_progress | implemented | claude-0401-2300-w001 |
| 2026-04-01T23:07Z | implemented | closed | apm-sync |