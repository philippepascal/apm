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

- [ ] `apm_core::init` is a public module in `apm-core` with no dependency on `std::io` interactive prompts or `serde_json`
- [ ] `apm_core::init::setup(root)` creates `tickets/` if it does not exist
- [ ] `apm_core::init::setup(root)` creates `.apm/config.toml` if it does not exist
- [ ] `apm_core::init::setup(root)` creates `.apm/agents.md` if it does not exist
- [ ] `apm_core::init::setup(root)` creates `.apm/spec-writer.md` if it does not exist
- [ ] `apm_core::init::setup(root)` creates `.apm/worker.md` if it does not exist
- [ ] `apm_core::init::setup(root)` is idempotent — calling it twice does not overwrite existing files
- [ ] `apm_core::init::setup(root)` adds `tickets/NEXT_ID` to `.gitignore`, creating the file if absent
- [ ] `apm_core::init::setup(root)` creates `CLAUDE.md` with `@.apm/agents.md` if the file does not exist
- [ ] `apm_core::init::setup(root)` prepends `@.apm/agents.md` to an existing `CLAUDE.md` that does not already contain it
- [ ] `apm_core::init::setup(root)` creates the worktrees directory from the generated config
- [ ] `apm_core::init::setup(root)` creates an initial git commit (staging `.apm/config.toml` and `.gitignore`) when the repo has no commits
- [ ] `apm_core::init::detect_default_branch(root)` returns the name of the current HEAD branch
- [ ] `apm_core::init::detect_default_branch(root)` returns `"main"` when git fails or produces no output
- [ ] `apm_core::init::migrate(root)` moves `apm.toml` → `.apm/config.toml` and `apm.agents.md` → `.apm/agents.md`
- [ ] `apm_core::init::migrate(root)` updates `@apm.agents.md` to `@.apm/agents.md` in `CLAUDE.md` if present
- [ ] `apm_core::init::migrate(root)` prints "Already migrated." and exits cleanly when `.apm/config.toml` already exists
- [ ] `apm init` produces the same output and filesystem result as before this refactor
- [ ] `cargo test --workspace` passes with unit tests covering `setup`, `detect_default_branch`, `migrate`, and `ensure_gitignore` in `apm_core::init`

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