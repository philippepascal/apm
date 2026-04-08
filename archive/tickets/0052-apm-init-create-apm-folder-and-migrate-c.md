+++
id = 52
title = "apm init: create .apm/ folder and migrate config"
state = "closed"
priority = 3
effort = 4
risk = 3
author = "claude-0329-1200-a1b2"
agent = "claude-0329-1200-a1b2"
branch = "ticket/0052-apm-init-create-apm-folder-and-migrate-c"
created_at = "2026-03-29T19:11:25.479427Z"
updated_at = "2026-03-30T02:02:46.501095Z"
+++

## Spec

### Problem

`apm init` currently writes project configuration files (`apm.toml`, `apm.agents.md`, `apm.worker.md`) directly to the repository root. This clutters the root directory and mixes APM infrastructure with project source. A dedicated `.apm/` folder would group all APM-managed files in one place, making the layout cleaner and aligning with the convention used by tools like `.github/`.

Existing repos that already have root-level `apm.toml` must keep working without any intervention — a flag-day migration would break every team currently using APM.

### Acceptance criteria

- [x] `apm init` on a fresh repo creates `.apm/config.toml`, `.apm/agents.md`, `.apm/spec-writer.md`, and `.apm/worker.md`; does not create `apm.toml` or `apm.agents.md` at the root
- [x] `CLAUDE.md` import line written by `apm init` references `.apm/agents.md` (i.e. `@.apm/agents.md`)
- [x] `Config::load` tries `.apm/config.toml` first; if absent, falls back silently to `apm.toml` at root — no warning, no error on either path
- [x] An existing repo with only `apm.toml` at root continues to work with all apm commands after upgrading (fallback is permanent, no deadline)
- [x] `apm init --migrate` on a repo that has root-level `apm.toml` moves it to `.apm/config.toml`, moves `apm.agents.md` to `.apm/agents.md` (if present), removes the old root files, updates the `@apm.agents.md` import line in `CLAUDE.md` to `@.apm/agents.md`, and prints each action taken
- [x] `apm init --migrate` is a no-op (with a message) when `.apm/config.toml` already exists
- [x] `apm init --migrate` is a no-op (with a message) when neither `apm.toml` nor `apm.agents.md` exists at root
- [x] `cargo test --workspace` passes

### Out of scope

- Moving `CLAUDE.md` to `.apm/` — it stays at the repository root (Claude Code reads it from there)
- Moving `.git/hooks/` — unchanged
- Moving ticket files in `tickets/` — unchanged
- Migrating `apm.worker.md` during `--migrate` (no existing repos have it at the documented root path)
- Any deprecation warning or deadline for root-level `apm.toml` — the fallback is permanent
- Auto-running migration on `apm init` when a root config already exists

### Approach

**1. Config loading (`apm-core/src/config.rs`)**

Change `Config::load` to check two paths in order:
1. `<repo_root>/.apm/config.toml`
2. `<repo_root>/apm.toml`

Return the first one that exists and parses successfully. If neither exists, return the existing "cannot read" error pointing at `apm.toml` (preserves current error UX for fresh repos that haven't run `apm init`).

**2. `apm init` — fresh repo path (`apm/src/cmd/init.rs`)**

- Create `.apm/` directory
- Write `.apm/config.toml` (same content as current `default_config()`)
- Write `.apm/agents.md` (current `default_agents_md()` content)
- Write `.apm/spec-writer.md` (new template, scaffolded with a `# Spec-writing agent` header and a `_fill in your spec-writing instructions here_` placeholder)
- Write `.apm/worker.md` (current `include_str!("../apm.worker.md")` content)
- Update `ensure_claude_md` import line to `@.apm/agents.md`
- Update `maybe_initial_commit` to stage `.apm/config.toml` and `.gitignore` instead of `apm.toml`

**3. `apm init --migrate` flag**

Add `--migrate` flag to the `init` subcommand. When set:
1. Check if `.apm/config.toml` already exists → print "Already migrated." and exit
2. Check if `apm.toml` exists at root → if not, print "Nothing to migrate." and exit
3. Create `.apm/` directory
4. Move `apm.toml` → `.apm/config.toml`, print `Moved apm.toml → .apm/config.toml`
5. If `apm.agents.md` exists at root: move → `.apm/agents.md`, print `Moved apm.agents.md → .apm/agents.md`
6. In `CLAUDE.md`, replace `@apm.agents.md` with `@.apm/agents.md` if present, print `Updated CLAUDE.md`

**4. No changes needed** to any other command — they all go through `Config::load`, which gains the fallback transparently.

## History

| When | From | To | By |
|------|------|----|----|
| 2026-03-29T19:11Z | — | new | claude-0329-1200-a1b2 |
| 2026-03-29T19:36Z | new | in_design | claude-0329-spec-52 |
| 2026-03-29T19:38Z | in_design | specd | claude-0329-spec-52 |
| 2026-03-29T19:42Z | specd | ready | claude-0329-1200-a1b2 |
| 2026-03-29T19:48Z | ready | in_progress | claude-0329-impl-52 |
| 2026-03-29T19:51Z | claude-0329-impl-52 | claude-0329-1200-a1b2 | handoff |
| 2026-03-29T20:16Z | in_progress | implemented | claude-0329-main |
| 2026-03-29T20:19Z | implemented | accepted | claude-0329-main |
| 2026-03-30T02:02Z | accepted | closed | apm-sync |