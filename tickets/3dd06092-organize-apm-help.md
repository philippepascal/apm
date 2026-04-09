+++
id = "3dd06092"
title = "organize apm help"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/3dd06092-organize-apm-help"
created_at = "2026-04-09T00:55:24.172727Z"
updated_at = "2026-04-09T00:55:49.985306Z"
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

Explicit list of what this ticket does not cover.

### Approach

How the implementation will work.

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-04-09T00:55Z | — | new | philippepascal |
| 2026-04-09T00:55Z | new | groomed | apm |
| 2026-04-09T00:55Z | groomed | in_design | philippepascal |