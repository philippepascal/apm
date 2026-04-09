+++
id = "3dd06092"
title = "organize apm help"
state = "ready"
priority = 0
effort = 3
risk = 2
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/3dd06092-organize-apm-help"
created_at = "2026-04-09T00:55:24.172727Z"
updated_at = "2026-04-09T01:33:16.716666Z"
+++

## Spec

### Problem

The `apm --help` output lists all ~25 commands in a flat, undifferentiated block. Commands from very different concern areas — browsing tickets, driving workflow, maintaining the repo, administering the server — sit side by side with no visual grouping. A new user or agent scanning the list has no quick signal about which commands matter for their role.

The desired behaviour is a grouped help output where commands are clustered under short headings (e.g. "Ticket management", "Workflow", "Maintenance", "Server"). The order and grouping should match the natural workflow: setup and browsing first, then the actions most commonly used day-to-day, with maintenance and server admin at the bottom.

This affects every user of the CLI — human engineers, supervisors, and agent workers — since `apm --help` is typically the first thing consulted when learning or troubleshooting the tool.

### Acceptance criteria

- [ ] `apm --help` output displays commands under named group headings (e.g. "Ticket management", "Workflow", "Maintenance", "Server")
- [ ] Each existing command appears under exactly one group heading
- [ ] Commands not shown in groups (hidden commands like `_hook`) remain hidden
- [ ] `apm <command> --help` output for individual commands is unchanged
- [ ] `apm help <command>` still works and shows per-command help
- [ ] Group headings appear in this order: Setup, Ticket management, Workflow, Epics, Maintenance, Server
- [ ] The long_about preamble (workflow states, actors, common entry points) is preserved unchanged

### Out of scope

- Per-command help text rewrites (doc comments / long_about on individual commands)
- Reordering commands within a group beyond what the grouping itself implies
- Adding new commands or removing existing ones
- Changing any CLI behaviour — only the top-level help display changes
- Filtering or personalising help output based on user role (agent vs supervisor)

### Approach

**Proposed groupings** (order within each group follows enum declaration order):

| Group heading | Commands |
|---|---|
| Setup | init, agents |
| Ticket management | new, list, show, set, state, spec, close, assign |
| Workflow | next, start, review, work, sync |
| Epics | epic |
| Maintenance | worktrees, clean, workers, verify, validate, archive |
| Server | register, sessions, revoke |

**Implementation — Clap 4 flatten + next_help_heading**

Split the flat `Command` enum in `apm/src/main.rs` into one sub-enum per group. Each sub-enum carries `#[command(next_help_heading = "<Group>")]`. The top-level `Command` enum holds one `#[command(flatten)]` variant per sub-enum.

Files that change:
- `apm/src/main.rs` — only file that needs modification

Steps:
1. Define six new sub-enums (SetupCommands, TicketCommands, WorkflowCommands, EpicCommands, MaintenanceCommands, ServerCommands), each annotated with `#[command(next_help_heading = "...")]`
2. Move each variant of the existing `Command` enum into the appropriate sub-enum, preserving all attributes and fields exactly
3. Replace the body of `Command` with six flatten variants, one per sub-enum
4. Update every match arm in `main()` to route through the new wrapper: e.g. `Command::Ticket(TicketCommands::New { ... }) => cmd::new::run(...)`. Handler modules in `apm/src/cmd/` are untouched
5. Keep `_hook` in MaintenanceCommands with `#[command(hide = true)]`; keep `EpicCommand` (already a nested sub-enum) unchanged inside EpicCommands

**Fallback** if Clap 4 does not render section headers for flattened subcommand enums at runtime: use `display_order` on each variant to cluster them in the flat enum and add group labels in the `long_about` preamble. This is lower-fidelity (no inline headers) but requires no structural change.

No handler modules change. The only observable effect is the `apm --help` output gaining group headings.

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-04-09T00:55Z | — | new | philippepascal |
| 2026-04-09T00:55Z | new | groomed | apm |
| 2026-04-09T00:55Z | groomed | in_design | philippepascal |
| 2026-04-09T00:59Z | in_design | specd | claude-0409-0055-acb8 |
| 2026-04-09T01:15Z | specd | ready | apm |
| 2026-04-09T01:15Z | ready | in_progress | philippepascal |
| 2026-04-09T01:33Z | in_progress | ready | apm |
