+++
id = "a595add3"
title = "Split config.toml into focused files for state machine, ticket structure, and project settings"
state = "in_design"
priority = 9
effort = 0
risk = 0
author = "claude-0401-2145-a8f3"
agent = "42283"
branch = "ticket/a595add3-split-config-toml-into-focused-files-for"
created_at = "2026-04-01T22:27:35.511052Z"
updated_at = "2026-04-01T22:35:07.194666Z"
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

- [ ] `apm` loads a project whose `.apm/` directory contains three separate files (`config.toml`, `workflow.toml`, `ticket.toml`) without error
- [ ] `apm` loads a project whose `.apm/config.toml` still contains `[workflow]` and `[[ticket.sections]]` (legacy monolithic layout) without error
- [ ] When both `workflow.toml` and a `[workflow]` block in `config.toml` exist, the content from `workflow.toml` takes precedence
- [ ] When both `ticket.toml` and `[[ticket.sections]]` in `config.toml` exist, the content from `ticket.toml` takes precedence
- [ ] `apm init` on a new project creates `.apm/config.toml`, `.apm/workflow.toml`, and `.apm/ticket.toml` as separate files
- [ ] The live `.apm/config.toml` in this repository no longer contains `[[workflow.states]]` or `[[ticket.sections]]` blocks
- [ ] `cargo test --workspace` passes

### Out of scope

- Changing the schema or content of any config section (states, transitions, ticket sections remain identical)
- Providing a CLI command or migration tool to split an existing monolithic `config.toml` — existing projects keep working as-is; only new `apm init` produces the split layout
- Validating that the three files do not contain conflicting or duplicate keys beyond the precedence rule stated in acceptance criteria
- Moving `[tickets]` (the `dir` path) out of `config.toml` — it stays in the project settings file
- Any UI or documentation changes beyond updating the `apm.agents.md` reference if it mentions `apm.toml`

### Approach

How the implementation will work.

### Open questions



### Amendment requests



## History

| When | From | To | By |
|------|------|----|----|
| 2026-04-01T22:27Z | — | new | claude-0401-2145-a8f3 |
| 2026-04-01T22:28Z | new | groomed | claude-0401-2145-a8f3 |
| 2026-04-01T22:35Z | groomed | in_design | philippepascal |