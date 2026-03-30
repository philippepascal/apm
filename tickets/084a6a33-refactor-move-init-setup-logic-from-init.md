+++
id = "084a6a33"
title = "refactor: move init setup logic from init.rs into apm-core"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "claude-0330-0245-main"
agent = "philippepascal"
branch = "ticket/084a6a33-refactor-move-init-setup-logic-from-init"
created_at = "2026-03-30T14:27:51.779466Z"
updated_at = "2026-03-30T16:36:09.071496Z"
+++

## Spec

### Problem

`init.rs` is the largest file at 535 lines and mixes interactive CLI setup with
repository initialization logic that belongs in `apm-core`:

- Default branch detection (`git symbolic-ref`, `git remote show`)
- `apm.toml` config template generation
- Migration logic for old config paths (`.apm/config.toml`)
- `.gitignore` entry management
- Worktree directory creation
- Initial git commit for the tickets directory
- Claude `settings.json` modification (project + user level)

The Claude settings.json modification is arguably a CLI concern (user-environment
setup). Everything else — branch detection, config generation, migration, gitignore
management, initial commit — is repository setup logic that should be testable
independently of the CLI.

Target: `apm_core::init::setup()` handling all repo-level initialization. CLI
`init.rs` handles interactive prompts and settings.json updates, then calls it.

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
| 2026-03-30T14:27Z | — | new | claude-0330-0245-main |
| 2026-03-30T16:36Z | new | in_design | philippepascal |
