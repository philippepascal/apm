+++
id = 77
title = "Audit and update CLI help text for consistency and completeness"
state = "closed"
priority = 0
effort = 3
risk = 1
author = "claude-0329-1430-main"
agent = "claude-0330-0245-main"
branch = "ticket/0077-audit-and-update-cli-help-text-for-consi"
created_at = "2026-03-30T00:59:27.415791Z"
updated_at = "2026-03-30T05:24:22.183366Z"
+++

## Spec

### Problem

Most `apm` subcommands have blank or minimal help text. Arguments have no descriptions. Options are listed without explanation. A new user or agent reading `apm <cmd> --help` cannot understand what a command does, what its arguments expect, or what its flags control. This makes onboarding harder and forces agents to read source code to understand the CLI contract.

Current gaps (representative sample):
- `apm list --state <STATE>` — no description of valid values
- `apm new <TITLE>` — no description of the title argument or available options
- `apm state <ID> <STATE>` — no description of what states are valid
- `apm set <ID> <FIELD> <VALUE>` — no description of valid fields or value ranges
- `apm start [ID]` — spawn/next options undocumented
- `apm take <ID>` — no description of what "take over" means
- `apm review <ID>` — `--to` flag unlisted in help output
- `apm spec <ID>` — section, mark, check flags undocumented

### Acceptance criteria

- [x] Every subcommand has a non-empty `about` string that describes what it does in one sentence
- [x] Every positional argument has a `.help("...")` annotation
- [x] Every `--flag` / `--option` has a `.help("...")` annotation
- [x] `apm --help` top-level output gives a one-line description of each subcommand that is accurate and consistent with the command's own `--help`
- [x] Valid values for enum-like arguments (e.g. `--state`, `FIELD` in `apm set`) are listed or described in the help text
- [x] `cargo test --workspace` passes after the changes

### Out of scope

- Redesigning any command's behavior or flags
- Adding new flags or options
- Man page or markdown docs generation

### Approach

Go through `apm/src/main.rs` and each file in `apm/src/cmd/` and add or improve:

- `#[command(about = "...")]` on each `Args` struct
- `.help("...")` on each `#[arg(...)]` field

For enum-like string arguments (state names, field names), list the valid values inline in the help string or use Clap's `value_parser` with a `PossibleValuesParser` where the set is static.

Work file by file: `main.rs`, then each cmd in alphabetical order.

## History

| When | From | To | By |
|------|------|----|----|
| 2026-03-30T00:59Z | — | new | claude-0329-1430-main |
| 2026-03-30T01:01Z | new | in_design | claude-0329-1430-main |
| 2026-03-30T01:03Z | in_design | specd | claude-0329-1430-main |
| 2026-03-30T01:05Z | specd | ready | claude-0329-1430-main |
| 2026-03-30T01:05Z | ready | in_progress | claude-0329-1430-main |
| 2026-03-30T02:43Z | claude-0329-1430-main | claude-0330-0245-main | handoff |
| 2026-03-30T02:48Z | in_progress | implemented | claude-0330-0245-main |
| 2026-03-30T04:43Z | implemented | accepted | apm |
| 2026-03-30T05:24Z | accepted | closed | apm-sync |