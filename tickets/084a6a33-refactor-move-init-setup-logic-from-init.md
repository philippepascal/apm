+++
id = "084a6a33"
title = "refactor: move init setup logic from init.rs into apm-core"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "claude-0330-0245-main"
agent = "110"
branch = "ticket/084a6a33-refactor-move-init-setup-logic-from-init"
created_at = "2026-03-30T14:27:51.779466Z"
updated_at = "2026-03-30T16:36:09.071496Z"
+++

## Spec

### Problem

`init.rs` is the largest file in the CLI crate at 535 lines and conflates two distinct concerns: interactive user-environment setup (Claude settings.json prompts) and repository initialization logic that operates purely on the filesystem and git.

The repo-level functions — default branch detection, config template generation, .gitignore management, CLAUDE.md maintenance, worktree directory creation, initial commit, and config migration — have no dependency on a TTY and no need for user interaction. Living in the CLI crate they cannot be unit-tested without spinning up the full CLI, and they cannot be reused by other tools or future library consumers.

Moving this logic into `apm_core::init` gives the project a clean boundary: `apm-core` owns every filesystem/git operation needed to initialise a repo, and the CLI layer owns only the interactive prompts (`update_claude_settings`, `update_user_claude_settings`) and the call to `apm_core::init::setup()`.

### Acceptance criteria


### Out of scope

Explicit list of what this ticket does not cover.

### Approach

How the implementation will work.

### Open questions



### Amendment requests



## History

| When | From | To | By |
|------|------|----|----|
| 2026-03-30T14:27Z | — | new | claude-0330-0245-main |
| 2026-03-30T16:36Z | new | in_design | philippepascal |