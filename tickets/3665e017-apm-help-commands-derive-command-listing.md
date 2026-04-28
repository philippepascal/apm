+++
id = "3665e017"
title = "apm help commands: derive command listing from clap"
state = "groomed"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/3665e017-apm-help-commands-derive-command-listing"
created_at = "2026-04-28T19:27:18.217337Z"
updated_at = "2026-04-28T19:32:48.707128Z"
epic = "e3b24cb9"
target_branch = "epic/e3b24cb9-apm-help-auto-derived-git-style-topic-he"
depends_on = ["bc89e0a0"]
+++

## Spec

### Problem

Replace the `render_commands()` stub from ticket bc89e0a0 with a real renderer that walks clap's introspection to produce a unified command/flag reference.

**Scope:** auto-derive the command listing from `crate::Cli::command()`. No hand-written command catalog.

**Behavior to implement:**
- Walk every subcommand recursively (top-level commands plus nested ones like `apm epic new`, `apm epic close`).
- For each command: name, one-line description (`get_about`), positional args (with type and description), flags (with long/short names, value name, default, description).
- Group long-form details under each command. Sort top-level commands alphabetically (or follow the order in `Cli`).
- Output is a single string returned to the caller; written to stdout.

**Implementation pointers:**
- Use `clap::Command::get_subcommands()`, `get_arguments()`, and the various `get_*` accessors to walk the command tree at runtime.
- This avoids drift entirely — the help reflects the actual Cli definition. New commands and flags appear automatically.
- Examples and `long_about` strings already live on each `#[command]` attribute; surface them in the rendered output.

**Acceptance pointers (for spec phase):**
- Adding a new `#[arg]` to any subcommand must show up in `apm help commands` without further code changes.
- Adding a whole new subcommand likewise appears automatically.
- Output is readable in a 100-column terminal without a pager.

**Out of scope:**
- Color or styling.
- Markdown rendering (plain text output).
- A separate `--help` per topic (we already have clap's built-in).

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
