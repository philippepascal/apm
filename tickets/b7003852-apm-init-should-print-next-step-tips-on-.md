+++
id = "b7003852"
title = "apm init should print next-step tips on completion"
state = "implemented"
priority = 0
effort = 2
risk = 1
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/b7003852-apm-init-should-print-next-step-tips-on-"
created_at = "2026-04-24T06:28:19.582833Z"
updated_at = "2026-04-24T07:39:18.242594Z"
+++

## Spec

### Problem

After `apm init` completes, the only output is a bare `"apm initialized."` line (`apm/src/cmd/init.rs:62`). New users receive no cue on what to do next: whether to commit the generated `.apm/` files, how to create a first ticket, that a web UI (`apm-server`) exists, or where to find the full command reference.

The desired behaviour is to print a short tips block immediately after `"apm initialized."` that surfaces the four most useful next steps. The block should be suppressed automatically when stdout is not a TTY (so CI pipelines stay clean) and should also respect a `--quiet` flag, consistent with the pattern already established by `apm sync`.

### Acceptance criteria

- [x] After `apm init` on a TTY without `--quiet`, a tips block is printed after `"apm initialized."` containing suggestions to commit `.apm/` files, run `apm new`, try `apm-server`, and check `apm --help`
- [x] When stdout is not a TTY, the tips block is suppressed and only `"apm initialized."` is printed
- [x] `apm init --quiet` suppresses the tips block even when run on a TTY
- [x] `apm init --quiet` does not suppress `"apm initialized."` (the confirmation line always prints)
- [x] `apm init --quiet` is accepted by the CLI without error (the flag is wired up end-to-end)
- [x] `apm init --help` documents the `--quiet` flag with a description

### Out of scope

- Coloured or styled terminal output (no colour library is in use; plain text only)
- Tips after `apm init --migrate` (migration is a distinct workflow, not initial setup)
- Any flag beyond `--quiet` (e.g. `--no-tips`, `--verbose`)
- Changes to the messages printed before `"apm initialized."` (the setup log lines)
- Changes to `apm-core`

### Approach

Two files change: `apm/src/main.rs` and `apm/src/cmd/init.rs`. No changes to `apm-core`.

**`apm/src/main.rs` — Init subcommand struct**

Add a `quiet` field using the same pattern as the `Sync` subcommand (lines 391-393):

```rust
/// Suppress non-error output
#[arg(long)]
quiet: bool,
```

In the match arm that calls `cmd::init::run()`, pass the new `quiet` argument.

**`apm/src/cmd/init.rs` — `run()` function**

- Add `quiet: bool` to the `run()` signature.
- After the existing `println!("apm initialized.");` (line 62), append:

```rust
if std::io::stdout().is_terminal() && !quiet {
    println!();
    println!("Next steps:");
    println!("  * Commit the config:   git add .apm/ && git commit -m 'chore: init apm'");
    println!("  * Create a ticket:     apm new");
    println!("  * Open the web UI:     apm-server");
    println!("  * Full CLI reference:  apm --help");
}
```

Use `stdout().is_terminal()` (consistent with `clean.rs`) rather than the existing `is_tty` variable, which checks stdin and is scoped to interactive prompting. `IsTerminal` is already imported in `init.rs` via `use std::io::{self, BufRead, IsTerminal, Write};`, so no new import is needed.

**Implementation order:** wire the flag in `main.rs` first (compile check), then add the tips block.

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-04-24T06:28Z | — | new | philippepascal |
| 2026-04-24T07:12Z | new | groomed | philippepascal |
| 2026-04-24T07:14Z | groomed | in_design | philippepascal |
| 2026-04-24T07:18Z | in_design | specd | claude-0424-0714-0008 |
| 2026-04-24T07:25Z | specd | ready | philippepascal |
| 2026-04-24T07:36Z | ready | in_progress | philippepascal |
| 2026-04-24T07:39Z | in_progress | implemented | claude-0424-0736-13f0 |
