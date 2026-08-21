+++
id = "5297b67e"
title = "apm help and version should work even outside a repo"
state = "ready"
priority = 3
effort = 2
risk = 1
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/5297b67e-apm-help-and-version-should-work-even-ou"
created_at = "2026-08-21T19:59:20.649883Z"
updated_at = "2026-08-21T20:28:25.041184Z"
+++

## Spec

### Problem

The top-level `--version`/`--help` flags already work outside a git repository, because clap intercepts and handles them inside `Cli::parse()` before any of apm's own code runs. But the explicit `apm version` and `apm help [topic]` subcommands do not: `apm/src/main.rs` unconditionally calls `repo_root()` (which shells out to `git rev-parse --show-toplevel`) before dispatching to any subcommand handler, with a single carve-out for the hidden `path-guard` command. Outside a git repository, `repo_root()` bails with "not inside a git repository", so `apm version` and `apm help` never get a chance to run — even though neither needs a repository at all. `cmd::version::run()` only prints compile-time constants, and `cmd::help::run()` only introspects the statically-built `clap::Command` tree returned by `Cli::command()`; neither touches `root`, `.apm/config.toml`, or any ticket data.

This is a rough edge for anyone evaluating apm for the first time (checking `apm version` or reading `apm help` before ever running `git init` or `apm init`), and for any script or CI step that wants to sanity-check the installed apm version from an arbitrary working directory.

### Acceptance criteria

- [ ] `apm version` run from a directory outside any git repository prints the version line and exits 0
- [ ] `apm help` (no topic) run from a directory outside any git repository prints the topic overview and exits 0
- [ ] `apm help commands` run from a directory outside any git repository prints the full command reference and exits 0
- [ ] `apm help config`, `apm help workflow`, and `apm help ticket` each run successfully (exit 0, non-empty output) from outside any git repository
- [ ] `apm help badtopic` run from outside any git repository still fails with the "unknown help topic" error and a non-zero exit code — it fails for that reason, not because a repository is missing
- [ ] `apm version` and `apm help [topic]` behave exactly as before when run from inside a git repository, whether or not apm is initialized there
- [ ] Every other subcommand (e.g. `apm list`, `apm show`, `apm init`) still fails with the existing "not inside a git repository" error when run outside a repository — this ticket does not widen that exemption

### Out of scope

- `apm init` still requires being inside a git repository — it writes `.apm/` under `root` and cannot function without one; this ticket does not change that
- Top-level `apm --version`, `apm -V`, `apm --help`, `apm -h` — these already work outside a repo via clap's built-in flag handling; no change needed
- `apm <subcommand> --help` (e.g. `apm list --help`) — already handled by clap during argument parsing, before any subcommand dispatch; unaffected by this ticket
- Reworking or skipping the logging setup and hash-trip config-validation logic for `version`/`help` in any general way — they are simply never reached for these two commands, the same way they are already never reached for `path-guard`

### Approach

#### main.rs dispatch order

All the logic lives in `apm/src/main.rs`. Today `main()`:

1. Parses `Cli` (`main.rs:1036`).
2. Special-cases `Command::PathGuard` and returns before touching `root` (`main.rs:1040-1043`).
3. Unconditionally computes `let root = repo_root()?;` (`main.rs:1045`), which shells out to `git rev-parse --show-toplevel` and bails with `"not inside a git repository"` on failure.
4. Runs logging setup and the `hash_trip` config-validation check (both keyed off `root`).
5. Matches `cli.command` and dispatches to the per-subcommand handler, including `Command::Version => { cmd::version::run(); Ok(()) }` (`main.rs:1349-1352`) and `Command::Help { topic } => cmd::help::run(topic.as_deref(), Cli::command())` (`main.rs:1353`).

`cmd::version::run()` (`apm/src/cmd/version.rs`) takes no arguments and only prints `env!("CARGO_PKG_VERSION")` / `env!("APM_GIT_DESCRIBE")`. `cmd::help::run(topic, cli_cmd)` (`apm/src/cmd/help.rs`) takes the topic string and a `clap::Command` built from `Cli::command()`; every render function (`render_overview`, `render_commands`, `render_config`, `render_workflow`, `render_ticket`) works off that static command tree or the `schemars`-derived `Config`/`TicketConfig`/`WorkflowConfig` schemas — none of it reads `root`. So both commands can run before step 3.

Fix: widen the early-dispatch block that currently only special-cases `PathGuard` to also cover `Version` and `Help`, and return before `repo_root()` is called:

```rust
fn main() -> Result<()> {
    let cli = Cli::parse();

    // path-guard, version, and help need no git/config/ticket access and must
    // keep working even outside a git repository. Dispatch them before
    // repo_root() so a missing repo never blocks them.
    match &cli.command {
        Command::PathGuard => {
            cmd::path_guard::run();
            return Ok(());
        }
        Command::Version => {
            cmd::version::run();
            return Ok(());
        }
        Command::Help { topic } => {
            return cmd::help::run(topic.as_deref(), Cli::command());
        }
        _ => {}
    }

    let root = repo_root()?;
    ...
```

Match on `&cli.command` (not by value) so `cli.command` is still available for the later `match cli.command { ... }` block further down — `Command::Help { topic }` is cloned out as `topic.as_deref()` via the reference, matching the existing call site's pattern.

Update the two now-unreachable arms further down (`main.rs:1349-1353`) to `unreachable!(...)`, mirroring the existing `Command::PathGuard => unreachable!("handled before repo_root()")` arm at `main.rs:1232`:

```rust
Command::Version => unreachable!("handled before repo_root()"),
Command::Help { .. } => unreachable!("handled before repo_root()"),
```

No changes are needed in `cmd/version.rs`, `cmd/help.rs`, `hash_trip.rs`, or anywhere else — `Help` was already exempt from the hash-trip check (`hash_trip::is_exempt_command`) and `Version` never touches config, so removing both from the `root`-gated path changes no other behavior.

#### Tests

Add e2e tests to `apm/tests/e2e.rs` that invoke the compiled binary (`APM` / the existing `apm()` helper, called with a bare `tempfile::tempdir()` that is never `git init`-ed, so it is guaranteed to sit outside any repository) covering:

- `apm version` exits 0 and stdout contains `"apm "`.
- `apm help` exits 0 and stdout contains the topic overview text (`"Topics:"`).
- `apm help commands` exits 0 and stdout contains a known subcommand name (e.g. `"version"`).
- `apm help config` / `apm help workflow` / `apm help ticket` each exit 0 with non-empty stdout.
- `apm help nonexistent-topic` exits non-zero and stderr contains `"unknown help topic"` (confirms the failure is the topic-validation error, not a repo error).
- A control case, e.g. `apm list`, still exits non-zero with stderr containing `"not inside a git repository"` when run from the same non-repo tempdir — guards against accidentally widening the exemption to other commands.

Existing e2e tests already build a full repo via `Env::setup()`; the new tests do not use `Env` at all since they specifically need to run *outside* a repo — use a fresh `tempfile::tempdir()` directly with the existing free `apm()` helper function.

Run `cargo test --workspace` to confirm nothing else regresses (in particular, `hash_trip::tests` and `apm-core` unit tests are untouched by this change).

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-08-21T19:59Z | — | new | philippepascal |
| 2026-08-21T19:59Z | new | groomed | philippepascal |
| 2026-08-21T20:06Z | groomed | in_design | philippepascal |
| 2026-08-21T20:09Z | in_design | specd | claude |
| 2026-08-21T20:28Z | specd | ready | philippepascal |
