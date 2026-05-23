# Project Context

## What we are building

APM is a Git-native, agent-first project management tool. Each ticket lives on its own `ticket/<id>-<slug>` branch as a Markdown file with TOML frontmatter (between `+++` delimiters). The tool is designed so agent workers can pick up tickets, implement them in isolated git worktrees, and transition state — all driven by the `apm` CLI.

## Tech stack

- Rust (workspace with multiple crates)
- Git (branches and worktrees are the storage layer)
- GitHub (PRs opened automatically by `apm state <id> implemented`)
- GitHub Actions for CI

## Repo structure

```
apm/           CLI binary — subcommands in apm/src/cmd/
apm-core/      Core library — ticket parsing, state machine, prompt building, init scaffolding
apm-server/    Web UI — depends on apm-core
apm-ui/        Frontend assets for apm-server
tickets/       Ticket Markdown files (one per ticket, on its own branch)
archive/       Closed ticket files
testdata/      Fixtures used by tests
```

## Module responsibilities

**`apm-core`** owns everything that is not I/O-bound by the CLI or the web server: ticket parsing and serialisation, the state machine (`apm-core/src/state.rs`), prompt construction (`apm-core/src/start.rs` — `build_system_prompt`), `apm init` scaffolding (`apm-core/src/init.rs`), and instructions generation (`apm-core/src/instructions.rs`). It has no dependency on `clap` or any CLI framework. Unit tests live inline in `apm-core/src/`; integration tests that need a real git repo go in `apm/tests/integration.rs`.

**`apm`** is the CLI binary. It depends on `apm-core` and `clap`. Each subcommand lives in its own file under `apm/src/cmd/`. End-to-end tests are in `apm/tests/e2e.rs`.

**`apm-server`** is the web UI. It depends on `apm-core` and serves the ticket board over HTTP.

## Key technical decisions

- Tickets are stored as Markdown files with TOML frontmatter on per-ticket branches (`ticket/<id>-<slug>`). The branch name is the source of truth for the ticket's identity — renaming the file breaks lookup.
- State machine transitions are defined in `apm.toml` under `[[workflow.states]]`. The `apm state` command enforces valid transitions and auto-commits the History table row to the ticket branch.
- Prompt construction uses a cascade (per-agent file → transition instructions → profile → workers list → built-in default). The `build_system_prompt` function in `apm-core/src/start.rs` assembles the final prompt from three composed layers: dynamic APM system knowledge (from `apm instructions`), project context (`apm.project.md`), and role-specific instructions (the role file).
- Unit tests are inline in `apm-core/src/`; integration tests use temp git repos in `apm/tests/integration.rs`. Run `cargo test --workspace` to verify all tests pass.
