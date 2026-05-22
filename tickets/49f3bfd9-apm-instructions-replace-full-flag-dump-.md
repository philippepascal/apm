+++
id = "49f3bfd9"
title = "apm instructions: replace full flag dump with compact one-liner-per-command summary"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/49f3bfd9-apm-instructions-replace-full-flag-dump-"
created_at = "2026-05-22T08:04:36.768358Z"
updated_at = "2026-05-22T08:08:28.004812Z"
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

How the implementation will work.

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-05-22T08:04Z | — | new | philippepascal |
| 2026-05-22T08:05Z | new | groomed | philippepascal |
| 2026-05-22T08:08Z | groomed | in_design | philippepascal |