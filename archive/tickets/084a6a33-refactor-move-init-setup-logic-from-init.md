+++
id = "084a6a33"
title = "refactor: move init setup logic from init.rs into apm-core"
state = "closed"
priority = 0
effort = 4
risk = 2
author = "claude-0330-0245-main"
agent = "82052"
branch = "ticket/084a6a33-refactor-move-init-setup-logic-from-init"
created_at = "2026-03-30T14:27:51.779466Z"
updated_at = "2026-03-30T18:07:53.252930Z"
+++

## Spec

### Problem

`init.rs` is the largest file in the CLI crate at 535 lines and conflates two distinct concerns: interactive user-environment setup (Claude settings.json prompts) and repository initialization logic that operates purely on the filesystem and git.

The repo-level functions — default branch detection, config template generation, .gitignore management, CLAUDE.md maintenance, worktree directory creation, initial commit, and config migration — have no dependency on a TTY and no need for user interaction. Living in the CLI crate they cannot be unit-tested without spinning up the full CLI, and they cannot be reused by other tools or future library consumers.

Moving this logic into `apm_core::init` gives the project a clean boundary: `apm-core` owns every filesystem/git operation needed to initialise a repo, and the CLI layer owns only the interactive prompts (`update_claude_settings`, `update_user_claude_settings`) and the call to `apm_core::init::setup()`.

### Acceptance criteria

- [x] `apm_core::init` is a public module in `apm-core` with no dependency on `std::io` interactive prompts or `serde_json`
- [x] `apm_core::init::setup(root)` creates `tickets/` if it does not exist
- [x] `apm_core::init::setup(root)` creates `.apm/config.toml` if it does not exist
- [x] `apm_core::init::setup(root)` creates `.apm/agents.md` if it does not exist
- [x] `apm_core::init::setup(root)` creates `.apm/spec-writer.md` if it does not exist
- [x] `apm_core::init::setup(root)` creates `.apm/worker.md` if it does not exist
- [x] `apm_core::init::setup(root)` is idempotent — calling it twice does not overwrite existing files
- [x] `apm_core::init::setup(root)` adds `tickets/NEXT_ID` to `.gitignore`, creating the file if absent
- [x] `apm_core::init::setup(root)` creates `CLAUDE.md` with `@.apm/agents.md` if the file does not exist
- [x] `apm_core::init::setup(root)` prepends `@.apm/agents.md` to an existing `CLAUDE.md` that does not already contain it
- [x] `apm_core::init::setup(root)` creates the worktrees directory from the generated config
- [x] `apm_core::init::setup(root)` creates an initial git commit (staging `.apm/config.toml` and `.gitignore`) when the repo has no commits
- [x] `apm_core::init::detect_default_branch(root)` returns the name of the current HEAD branch
- [x] `apm_core::init::detect_default_branch(root)` returns `"main"` when git fails or produces no output
- [x] `apm_core::init::migrate(root)` moves `apm.toml` → `.apm/config.toml` and `apm.agents.md` → `.apm/agents.md`
- [x] `apm_core::init::migrate(root)` updates `@apm.agents.md` to `@.apm/agents.md` in `CLAUDE.md` if present
- [x] `apm_core::init::migrate(root)` prints "Already migrated." and exits cleanly when `.apm/config.toml` already exists
- [x] `apm init` produces the same output and filesystem result as before this refactor
- [x] `cargo test --workspace` passes with unit tests covering `setup`, `detect_default_branch`, `migrate`, and `ensure_gitignore` in `apm_core::init`

### Out of scope

- Changing the behaviour of `apm init` — this is a pure refactor; no new features
- Moving `update_claude_settings()` or `update_user_claude_settings()` to apm-core (they read stdin and belong in the CLI)
- Moving `warn_if_settings_untracked()` to apm-core (it is a UX warning, not repo setup)
- Changing the config template format or the default workflow states
- Adding a `--dry-run` flag or any other new CLI options
- Removing the `--migrate` subcommand behaviour

### Approach

**New file: `apm-core/src/init.rs`**

Move the following functions verbatim from `apm/src/cmd/init.rs`, adjusting visibility to `pub`:

- `detect_default_branch(root: &Path) -> String` — no changes needed
- `default_config(name: &str, default_branch: &str) -> String` — no changes needed; keep the `#[cfg(target_os)]` `default_log_file` helper alongside it
- `ensure_gitignore(path: &Path) -> Result<()>` — change signature from `&PathBuf` to `&Path`
- `ensure_claude_md(root: &Path, agents_path: &str) -> Result<()>` — no changes needed
- `maybe_initial_commit(root: &Path) -> Result<()>` — no changes needed
- `migrate(root: &Path) -> Result<()>` — renamed from `run_migrate`; same body

Add a top-level `pub fn setup(root: &Path) -> Result<()>` that contains the repo-setup steps currently inlined in `apm/src/cmd/init.rs::run()`:
1. Create `tickets/`
2. Create `.apm/` and write the four files (`config.toml`, `agents.md`, `spec-writer.md`, `worker.md`) if absent
3. Call `ensure_claude_md(root, ".apm/agents.md")`
4. Call `ensure_gitignore(&root.join(".gitignore"))`
5. Call `maybe_initial_commit(root)`
6. Call `ensure_worktrees_dir(root)` (moved from CLI as well)

The `default_agents_md()` helper uses `include_str!("../../../apm.agents.md")` (relative to `apm/src/cmd/init.rs`). In `apm-core/src/init.rs` the path becomes `include_str!("../../apm.agents.md")`. Verify the relative path at compile time.

The `include_str!("../apm.worker.md")` for `worker.md` references `apm/src/apm.worker.md`. Options:
- Copy `apm.worker.md` to `apm-core/src/apm.worker.md` and use `include_str!("apm.worker.md")`
- Or keep the file in `apm/src/` and use a cross-crate path `include_str!("../../apm/src/apm.worker.md")` from `apm-core/src/init.rs`

Use the first option (copy into `apm-core/src/`) to keep `apm-core` self-contained. Delete the copy in `apm/src/` only if nothing else references it.

**Update `apm-core/src/lib.rs`**

Add `pub mod init;`

**Update `apm/src/cmd/init.rs`**

Replace the bodies of `run()` and `run_migrate()` with calls to `apm_core::init::setup(root)?` and `apm_core::init::migrate(root)?`. Delete all moved functions from `init.rs`. Keep `update_claude_settings`, `update_user_claude_settings`, `warn_if_settings_untracked`, `APM_ALLOW_ENTRIES`, `APM_USER_ALLOW_ENTRIES`.

**Cargo.toml changes**

`apm-core` does not currently use `std::process::Command` via a crate dep — this is stdlib so no new deps needed. No Cargo.toml changes expected.

**Tests in `apm-core/src/init.rs`**

Add a `#[cfg(test)]` block using `tempfile::TempDir`. Each test creates a bare temp dir, runs `git init`, and calls the function under test:
- `detect_default_branch` on a fresh repo (expect "main" or current branch)
- `detect_default_branch` with a non-git dir (expect "main")
- `ensure_gitignore` creates the file when absent
- `ensure_gitignore` appends only the missing entry when file exists
- `ensure_gitignore` is idempotent
- `setup` creates all expected files and dirs
- `migrate` moves files and updates CLAUDE.md import
- `migrate` is a no-op when already migrated

**Order of steps**
1. Create `apm-core/src/init.rs` with moved functions + `setup()`
2. Update `apm-core/src/lib.rs`
3. Update `apm/src/cmd/init.rs`
4. Run `cargo test --workspace` and fix any compile errors
5. Delete temp spec files from repo root before committing

### Open questions



### Amendment requests
## History

| When | From | To | By |
|------|------|----|----|
| 2026-03-30T14:27Z | — | new | claude-0330-0245-main |
| 2026-03-30T16:36Z | new | in_design | philippepascal |
| 2026-03-30T16:40Z | in_design | specd | claude-0330-1640-spec7 |
| 2026-03-30T16:53Z | specd | ready | philippepascal |
| 2026-03-30T17:26Z | ready | in_progress | philippepascal |
| 2026-03-30T17:30Z | in_progress | implemented | claude-0330-1726-83a0 |
| 2026-03-30T18:04Z | implemented | accepted | philippepascal |
| 2026-03-30T18:07Z | accepted | closed | apm-sync |