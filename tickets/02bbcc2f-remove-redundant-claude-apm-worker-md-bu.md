+++
id = "02bbcc2f"
title = "Remove redundant claude/apm.worker.md built-in default"
state = "in_progress"
priority = 0
effort = 2
risk = 1
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/02bbcc2f-remove-redundant-claude-apm-worker-md-bu"
created_at = "2026-05-22T23:22:45.436649Z"
updated_at = "2026-05-23T03:20:25.729986Z"
epic = "ab6e5db7"
target_branch = "epic/ab6e5db7-prompt-management-redesign"
depends_on = ["78eeb755"]
+++

## Spec

### Problem

apm-core/src/default/agents/claude/apm.worker.md is byte-for-byte identical to apm-core/src/default/agents/default/apm.worker.md — it adds zero value and creates a maintenance burden. After T5 rewrites the default worker.md, this override should be deleted. Changes: (1) delete apm-core/src/default/agents/claude/apm.worker.md; (2) remove const CLAUDE_WORKER_DEFAULT in apm-core/src/start.rs:7; (3) update resolve_builtin_instructions() to remove the ("claude", "worker") => Some(CLAUDE_WORKER_DEFAULT) arm — it will fall through to the default lookup or the per-agent file in the project's .apm/agents/claude/ directory. Integration tests that assert on the claude/worker prompt content will need to be updated.

### Acceptance criteria

- [x] `apm-core/src/default/agents/claude/apm.worker.md` is absent from the repository
- [x] `cargo build --workspace` succeeds with no compile errors (no dangling `include_str!` or undefined constant)
- [x] `cargo test --workspace` passes with no test referencing `CLAUDE_WORKER_DEFAULT`
- [x] `apm init` on a fresh directory does not write `.apm/agents/claude/apm.worker.md`
- [x] `apm prompt --agent claude --role worker` resolves to the same content as `apm prompt --agent default --role worker` when no per-project override exists
- [x] The `worker_md_sync` integration test is removed (it tested a now-deleted invariant)

### Out of scope

- Rewriting the content of `apm.worker.md` — covered by 78eeb755
- Deleting or modifying `apm-core/src/default/agents/claude/apm.spec-writer.md`
- Migrating the project's own `.apm/agents/` directory — covered by 7c5c491d
- Changing what `apm instructions` emits — covered by 4bee5771
- Changing the cascade or resolution algorithm in `build_system_prompt` — covered by d8e2fa0e

### Approach

Four locations change; no new files are added.

**1. Delete the file**
Remove `apm-core/src/default/agents/claude/apm.worker.md`.

**2. `apm-core/src/start.rs`**
- Line 7: delete `const CLAUDE_WORKER_DEFAULT: &str = include_str!("default/agents/claude/apm.worker.md");`
- In `resolve_builtin_instructions()`: remove the `("claude", "worker") => Some(CLAUDE_WORKER_DEFAULT)` match arm. After removal, a claude/worker lookup falls through to whatever arm handles the default worker (or returns `None` if no such arm exists, relying on the cascade to pick up the project's `.apm/agents/default/apm.worker.md`).
- Tests that assert on `super::CLAUDE_WORKER_DEFAULT` (lines ~1353, 1373, 1385, 1396, 1416): switch each reference to the constant that holds the default worker content. Identify that constant by inspecting the remaining arms of `resolve_builtin_instructions`; it is the value that `resolve_builtin_instructions("default", "worker")` returns, or the fallback used by the test setup when no per-agent override is present.

**3. `apm-core/src/init.rs`**
- In `setup()`: remove the `write_default` call that writes `agents/claude/apm.worker.md` (lines ~154–159).
- In the integration test (`setup_creates_expected_files` or `test_setup_creates_all_files`, line ~668): remove the assertion that `.apm/agents/claude/apm.worker.md` exists.

**4. `apm-core/tests/worker_md_sync.rs`**
- Delete the test function `default_and_per_agent_apm_worker_md_are_identical` (lines ~56–118) in its entirety. It tested that the two files were byte-identical — an invariant that disappears with the file. If this is the only test in the file, delete the file; otherwise remove only that function.

**Verification**
Run `cargo test --workspace` — all tests must pass before marking implemented.

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-05-22T23:22Z | — | new | philippepascal |
| 2026-05-22T23:50Z | new | groomed | philippepascal |
| 2026-05-23T00:14Z | groomed | in_design | philippepascal |
| 2026-05-23T00:17Z | in_design | specd | claude-0522-0015-b7f2 |
| 2026-05-23T02:58Z | specd | ready | philippepascal |
| 2026-05-23T03:20Z | ready | in_progress | philippepascal |
