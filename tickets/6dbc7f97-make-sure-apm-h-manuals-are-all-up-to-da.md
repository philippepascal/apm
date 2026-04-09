+++
id = "6dbc7f97"
title = "make sure apm <> -h manuals are all up to date"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/6dbc7f97-make-sure-apm-h-manuals-are-all-up-to-da"
created_at = "2026-04-08T04:04:31.823861Z"
updated_at = "2026-04-09T00:01:47.676790Z"
+++

## Spec

### Problem

The `apm -h` and `apm <subcommand> -h` help text (defined as `long_about` / `///` doc comments in `apm/src/main.rs`) has drifted from the actual implementation in three places. `docs/commands.md` is already accurate and serves as the authoritative reference; the code's own help output is the thing that needs updating.

The three stale spots are:

1. **`apm worktrees`** -- The `long_about` block describes a `--add <id>` flag and includes an `--add` example. That flag was removed; only `--remove` exists in the struct.

2. **`apm agents`** -- Both the short summary and the `long_about` opening hardcode `apm.agents.md`. The implementation (`cmd/agents.rs`) reads from the path configured under `[agents] instructions` in `.apm/apm.toml`; there is no hardcoded filename in the runtime code.

3. **`apm init`** -- The `long_about` lists the files created as `apm.toml` and `apm.agents.md`. The actual `apm_core::init::setup` creates `config.toml`, `workflow.toml`, `ticket.toml`, `agents.md`, `apm.spec-writer.md`, and `apm.worker.md` inside `.apm/`. These old names predate the `.apm/` directory migration.

Anyone reading `apm worktrees -h`, `apm agents -h`, or `apm init -h` will see incorrect information.

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
| 2026-04-08T04:04Z | — | new | philippepascal |
| 2026-04-08T23:49Z | new | groomed | apm |
| 2026-04-09T00:01Z | groomed | in_design | philippepascal |