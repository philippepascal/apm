+++
id = "3665e017"
title = "apm help commands: derive command listing from clap"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/3665e017-apm-help-commands-derive-command-listing"
created_at = "2026-04-28T19:27:18.217337Z"
updated_at = "2026-04-28T19:37:58.638986Z"
epic = "e3b24cb9"
target_branch = "epic/e3b24cb9-apm-help-auto-derived-git-style-topic-he"
depends_on = ["bc89e0a0"]
+++

## Spec

### Problem

The `render_commands()` function in `apm/src/cmd/help.rs` is introduced as a stub by ticket bc89e0a0. It returns a placeholder string and does nothing useful. As a result, `apm help commands` gives no actionable information to users.

This ticket replaces that stub with a real implementation that walks clap's introspection API at runtime to produce a full command/flag reference. Because the output is derived directly from `crate::Cli::command()`, it never drifts from the actual CLI definition — new commands, subcommands, and flags appear automatically without any code changes to `help.rs`.

The current `help_template` in `main.rs` provides a grouped overview (Setup / Ticket management / Workflow / Epics / Maintenance / Server), but it is hand-written and contains only one-liners. Users who need to know what arguments a specific command accepts must run `apm <command> --help` individually. `apm help commands` should give the full argument reference for every command in one place.

### Acceptance criteria

- [ ] `apm help commands` exits 0 and prints output to stdout
- [ ] Every non-hidden top-level command appears in the output (hidden commands such as `_hook` do not appear)
- [ ] Each command entry shows the command name and its one-line description (from `get_about()`)
- [ ] Each command entry lists its positional arguments with their value-name label and help text
- [ ] Each command entry lists its flags/options with long name, short name (if any), value name (if any), and help text
- [ ] Flags with a default value show the default in their entry
- [ ] Auto-generated clap flags (`--help`, `--version`) do not appear in the output
- [ ] Nested subcommands (e.g. `epic new`, `epic close`) are listed with their full dotted/spaced path under the parent command
- [ ] Adding a new `#[arg]` to any subcommand causes it to appear in `apm help commands` output without changes to `help.rs`
- [ ] Adding a new variant to the `Command` enum causes it to appear in `apm help commands` output without changes to `help.rs`
- [ ] Output lines do not exceed 100 characters (long descriptions are wrapped at word boundaries)
- [ ] Output contains no ANSI escape codes (plain text only)

### Out of scope

- Color, ANSI styling, or markdown rendering in the output
- Pager integration (no `less`/`more` invocation)
- Fuzzy-matching or "did you mean?" suggestions for unknown topics (that belongs in ticket bc89e0a0's dispatcher)
- Changes to how `apm <subcommand> --help` works (clap-native help is untouched)
- Content for the `config`, `workflow`, or `ticket` help topics (sibling tickets d486d183, 7ba021e8, 14214305)
- Grouping commands by category (Setup / Workflow / etc.) — alphabetical or Cli-enum order only

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
| 2026-04-28T19:37Z | groomed | in_design | philippepascal |