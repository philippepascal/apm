+++
id = "3665e017"
title = "apm help commands: derive command listing from clap"
state = "ready"
priority = 0
effort = 4
risk = 2
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/3665e017-apm-help-commands-derive-command-listing"
created_at = "2026-04-28T19:27:18.217337Z"
updated_at = "2026-04-29T03:42:50.701084Z"
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
- [ ] Top-level commands appear in alphabetical order in the output
- [ ] Each command entry shows the command name and its one-line description (from `get_about()`)
- [ ] Each command entry lists its positional arguments with their value-name label and help text
- [ ] Each command entry lists its flags/options with long name, short name (if any), value name (if any), and help text
- [ ] Flags with a default value show the default in their entry
- [ ] Auto-generated clap flags (`--help`, `--version`) do not appear in the output
- [ ] All five current `epic` subcommands — `epic new`, `epic close`, `epic list`, `epic show`, `epic set` — appear in the output under the `epic` parent entry, in declaration order
- [ ] Adding a new `#[arg]` to any subcommand causes it to appear in `apm help commands` output without changes to `help.rs`
- [ ] Adding a new variant to the `Command` enum causes it to appear in `apm help commands` output without changes to `help.rs`
- [ ] Output lines do not exceed 100 characters; `(default: …)` annotations count toward the 100-character limit and are wrapped at word boundaries like any other text
- [ ] Output contains no ANSI escape codes (plain text only)

### Out of scope

- Color, ANSI styling, or markdown rendering in the output
- Pager integration (no `less`/`more` invocation)
- Fuzzy-matching or "did you mean?" suggestions for unknown topics (that belongs in ticket bc89e0a0's dispatcher)
- Changes to how `apm <subcommand> --help` works (clap-native help is untouched)
- Content for the `config`, `workflow`, or `ticket` help topics (sibling tickets d486d183, 7ba021e8, 14214305)
- Grouping commands by category (Setup / Workflow / etc.) — top-level commands are sorted alphabetically; subcommands appear in declaration order
- Reconciling output style with the `config`/`workflow`/`ticket` topics — the divergence is intentional: this ticket uses flat word-wrapped lines because commands form a hierarchy, while schema topics use column-aligned tables because they describe flat key-value fields; `bc89e0a0`'s `render_overview()` source should include a comment acknowledging this distinction

### Approach

**File to change:** `apm/src/cmd/help.rs` — replace the body of `render_commands()` only. No other files change.

**Key dependencies:**
- `use clap::CommandFactory;` — brings `Cli::command()` into scope (clap 4 derive feature, already in Cargo.toml)
- `use crate::Cli;` — the top-level parser struct defined in `main.rs`

**Output style decision:** `apm help commands` uses flat word-wrapped lines rather than the column-aligned table format that `config`/`workflow`/`ticket` topics use (via the `help_schema` infra from ticket 069c3403). The divergence is intentional: commands form a hierarchy (command → positionals → flags → subcommands) that does not fit a key-value table layout. `bc89e0a0`'s `render_overview()` source should include a brief comment noting that commands and schema topics use different layouts due to differing data shapes.

**Algorithm — `render_commands()`:**

1. Call `crate::Cli::command()` to obtain the root `clap::Command`.
2. Extract its subcommands via `root.get_subcommands()`. These are the 30+ top-level commands.
3. Collect into a `Vec`, then sort by `cmd.get_name()` alphabetically. This sort applies to top-level commands only; nested subcommands are not re-sorted and appear in declaration order as returned by `cmd.get_subcommands()`.
4. For each top-level command, call a helper `fn render_one(cmd: &clap::Command, prefix: &str) -> String` that:
   a. Skips if `cmd.is_hide_set()`.
   b. Builds a usage line: `{prefix}{name} {positional_summary}` where `positional_summary` lists positional arg value-names in angle brackets (e.g. `<TITLE>`, `[ID]`).
   c. Appends the about string on the next line, indented 2 spaces, word-wrapped at 100 columns.
   d. For each non-hidden argument from `cmd.get_arguments()`:
      - Skip if `arg.is_hide_set()` or if the arg id is `"help"` or `"version"` (clap auto-generated).
      - For positionals: already covered in the usage line; skip here.
      - For flags/options: format as `  -s, --long-name <VALUE>   help text (default: X)` (or `  --long-name` if no short; omit `<VALUE>` if it's a boolean flag; omit default clause if none).
      - Wrap the complete formatted flag line at 100 characters — the `(default: X)` annotation is part of the visible output and counts toward the limit; continuation lines align under the help text start.
   e. If the command has subcommands (`cmd.get_subcommands().next().is_some()`), recurse: for each non-hidden subcommand call `render_one(sub, &format!("{prefix}{name} "))` and indent the block by 2 additional spaces. Subcommands are not re-sorted.
5. Join all rendered blocks with a blank line separator.
6. Prepend a one-line header: `"Commands\n========\n"`.
7. Return the assembled String.

**StyledStr conversion:** In clap 4, `get_about()`, `get_help()`, and `get_value_names()` return `Option<&StyledStr>`. Call `.to_string()` to get plain text (StyledStr implements Display).

**OsStr conversion:** `get_default_values()` returns `&[OsStr]`. Use `.to_string_lossy()` on each element.

**Positional arg detection:** `arg.is_positional()` returns true when the arg has no `--long` and no `-s` short flag.

**Optional positionals:** `arg.get_required()` (or check `arg.get_num_args()`) distinguishes required vs optional positionals; wrap optional value-names in `[]`, required in `<>`.

**Nested subcommands:** Only `Epic` currently has nested subcommands (`epic new`, `epic close`, `epic list`, `epic show`, `epic set`). The recursive approach handles any future nesting automatically.

**100-column wrapping helper:** A small `fn wrap(text: &str, indent: usize, max_width: usize) -> String` that inserts newlines at word boundaries. Use `textwrap` crate if already in the dependency tree, otherwise implement a simple word-wrap loop (the crate is not required — the wrapping logic is a handful of lines). The wrap limit applies to the complete rendered line including any `(default: X)` suffix.

**Implementation order:**
1. Add `use clap::CommandFactory;` and `use crate::Cli;` at the top of `help.rs`
2. Implement the `wrap()` helper
3. Implement `render_one(cmd, prefix)` recursively
4. Replace the stub body of `render_commands()` with a call that builds the root command, iterates top-level commands sorted alphabetically, and joins the results
5. `cargo build` to confirm it compiles
6. `apm help commands` smoke test against all 13 acceptance criteria

### Open questions


### Amendment requests

- [x] Mandate alphabetical sort order for top-level commands. Nested subcommands follow the parent's order. Currently the spec says "alphabetical or Cli-enum order" — pick alphabetical and lock it in an AC.
- [x] Specify whether inline `(default: …)` annotations count toward the 100-char wrap limit. Recommend: yes, they count — the wrap is about visible output width.
- [x] AC must explicitly list `epic new` and `epic close` (and any other current nested subcommands) to lock in that recursive coverage works on the real Cli definition.
- [x] Cross-cutting style decision: `apm help commands` produces flat lines with word-wrapping, while `apm help config|workflow|ticket` produce a column-aligned table from the auto-derive infra. Decide before implementing: (a) reconcile so all four topics share a style, or (b) document the divergence intentionally in `bc89e0a0`'s overview output ("commands vs config schemas use different layouts because the data shapes differ"). Either is defensible; pick one.

### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-04-28T19:27Z | — | new | philippepascal |
| 2026-04-28T19:32Z | new | groomed | philippepascal |
| 2026-04-28T19:37Z | groomed | in_design | philippepascal |
| 2026-04-28T19:42Z | in_design | specd | claude-0428-1937-c708 |
| 2026-04-28T20:17Z | specd | ammend | philippepascal |
| 2026-04-28T20:17Z | ammend | in_design | philippepascal |
| 2026-04-28T20:21Z | in_design | specd | claude-0428-2017-f9f0 |
| 2026-04-29T03:42Z | specd | ready | philippepascal |
