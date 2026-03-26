+++
id = 10
title = "Add apm agents command (print agent instructions)"
state = "specd"
priority = 5
effort = 1
risk = 1
created = "2026-03-25"
updated = "2026-03-25"
+++

## Spec

### Problem

The `apm.agents.md` file contains instructions for agents on how to work within
the APM workflow. Agents need a way to read these instructions via the CLI without
knowing the file path. The `[agents] instructions` field in `apm.toml` points to
this file; `apm agents` should print its contents.

### Acceptance criteria

- [ ] `apm agents` prints the contents of the file referenced by `[agents] instructions` in `apm.toml`
- [ ] If `instructions` is not set in `apm.toml`, prints a message indicating no instructions file is configured
- [ ] If the file is missing, prints a clear error (not a panic)

### Out of scope

- Editing the instructions file via CLI
- Multiple instructions files

### Approach

New subcommand `apm agents` in `apm/src/cmd/agents.rs`. Read `config.agents.instructions`,
resolve relative to repo root, print with `std::fs::read_to_string`.

## History

| Date | Actor | Transition | Note |
|------|-------|------------|------|
| 2026-03-25 | manual | new → specd | |
