+++
id = "a3c34ddc"
title = "Move shell-discipline rules from apm instructions into Claude role files"
state = "in_progress"
priority = 0
effort = 4
risk = 2
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/a3c34ddc-move-shell-discipline-rules-from-apm-ins"
created_at = "2026-05-30T16:45:44.032054Z"
updated_at = "2026-05-30T18:33:30.981458Z"
+++

## Spec

### Problem

`SHELL_DISCIPLINE_BODY` in `apm-core/src/instructions.rs` contains Claude-specific guidance: it references Claude Code's permission allow-list syntax (`Bash(ls *)`, `Bash(bash *)`), the `--print` headless mode, and parallel tool-use block behaviour. These mechanics are not shared by other agent runtimes. Yet `apm instructions` emits this section to every agent regardless of type, so a non-Claude agent is forced to read rules that do not apply to it. The parallel-tool-batching rule introduced by ticket 753d9ba5 lives in the same constant and carries the same misplacement.

The fix is to move the entire `SHELL_DISCIPLINE_BODY` block out of `apm instructions` and into each Claude role file (`apm.coder.md`, `apm.spec-writer.md`, `apm.main-agent.md`), positioned before the first role-specific section so it appears early in the agent's context. `apm instructions` is left with content that is genuinely agent-agnostic: state machine, ticket format, session identity, and command reference. Non-Claude agents simply stop receiving guidance that was never relevant to them.

### Acceptance criteria

- [x] `apm instructions` output does not contain a `## Shell Discipline` heading
- [x] `apm instructions` output does not contain the text "Do not batch tool calls in parallel"
- [x] `apm instructions` output does not contain `&&` as shell-discipline guidance
- [ ] `apm prompt` output for a `claude/coder` ticket contains `## Shell Discipline` and the no-chaining rule
- [ ] `apm prompt` output for a `claude/spec-writer` ticket contains `## Shell Discipline`
- [ ] The `## Shell Discipline` section appears before the first role-specific section in `apm.coder.md`
- [ ] The `## Shell Discipline` section appears before the first role-specific section in `apm.spec-writer.md`
- [ ] The `## Shell Discipline` section appears before the first role-specific section in `apm.main-agent.md`
- [ ] `apm-core/src/default/agents/claude/apm.coder.md` is byte-identical to `.apm/agents/claude/apm.coder.md`
- [ ] `apm-core/src/default/agents/claude/apm.spec-writer.md` is byte-identical to `.apm/agents/claude/apm.spec-writer.md`
- [ ] `apm-core/src/default/agents/claude/apm.main-agent.md` is byte-identical to `.apm/agents/claude/apm.main-agent.md`
- [ ] `cargo test --workspace` passes

### Out of scope

- Reformatting or restructuring any other section of `apm instructions`
- Changes to the role-file cascade logic in `build_system_prompt_body`
- Non-Claude agent files (`.apm/agents/<other>/`)
- The `CLAUDE.md` `@`-import pattern — rules go directly into role files, not a shared imported file
- Adding shell-discipline rules to any agent outside of the three Claude role files listed above

### Approach

Six files change; every edit to a default template must be mirrored in the corresponding `.apm/` project copy (or the `worker_md_sync` tests will fail).

#### 1. Remove shell discipline from `apm-core/src/instructions.rs`

- Delete the `SHELL_DISCIPLINE_BODY` static constant.
- Remove the three-line block in `generate()` that pushes `## Shell Discipline` and the constant (the comment `// 3. Shell discipline` through the trailing `out.push('\n')` call).
- Renumber the remaining step comments: old step 4 (Session Identity) → step 3; old step 5 (Command Reference) → step 4.
- Update three existing tests:
  - `generate_no_role_contains_all_sections` — remove the `assert!(out.contains("## Shell Discipline"))` assertion.
  - `generate_no_role_sections_in_order` — remove `pos_sd`, and the two assertions that reference it.
  - `generate_role_independent_sections` — remove the `## Shell Discipline` and `git -C` assertions.
- Add a new test `shell_discipline_absent_from_instructions` that calls `generate()` with no config and asserts `!out.contains("## Shell Discipline")` and `!out.contains("Do not batch tool calls in parallel")`.

#### 2. Add shell-discipline section to `apm.coder.md` (both copies)

Files: `apm-core/src/default/agents/claude/apm.coder.md` and `.apm/agents/claude/apm.coder.md`.

- Remove the sentence that says shell discipline is covered by `apm instructions` (the one-line note in the file header).
- Insert a `## Shell Discipline` section immediately after the opening `---` separator, before `## Scope limits`. Content is the full text of the current `SHELL_DISCIPLINE_BODY` constant (all rules and examples, verbatim). Close the section with a `---` separator.

#### 3. Add shell-discipline section to `apm.spec-writer.md` (both copies)

Files: `apm-core/src/default/agents/claude/apm.spec-writer.md` and `.apm/agents/claude/apm.spec-writer.md`.

- Insert the same `## Shell Discipline` section (full `SHELL_DISCIPLINE_BODY` content, bounded by `---` separators) before the existing `## How to save spec sections` section.

#### 4. Add shell-discipline section to `apm.main-agent.md` (both copies)

Files: `apm-core/src/default/agents/claude/apm.main-agent.md` and `.apm/agents/claude/apm.main-agent.md`.

- Insert the same `## Shell Discipline` section before `## What you do`.
- In the Startup sequence, update item 1's parenthetical to remove "shell discipline": change "state machine, ticket format, shell discipline, and command reference" → "state machine, ticket format, session identity, and command reference".

#### 5. Add tests

In `apm-core/src/start.rs` (test module), add:
- `build_system_prompt_coder_contains_shell_discipline`: create a temp dir with no `.apm` config, call `build_system_prompt(tmp, None, "claude", "coder")`, assert the result contains `"## Shell Discipline"`.

In `apm-core/tests/worker_md_sync.rs`, add:
- `default_and_project_apm_main_agent_md_are_identical`: same pattern as the two existing tests, comparing `apm-core/src/default/agents/claude/apm.main-agent.md` against `.apm/agents/claude/apm.main-agent.md`.

The existing `default_and_project_apm_worker_md_are_identical` and `default_and_project_apm_spec_writer_md_are_identical` tests enforce that coder and spec-writer copies stay in sync automatically — no changes needed to those tests other than ensuring the edits are applied to both copies.

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-05-30T16:45Z | — | new | philippepascal |
| 2026-05-30T17:08Z | new | groomed | philippepascal |
| 2026-05-30T17:17Z | groomed | in_design | philippepascal |
| 2026-05-30T17:21Z | in_design | specd | claude |
| 2026-05-30T18:09Z | specd | ready | philippepascal |
| 2026-05-30T18:33Z | ready | in_progress | philippepascal |