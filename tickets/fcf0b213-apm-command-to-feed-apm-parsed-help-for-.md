+++
id = "fcf0b213"
title = "apm command to feed apm parsed help for agents"
state = "ready"
priority = 0
effort = 2
risk = 1
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/fcf0b213-apm-command-to-feed-apm-parsed-help-for-"
created_at = "2026-05-07T20:41:08.889701Z"
updated_at = "2026-05-22T02:25:42.545011Z"
+++

## Spec

### Problem

Agent instruction files (e.g., `.apm/agents/claude/apm.worker.md`) currently contain manually maintained summaries of available APM commands. These summaries are vague and drift out of sync as commands are added, renamed, or gain new flags. As a result, agents operating from stale instructions may invoke wrong syntax, miss new flags, or apply workarounds for limitations that no longer exist.

The accurate, complete command metadata already exists in the clap command definitions — the same source that powers `apm help commands`. A new `apm instructions` command exposes that metadata as a compact, plain-text guide. Agents can call it at startup or on demand to get an authoritative, always-current reference without relying on hardcoded prose.

### Acceptance criteria

- [ ] `apm instructions` exits 0 and prints output to stdout
- [ ] The output includes every visible top-level command with its one-line description, positional arguments, and flags (including defaults)
- [ ] The output is plain text with no ANSI escape codes
- [ ] The command listing is generated from clap command metadata, not a separately maintained string — adding or modifying a command definition automatically reflects in the output
- [ ] `apm instructions` appears in the output of `apm help commands` (automatically, as a registered command)
- [ ] A brief preamble (1–2 lines) precedes the command listing to orient agents reading the output cold

### Out of scope

- Modifying or replacing existing agent instruction files (`.apm/agents/*/apm.*.md`)
- Auto-injecting `apm instructions` output into agent system prompts or user messages
- Flags or options on the command itself (e.g., `--format`, `--compact`, `--topic`)
- Config/workflow/ticket schema documentation (already covered by `apm help config`, `apm help workflow`, `apm help ticket`)
- Localisation or i18n of the output

### Approach

#### 1. Register the command

In `apm/src/main.rs`, add an `Instructions` variant to the `Command` enum:

```rust
/// Output a compact plain-text guide for agents on how to use apm
Instructions,
```

Add a dispatch arm in the `match cli.command` block:

```rust
Command::Instructions => cmd::instructions::run(Cli::command()),
```

`Cli::command()` is available via the `CommandFactory` import already at the top of `main.rs`.

#### 2. New module `apm/src/cmd/instructions.rs`

Implement `pub fn run(cli_cmd: clap::Command) -> Result<()>` that prints a 2-line preamble then delegates to `super::help::run(Some("commands"), cli_cmd)`.

`help::run` is already `pub` and accepts a clap `Command` by value. Passing `Some("commands")` routes to the existing `render_commands()` path, which walks the clap tree and renders every visible command with its about text, positionals, flags, and defaults — no ANSI codes.

The preamble (printed before delegating) should read:

```
apm — Agent Project Manager
Run `apm <command> --help` for full flag details on any command.
```

#### 3. Wire up the module

In `apm/src/cmd/mod.rs` (or wherever cmd submodules are declared), add `pub mod instructions;`.

#### 4. No changes needed in `help.rs`

The `render_commands()` path already produces ANSI-free output (confirmed by the existing `no_ansi_in_output` test). The new command reuses it as-is; no refactoring of `help.rs` internals is required.

#### 5. Test coverage

Add a unit test in `instructions.rs` that builds a minimal `clap::Command`, calls `run()`, and asserts:
- No panic / no `Err` return
- The preamble string "apm — Agent Project Manager" is present in the output
- At least one command name from the test command tree appears in the output
- No ANSI escape code byte (`\x1b`) appears in the output

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-05-07T20:41Z | — | new | philippepascal |
| 2026-05-21T22:59Z | new | groomed | philippepascal |
| 2026-05-21T23:00Z | groomed | in_design | philippepascal |
| 2026-05-21T23:08Z | in_design | specd | claude-0521-2300-c320 |
| 2026-05-22T02:25Z | specd | ready | philippepascal |
