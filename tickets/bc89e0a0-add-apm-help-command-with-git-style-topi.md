+++
id = "bc89e0a0"
title = "Add apm help command with git-style topic dispatch"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/bc89e0a0-add-apm-help-command-with-git-style-topi"
created_at = "2026-04-28T19:27:00.760945Z"
updated_at = "2026-04-28T19:33:33.568010Z"
epic = "e3b24cb9"
target_branch = "epic/e3b24cb9-apm-help-auto-derived-git-style-topic-he"
+++

## Spec

### Problem

There is no unified `apm help` command today. Users discover surface area by running `apm <subcommand> --help` for each command and reading source for config schemas. This ticket adds the top-level `apm help [topic]` command with git-style topic dispatch.

**Scope (this ticket):** the CLI plumbing only. Topic content arrives in subsequent tickets in this epic.

**Behavior to implement:**
- `apm help` (no topic) prints an overview: a short description of the help system plus the list of available topics with one-line summaries.
- `apm help <topic>` calls a topic-specific renderer. Initial topics: `commands`, `config`, `workflow`, `ticket`.
- `apm help <unknown-topic>` exits non-zero with a clear error and the topic list.
- Each topic renderer is a separate function returning `String` so subsequent tickets can replace stubs independently.

**Implementation pointers:**
- `apm/src/main.rs`: add `Help { topic: Option<String> }` to the `Command` enum. Wire dispatch to `cmd::help::run(topic.as_deref())`.
- `apm/src/cmd/help.rs`: new module. Pub fn `run(topic: Option<&str>) -> Result<()>`. Internal stub functions `render_commands()`, `render_config()`, `render_workflow()`, `render_ticket()` that initially return "This topic will be populated by ticket <ID>" placeholders.
- Output goes to stdout. No paging in this ticket.

**Out of scope:**
- Actual content for any topic (each is its own ticket).
- Auto-derive infrastructure (separate ticket).
- Pager integration, markdown rendering, color output.

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
| 2026-04-28T19:27Z | — | new | philippepascal |
| 2026-04-28T19:32Z | new | groomed | philippepascal |
| 2026-04-28T19:33Z | groomed | in_design | philippepascal |
