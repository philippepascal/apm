+++
id = "7ef960f2"
title = "Update apm init for new file structure"
state = "in_design"
priority = 0
effort = 4
risk = 3
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/7ef960f2-update-apm-init-for-new-file-structure"
created_at = "2026-05-22T23:23:20.147068Z"
updated_at = "2026-05-23T01:47:54.728220Z"
epic = "ab6e5db7"
target_branch = "epic/ab6e5db7-prompt-management-redesign"
depends_on = ["edb0cf35", "d8e2fa0e", "02bbcc2f", "1fce91bd"]
+++

## Spec

### Problem

`apm init` (`apm-core/src/init.rs setup()`) was designed around the old monolithic `agents.md` file structure. With the prompt management redesign (epic ab6e5db7), agent instruction files are split into three composed layers: dynamic APM knowledge from `apm instructions`, project context from `apm.project.md`, and role-specific instructions from role files. Four sibling tickets restructure those files — T2 (edb0cf35) creates the new built-in defaults, T3 (d8e2fa0e) renames the `[agents] instructions` config key to `project`, T6 (02bbcc2f) removes the redundant `claude/apm.worker.md` built-in, and T7 (1fce91bd) deletes the `agents.md` built-in.

This ticket wires those changes into `init.rs`: `setup()` must write the two new files instead of the old one, stop writing the redundant `claude/apm.worker.md`, inject the correct `@` imports into CLAUDE.md, and emit `project = ...` in the generated config. `migrate_flat_agent_files` must additionally handle existing projects that still reference `agents.md` — both in CLAUDE.md `@` imports and in config.toml `instructions = ...` keys — upgrading them to the new paths and key name in a single migration pass.

### Acceptance criteria

- [ ] `apm init` creates `.apm/agents/default/apm.project.md` using the built-in default template from T2 (edb0cf35)
- [ ] `apm init` creates `.apm/agents/default/apm.main-agent.md` using the built-in default from T2 (edb0cf35)
- [ ] `apm init` does not create `.apm/agents/default/agents.md`
- [ ] `apm init` does not create `.apm/agents/claude/apm.worker.md`
- [ ] `apm init` does NOT create `.apm/agents/claude/apm.spec-writer.md`
- [ ] A freshly initialized CLAUDE.md contains `@.apm/agents/default/apm.project.md` and `@.apm/agents/default/apm.main-agent.md` and does not contain `@.apm/agents/default/agents.md`
- [ ] The generated `config.toml` contains `project = ".apm/agents/default/apm.project.md"` in `[agents]` and does not contain `instructions = ".apm/agents/default/agents.md"`
- [ ] Running `apm init` on a project whose CLAUDE.md contains `@.apm/agents/default/agents.md` replaces that line with both new `@` imports
- [ ] Running `apm init` on a project whose `config.toml` has `instructions = ".apm/agents/default/agents.md"` rewrites it to `project = ".apm/agents/default/apm.project.md"`
- [ ] `cargo test --workspace` passes with all init tests updated to reflect the new structure

### Out of scope

- Creating the content of `apm.project.md` or `apm.main-agent.md` built-in defaults — covered by edb0cf35
- Deleting `apm-core/src/default/agents/default/agents.md` from the repository — covered by 1fce91bd
- Removing `CLAUDE_WORKER_DEFAULT` constant and its `resolve_builtin_instructions` arm — covered by 02bbcc2f
- Adding `project: Option<PathBuf>` to `AgentsConfig` or adding `effective_project()` — covered by d8e2fa0e
- Updating `apm instructions` or `apm prompt` CLI help text — covered by bfa41899
- Migrating this project's own `.apm/agents/` directory and CLAUDE.md — covered by 7c5c491d
- Updating `migrate_flat_agent_files` for the old pre-`agents/default/` flat paths (`.apm/agents.md` → `.apm/agents/default/agents.md`) — that logic already exists; this ticket only extends it to handle the next migration step

### Approach

All changes are in `apm-core/src/init.rs` and `apm/tests/e2e.rs`. By the time this ticket is implemented, T7 has already removed `default_agents_md()` and its `write_default` call for `agents.md`, and T6 has already removed the `claude/apm.worker.md` `write_default` call.

#### Step 1 — setup(): write new files and remove spec-writer write_default

**Add new write_default calls** — after the existing `write_default` calls for `apm.spec-writer.md` and `apm.worker.md` (lines ~143–144), add two new calls using `include_str!` inline — consistent with the existing pattern:

```rust
write_default(
    &agents_default_dir.join("apm.project.md"),
    include_str!("default/agents/default/apm.project.md"),
    ".apm/agents/default/apm.project.md",
    &mut messages,
)?;
write_default(
    &agents_default_dir.join("apm.main-agent.md"),
    include_str!("default/agents/default/apm.main-agent.md"),
    ".apm/agents/default/apm.main-agent.md",
    &mut messages,
)?;
```

These files are created by T2 (edb0cf35). Do NOT add constants to `start.rs` — use `include_str!` directly.

**Remove `claude/apm.spec-writer.md` write_default** — T4 (34ad9126) deletes `apm-core/src/default/agents/claude/apm.spec-writer.md`, so the `include_str!` in `setup()` at lines ~148–153 will not compile after T4 lands. Remove that entire `write_default` block. If `agents_claude_dir` is only referenced by this block (verify by inspecting surrounding code), remove its declaration and the directory-creation call that references it too — mirroring what T6 (02bbcc2f) does for `claude/apm.worker.md`.

#### Step 2 — ensure_claude_md(): accept multiple paths

Change the signature from `(root, agents_path: &str, ...)` to `(root, agents_paths: &[&str], ...)`. Rewrite the body:

1. Collect the paths that are not already in the file (check `contents.contains(&format!("@{p}"))` for each).
2. If none are absent, return `Ok(())`.
3. If some are absent, prepend them (in slice order) to the file content — one per line followed by `\n` — then `\n` separator before existing content if the file existed and was non-empty.
4. If the file did not exist, write only the absent paths.
5. Push a message: `"Updated CLAUDE.md (added ... import)"` or `"Created CLAUDE.md."`.

Update the call site in `setup()` (line ~160):
```rust
ensure_claude_md(root, &[
    ".apm/agents/default/apm.project.md",
    ".apm/agents/default/apm.main-agent.md",
], &mut messages)?;
```

#### Step 3 — default_config(): rename key

In the `[agents]` section of the format string, replace:
```toml
instructions = ".apm/agents/default/agents.md"
```
with:
```toml
project = ".apm/agents/default/apm.project.md"
```

#### Step 4 — migrate_flat_agent_files(): extend both rewrite tables

**CLAUDE.md (`path_rewrites`)** — add one entry after the existing entries:
```rust
("@.apm/agents/default/agents.md",
 "@.apm/agents/default/apm.project.md\n@.apm/agents/default/apm.main-agent.md"),
```
Place this AFTER `("@.apm/agents.md", "@.apm/agents/default/agents.md")` so the cascade works: a project with `@.apm/agents.md` gets migrated to `@.apm/agents/default/agents.md` by the first rule, and then immediately migrated again to the two new `@` imports by the new rule — both in a single call.

**config.toml / workflow.toml (`instructions_rewrites`)** — add one entry after the existing entries:
```rust
("instructions = \".apm/agents/default/agents.md\"",
 "project = \".apm/agents/default/apm.project.md\""),
```
Same cascade reasoning: a project with `instructions = ".apm/agents.md"` gets its path updated by the existing rule, then the key is renamed by this new rule, all in one pass.

#### Step 5 — Update unit tests in init.rs

**`setup_creates_expected_files`** (line ~652):
- Add: `assert!(tmp.path().join(".apm/agents/default/apm.project.md").exists());`
- Add: `assert!(tmp.path().join(".apm/agents/default/apm.main-agent.md").exists());`
- Add: `assert!(!tmp.path().join(".apm/agents/claude/apm.spec-writer.md").exists());`
- (T7 already flips the `agents.md` assertion to `!exists`; T6 already flips `claude/apm.worker.md` to `!exists`)

**New test `default_config_has_project_key`**:
```rust
let config = default_config("proj", "desc", "main", &[]);
assert!(config.contains("project = \".apm/agents/default/apm.project.md\""));
assert!(!config.contains("instructions = "));
```

**New test `setup_claude_md_contains_new_imports`**: call `setup()`, read CLAUDE.md, assert it contains both `@.apm/agents/default/apm.project.md` and `@.apm/agents/default/apm.main-agent.md`, and does not contain `@.apm/agents/default/agents.md`.

**`setup_migrates_flat_agent_files_to_agents_default`** (line ~939):
- Add: assert CLAUDE.md contains `@.apm/agents/default/apm.project.md`
- Add: assert CLAUDE.md contains `@.apm/agents/default/apm.main-agent.md`
- Add: assert CLAUDE.md does not contain `@.apm/agents/default/agents.md` (the cascade fully migrated it)
- Keep: assert `.apm/agents/default/agents.md` exists (the old file was physically moved there; moving is separate from the CLAUDE.md rewrite)

#### Step 6 — Update e2e assertions in apm/tests/e2e.rs

Around line 313:
- Change `agents.md` existence assertion to assert it does NOT exist. (Note: T7 already does this; verify T7's diff covers this line before touching it.)

Around line 319:
- Change `claude.contains("@.apm/agents/default/agents.md")` to assert it contains both `"@.apm/agents/default/apm.project.md"` and `"@.apm/agents/default/apm.main-agent.md"`.

#### Step 7 — Verify

Run `cargo test --workspace`. All tests must pass.

### Open questions


### Amendment requests

- [ ] AC item 5 is wrong: it says 'apm init still creates .apm/agents/claude/apm.spec-writer.md (content differs from default, kept per-agent)' — but T4 (34ad9126) deletes the built-in apm-core/src/default/agents/claude/apm.spec-writer.md, so the include_str! that setup() uses to write it will not compile after T4 lands. Change AC item 5 to: 'apm init does NOT create .apm/agents/claude/apm.spec-writer.md'.
- [ ] Approach Step 1 only adds new write_default calls; it does not mention removing the existing write_default block for claude/apm.spec-writer.md (init.rs:148-153). Add an explicit step: remove that write_default call and any agents_claude_dir creation that becomes unreferenced, mirroring what T6 does for apm.worker.md.

### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-05-22T23:23Z | — | new | philippepascal |
| 2026-05-22T23:50Z | new | groomed | philippepascal |
| 2026-05-23T00:25Z | groomed | in_design | philippepascal |
| 2026-05-23T00:30Z | in_design | specd | claude-0522-spec-7ef9 |
| 2026-05-23T01:28Z | specd | ammend | philippepascal |
| 2026-05-23T01:47Z | ammend | in_design | philippepascal |