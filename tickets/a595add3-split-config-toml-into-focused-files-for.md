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


### Out of scope

Explicit list of what this ticket does not cover.

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