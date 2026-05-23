+++
id = "1fce91bd"
title = "Remove agents.md built-in default"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/1fce91bd-remove-agents-md-built-in-default"
created_at = "2026-05-22T23:22:54.150045Z"
updated_at = "2026-05-23T00:17:40.097157Z"
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

Explicit list of what this ticket does not cover.

### Approach

How the implementation will work.

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-05-22T23:22Z | — | new | philippepascal |
| 2026-05-22T23:50Z | new | groomed | philippepascal |
| 2026-05-23T00:17Z | groomed | in_design | philippepascal |