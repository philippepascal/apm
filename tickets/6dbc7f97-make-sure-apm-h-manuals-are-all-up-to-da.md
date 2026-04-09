+++
id = "6dbc7f97"
title = "make sure apm <> -h manuals are all up to date"
state = "implemented"
priority = 0
effort = 2
risk = 1
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/6dbc7f97-make-sure-apm-h-manuals-are-all-up-to-da"
created_at = "2026-04-08T04:04:31.823861Z"
updated_at = "2026-04-09T00:29:16.575441Z"
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

- [x] `apm worktrees -h` does not mention `--add`
- [x] `apm worktrees -h` examples show only `apm worktrees` (list) and `apm worktrees --remove <id>`
- [x] `apm agents -h` short description does not reference `apm.agents.md` by name
- [x] `apm agents -h` long description references the configurable `[agents] instructions` path, not a hardcoded filename
- [x] `apm init -h` lists the correct files created: `config.toml`, `workflow.toml`, `ticket.toml`, `agents.md`, `apm.spec-writer.md`, `apm.worker.md`
- [x] `apm init -h` does not mention `apm.toml` or `apm.agents.md` as files that are created (only as migration sources in the `--migrate` description)
- [x] `cargo build` succeeds after the edits

### Out of scope

- Updating `docs/commands.md` (already accurate)
- Fixing any other commands not listed above
- Adding new flags or changing any runtime behaviour
- Updating `.apm/agents.md` template content

### Approach

All changes are in `apm/src/main.rs`. No runtime logic changes; only string literals in `long_about` and `///` short-description comments.

**Fix 1 -- `apm worktrees` (lines 392-412)**

Remove the paragraph about `--add` and the `--add` example line from `long_about`.

New `long_about`:
```
Manage permanent git worktrees for ticket branches.

APM uses permanent worktrees (in the apm--worktrees/ sibling directory by
default) so that agents can work on a ticket branch without disturbing the
main working tree. These worktrees survive `apm sync` and are reused across
sessions.

Examples:
  apm worktrees              # list all known worktrees
  apm worktrees --remove 42  # remove the worktree for ticket 42
```

**Fix 2 -- `apm agents` (lines 499-508)**

Change the `///` short summary from:
  "Print agent instructions from apm.agents.md"
to:
  "Print agent instructions configured in .apm/apm.toml"

Change the opening sentence of `long_about` from:
  "Print the contents of apm.agents.md to stdout."
to:
  "Print the contents of the instructions file configured under [agents] instructions in .apm/apm.toml."

**Fix 3 -- `apm init` (lines 73-86)**

Replace the file list in `long_about` from:
  * apm.toml      -- project config and state-machine definition
  * apm.agents.md -- agent onboarding instructions

to:
  * config.toml        -- project config
  * workflow.toml      -- state-machine definition
  * ticket.toml        -- ticket template
  * agents.md          -- agent onboarding instructions
  * apm.spec-writer.md -- spec-writer agent manual
  * apm.worker.md      -- worker agent manual

Update the `--migrate` flag `///` comment from:
  "Migrate root-level apm.toml and apm.agents.md to .apm/"
to:
  "Migrate root-level apm.toml -> .apm/config.toml and apm.agents.md -> .apm/agents.md"

After all edits, run `cargo build -p apm` to confirm no compile errors.

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-04-08T04:04Z | — | new | philippepascal |
| 2026-04-08T23:49Z | new | groomed | apm |
| 2026-04-09T00:01Z | groomed | in_design | philippepascal |
| 2026-04-09T00:10Z | in_design | specd | claude-0409-0001-5208 |
| 2026-04-09T00:24Z | specd | ready | apm |
| 2026-04-09T00:27Z | ready | in_progress | philippepascal |
| 2026-04-09T00:29Z | in_progress | implemented | claude-0409-0027-83e8 |
