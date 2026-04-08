+++
id = "05a79474"
title = "apm init: prompt for project name and description"
state = "closed"
priority = 0
effort = 2
risk = 1
author = "philippepascal"
agent = "74093"
branch = "ticket/05a79474-apm-init-prompt-for-project-name-and-des"
created_at = "2026-03-30T23:56:25.325030Z"
updated_at = "2026-03-31T05:04:46.706975Z"
+++

## Spec

### Problem

apm init generates a [project] section in apm.toml with empty name and description fields. Users must manually edit the file after init to fill these in.

apm init should interactively prompt for project name and description when stdin is a TTY. If the user presses Enter without input, or if stdin is not a TTY (e.g. piped or scripted), the fields are left empty. No input is required — the prompts are optional.

Example generated config with user input:

[project]
name = "apm"
description = "Git-native, agent-first project management tool"

### Acceptance criteria


### Out of scope

- No changes to `apm new`, `apm show`, or any other command
- No support for non-interactive flags (e.g. `--name`/`--description`) at this time
- No changes to the config schema — `description` already exists in `ProjectConfig`

### Approach

In `apm-core/src/init.rs`, the `setup()` function currently derives the project name silently from the directory.
The fix is to add an interactive prompt step before writing `.apm/config.toml`:

1. Check if stdin is a TTY (`std::io::IsTerminal::is_terminal(&std::io::stdin())`). If not, skip prompts and use the directory name with empty description (preserves CI/test behaviour).
2. If stdin is a TTY, print a prompt showing the default name, read a line, and use the trimmed input if non-empty, else the default.
3. Print a description prompt, read a line, use trimmed input (may be empty).
4. Pass `name` and `description` into `default_config()` so they are written into the generated TOML.
5. Update `default_config()` to include `description = "..."` in the `[project]` section.
6. Tests: verify that calling `setup()` in a non-TTY context (the default in tests) produces a config with the directory name and empty description.

### Open questions



### Amendment requests
## History

| When | From | To | By |
|------|------|----|----|
| 2026-03-30T23:56Z | — | new | philippepascal |
| 2026-03-30T23:56Z | new | in_design | philippepascal |
| 2026-03-30T23:59Z | in_design | specd | claude-0330-2356-b7f2 |
| 2026-03-31T00:10Z | specd | ready | apm |
| 2026-03-31T00:10Z | ready | in_progress | philippepascal |
| 2026-03-31T00:12Z | in_progress | implemented | claude-0331-0010-w7k2 |
| 2026-03-31T00:19Z | implemented | accepted | apm-sync |
| 2026-03-31T05:04Z | accepted | closed | apm-sync |