+++
id = "1fce91bd"
title = "Remove agents.md built-in default"
state = "in_design"
priority = 0
effort = 2
risk = 1
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/1fce91bd-remove-agents-md-built-in-default"
created_at = "2026-05-22T23:22:54.150045Z"
updated_at = "2026-05-23T00:20:39.507349Z"
epic = "ab6e5db7"
target_branch = "epic/ab6e5db7-prompt-management-redesign"
depends_on = ["edb0cf35", "34ad9126", "78eeb755"]
+++

## Spec

### Problem

After T2 (edb0cf35) creates `apm.project.md` and `apm.main-agent.md` and T4/T5 (78eeb755, 34ad9126) rewrite the worker and spec-writer role files, `agents.md` is no longer needed — all of its content has been redistributed into the new role files. The built-in default at `apm-core/src/default/agents/default/agents.md` can be deleted and the code that compiles and writes it can be removed.

Four concrete changes follow: delete the built-in file, remove the `fn default_agents_md()` wrapper (the sole `include_str!` that embeds the file), remove the `write_default` call in `setup()` that writes agents.md to new projects, and update the test that currently asserts the file exists after `apm init`. The `ensure_claude_md` call (which injects `@.apm/agents/default/agents.md` into CLAUDE.md) and the `instructions` key in `default_config` are not changed here — those are covered by T8 (7ef960f2).

### Acceptance criteria

- [ ] `apm-core/src/default/agents/default/agents.md` does not exist in the repository after this ticket merges
- [ ] `fn default_agents_md()` is absent from `apm-core/src/init.rs`
- [ ] The `write_default` call for `agents.md` is absent from `setup()` in `apm-core/src/init.rs`
- [ ] No `include_str!` in any Rust source file references `default/agents/default/agents.md`
- [ ] `setup_creates_expected_files` does not assert that `.apm/agents/default/agents.md` exists
- [ ] `cargo test --workspace` passes with no new failures

### Out of scope

- Updating `ensure_claude_md` to inject different files into CLAUDE.md — covered by T8 (7ef960f2)
- Changing the `instructions` key in `default_config` to `project` — covered by T8 (7ef960f2) coordinated with T3 (d8e2fa0e)
- Updating the `prompt.rs` test `explain_prefix_shown` that uses `.apm/agents/default/agents.md` as the prefix path — covered by T3 (d8e2fa0e)
- Updating `migrate_flat_agent_files` migration rules — migration of old agents.md files is still valid; T8 handles any follow-on changes
- Updating `apm/src/main.rs` help text that references `agents.md` — covered by bfa41899
- Rewriting the content of any role file (`apm.worker.md`, `apm.spec-writer.md`) — covered by 78eeb755 and 34ad9126

### Approach

Four files change: one deletion and three edits.

**1. Delete `apm-core/src/default/agents/default/agents.md`**

Remove the file from the repository. This is the only built-in default being deleted; the other defaults (`apm.spec-writer.md`, `apm.worker.md`) stay.

**2. `apm-core/src/init.rs` — remove `fn default_agents_md()`**

Delete lines 392–394:
```rust
fn default_agents_md() -> &'static str {
    include_str!("default/agents/default/agents.md")
}
```

This removes the sole `include_str!` reference to the deleted file.

**3. `apm-core/src/init.rs` — remove the `write_default` call in `setup()`**

Delete line 142:
```rust
write_default(&agents_default_dir.join("agents.md"), default_agents_md(), ".apm/agents/default/agents.md", &mut messages)?;
```

Leave all surrounding lines intact: the `agents_default_dir` directory creation above it (line 139–141) stays because it is still needed for `apm.spec-writer.md` and `apm.worker.md`; the `ensure_claude_md` call below it (line 160) stays and is unchanged (T8 handles that).

**4. `apm-core/src/init.rs` — update `setup_creates_expected_files` test**

In the test at line 661, change:
```rust
assert!(tmp.path().join(".apm/agents/default/agents.md").exists());
```
to:
```rust
assert!(!tmp.path().join(".apm/agents/default/agents.md").exists());
```

This asserts the file is NOT written by a fresh `apm init`, which is the new expected behaviour.

**5. `apm/tests/e2e.rs` — update the e2e assertion**

Two lines in `full_ticket_lifecycle` (around line 313 and 319) assert that agents.md exists and that CLAUDE.md references it. The CLAUDE.md reference will remain true for now (the `ensure_claude_md` call is unchanged until T8), but the file existence assertion must be removed or inverted:

- Line 313: `assert!(env.root().join(".apm/agents/default/agents.md").exists(), ...)` — change to assert it does NOT exist.
- Line 319: `assert!(claude.contains("@.apm/agents/default/agents.md"), ...)` — leave unchanged; `ensure_claude_md` still injects this import until T8 removes it.

**6. Verify**

Run `cargo test --workspace`. All tests must pass before marking implemented.

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-05-22T23:22Z | — | new | philippepascal |
| 2026-05-22T23:50Z | new | groomed | philippepascal |
| 2026-05-23T00:17Z | groomed | in_design | philippepascal |