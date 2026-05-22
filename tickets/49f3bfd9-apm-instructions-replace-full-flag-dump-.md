+++
id = "49f3bfd9"
title = "apm instructions: replace full flag dump with compact one-liner-per-command summary"
state = "specd"
priority = 0
effort = 2
risk = 1
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/49f3bfd9-apm-instructions-replace-full-flag-dump-"
created_at = "2026-05-22T08:04:36.768358Z"
updated_at = "2026-05-22T08:10:34.915495Z"
+++

## Spec

### Problem

apm instructions currently delegates to help::render_commands() which outputs every top-level command with its full flag list, positional arguments, and defaults. This produces hundreds of lines — far too verbose for an agent loading it as context. Token cost is the main concern: an agent injecting apm instructions into its system prompt pays for every flag default it will never reference directly.

The preamble already says 'Run `apm <command> --help` for full flag details on any command.' The body should match that intent: one line per command, name and one-line description only.

Target format:
  apm list         List tickets matching optional filters
  apm show         Show full ticket details
  apm new          Create a new ticket
  apm state        Transition a ticket to a new state
  ...

Implementation: replace the render_commands() delegation in apm/src/cmd/instructions.rs with a compact renderer that walks the clap Command tree and prints only get_name() and get_about() per subcommand, aligned in two columns. The --help pointer in the preamble already covers flag discovery. Update the unit test assertions accordingly.

### Acceptance criteria

- [ ] `apm instructions` output contains `apm <name>` prefixed lines for every non-hidden top-level subcommand
- [ ] Each line shows only the command name and its one-line description; no flags, positionals, or defaults appear
- [ ] Command lines are two-column aligned: the `apm <name>` token is padded to a consistent width so all descriptions start in the same column
- [ ] Hidden subcommands (e.g. `path-guard`, `_hook`) are absent from the output
- [ ] The preamble (`apm — Agent Project Manager` and the `--help` pointer) is still present and unchanged
- [ ] `apm help commands` output is unchanged — still shows full flag/positional/default detail
- [ ] Unit tests in `instructions.rs` pass and do not assert flag or positional content

### Out of scope

- Changes to `apm help commands` output — that topic keeps its full flag/positional detail\n- Adding `about` text to commands that currently have none (a separate doc-quality concern)\n- Changing the preamble wording\n- Listing sub-subcommands (e.g. `apm epic new`) — top-level commands only

### Approach

#### Changes to `apm/src/cmd/instructions.rs`

Replace the body of `render()` so it no longer calls `super::help::render_commands()`. Instead, add a module-private `render_compact_commands()` function:

```rust
fn render_compact_commands(cli_cmd: &clap::Command) -> String {
    let mut cmds: Vec<&clap::Command> = cli_cmd
        .get_subcommands()
        .filter(|c| !c.is_hide_set())
        .collect();
    cmds.sort_by_key(|c| c.get_name());

    // Compute column width: len("apm ") + longest name + 2-space gap
    let max_name = cmds.iter().map(|c| c.get_name().len()).max().unwrap_or(0);
    let col_width = 4 + max_name; // "apm " prefix

    let mut out = String::new();
    for cmd in &cmds {
        let label = format!("apm {}", cmd.get_name());
        let about = cmd.get_about().map(|a| a.to_string()).unwrap_or_default();
        out.push_str(&format!("  {:<col_width$}  {}\n", label, about));
    }
    out
}
```

Update `render()` to call the new function:

```rust
fn render(cli_cmd: clap::Command) -> String {
    let mut out = String::from(PREAMBLE);
    out.push('\n');
    out.push_str(&render_compact_commands(&cli_cmd));
    out.push('\n');
    out
}
```

#### Unit test updates in `instructions.rs`

Remove or update the existing `render_includes_command_name` test — it still passes because command names do appear, but add explicit coverage for the new shape:

- **`render_compact_has_apm_prefix`** — output contains `"apm foo"` and `"apm bar"`
- **`render_compact_shows_about`** — output contains `"Do foo things"` and `"Do bar things"`
- **`render_compact_no_flags`** — output does not contain `"--verbose"` or `"--count"` (flags from the test fixture)
- **`render_compact_excludes_hidden`** — add a hidden subcommand to `make_test_cmd()`; assert it is absent
- **`render_no_ansi`** — unchanged; continues to assert no ANSI escapes

The existing `make_test_cmd()` helper already has `foo` (with `--verbose` flag) and `bar` (with `--count` flag), so the no-flags assertion exercises real coverage.

#### No changes to `help.rs`

`help::render_commands()` is the implementation behind `apm help commands` and must remain unchanged. The `instructions` module stops calling it; no modifications needed there.

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-05-22T08:04Z | — | new | philippepascal |
| 2026-05-22T08:05Z | new | groomed | philippepascal |
| 2026-05-22T08:08Z | groomed | in_design | philippepascal |
| 2026-05-22T08:10Z | in_design | specd | claude-0522-0808-6398 |
