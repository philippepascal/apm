+++
id = "34ad9126"
title = "Rewrite apm.spec-writer.md built-in default"
state = "ready"
priority = 0
effort = 2
risk = 2
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/34ad9126-rewrite-apm-spec-writer-md-built-in-defa"
created_at = "2026-05-22T23:22:22.098663Z"
updated_at = "2026-05-23T02:58:13.427968Z"
epic = "ab6e5db7"
target_branch = "epic/ab6e5db7-prompt-management-redesign"
depends_on = ["4bee5771"]
+++

## Spec

### Problem

The built-in default `apm-core/src/default/agents/default/apm.spec-writer.md` (272 lines) contains two categories of content that become redundant once T1 (4bee5771) lands:

1. **Runtime-specific role-identification language.** The file opens with "This session was started with `--disable-slash-commands`. Skill and slash command invocation is disabled." After T3 (d8e2fa0e) rewrites prompt assembly to compose three layers, the role file is selected by the system — it does not need to self-declare its activation conditions. This language is also fragile: it embeds runtime harness behaviour into a static text file.

2. **Shell discipline content.** The "How to save spec sections" block (Write-tool + `--set-file` pattern, avoiding `$()` subshells) duplicates what T1 will emit via its generic shell discipline section. The "Permitted `apm` commands" list duplicates what T1's command reference will emit when filtered to the `spec-writer` role. An unrelated `$(ls ...)` subshell survives in amendment step 6 and violates the discipline it teaches.

The History/Filename preservation rules (never hand-edit `## History`, never rename the ticket file) are spec-writer-unique and must be kept — sibling T5 (78eeb755) will copy them to `apm.worker.md`.

The claude-agent override (`apm-core/src/default/agents/claude/apm.spec-writer.md`, 237 lines) is a stale near-copy of the default that was never updated with the History/Filename rules. After the default rewrite it has no meaningful delta. Deleting it and pointing the `resolve_builtin_instructions` match arm to the rewritten default removes a maintenance liability with no behaviour change for projects that use the default cascade.

### Acceptance criteria

- [ ] `apm-core/src/default/agents/default/apm.spec-writer.md` does not contain the phrase "disable-slash-commands"
- [ ] `apm-core/src/default/agents/default/apm.spec-writer.md` does not contain a "Permitted `apm` commands" section
- [ ] `apm-core/src/default/agents/default/apm.spec-writer.md` does not contain the Write-tool / `--set-file` code block (the `# Short content — inline` / `# Long content — via temp file` examples)
- [ ] `apm-core/src/default/agents/default/apm.spec-writer.md` does not contain any `$(` subshell patterns
- [ ] `apm-core/src/default/agents/default/apm.spec-writer.md` retains the "Never hand-edit the History table" section
- [ ] `apm-core/src/default/agents/default/apm.spec-writer.md` retains the "Filename is fixed — never rename the ticket file" section
- [ ] `apm-core/src/default/agents/claude/apm.spec-writer.md` does not exist on disk
- [ ] `cargo test --workspace` passes with no new failures

### Out of scope

- Rewriting apm.worker.md (78eeb755)\n- Deleting agents.md (1fce91bd)\n- Deleting the claude/apm.worker.md override (02bbcc2f)\n- Migrating the project's own .apm/agents/ (7c5c491d)\n- Changing what T1 emits in its shell discipline section

### Approach

**File 1 — `apm-core/src/default/agents/default/apm.spec-writer.md`**

Apply the following removals to the current 272-line file:

- **Remove** the two-sentence runtime notice at the top of `## Scope limits` ("This session was started with `--disable-slash-commands`…" and "If you see skill availability information…"). Leave the section header and the remaining sub-sections ("Off-limits" and the permission-prompt paragraph) intact.
- **Remove** the "Permitted `apm` commands" bullet list (five items). T1 emits this filtered by role; the list in the role file is redundant once T1 lands.
- **Remove** the opening prose of `## How to save spec sections` and its code block (the `# Short content — inline` / `# Long content — via temp file` block). This is shell discipline; T1 owns it. Retain only the single line "Do NOT write the ticket markdown file directly. Always use `apm spec`." Keep the `### Never hand-edit the History table` and `### Filename is fixed` subsections unchanged.
- **Fix** amendment step 6: delete the `FILE=$(ls ...)` / `git -C` / `git commit` block. Replace with a note that `apm spec` calls auto-commit to the ticket branch, so no manual git step is needed — consistent with the same note that already appears in the `## Open questions` / `## Capability limitations` sections.

Do not touch any other section. The spec-writing guidance (Problem, AC, Out of scope, Approach, Effort/Risk scales), amendment flow, open-questions process, and capability-limitations sections are all in scope to keep verbatim.

**File 2 — `apm-core/src/default/agents/claude/apm.spec-writer.md`**

Delete the file. No content is worth preserving — it predates the History/Filename rules and is otherwise identical to the default.

**File 3 — `apm-core/src/start.rs`**

- Remove the constant: `const CLAUDE_SPEC_WRITER_DEFAULT: &str = include_str!("default/agents/claude/apm.spec-writer.md");`
- Add a new constant: `const DEFAULT_SPEC_WRITER_DEFAULT: &str = include_str!("default/agents/default/apm.spec-writer.md");`
- In `resolve_builtin_instructions`, change the arm `("claude", "spec-writer") => Some(CLAUDE_SPEC_WRITER_DEFAULT)` to `("claude", "spec-writer") => Some(DEFAULT_SPEC_WRITER_DEFAULT)`. This preserves the Level 4 fallback for projects that have no per-project `.apm/agents/claude/apm.spec-writer.md` (fresh `apm init` repos).
- Update the test `build_system_prompt_falls_back_to_builtin_spec_writer`: change `assert_eq!(result, super::CLAUDE_SPEC_WRITER_DEFAULT)` to `assert_eq!(result, super::DEFAULT_SPEC_WRITER_DEFAULT)`.

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-05-22T23:22Z | — | new | philippepascal |
| 2026-05-22T23:50Z | new | groomed | philippepascal |
| 2026-05-22T23:58Z | groomed | in_design | philippepascal |
| 2026-05-23T00:06Z | in_design | specd | claude-0522-2358-ce28 |
| 2026-05-23T02:58Z | specd | ready | philippepascal |
