+++
id = "34ad9126"
title = "Rewrite apm.spec-writer.md built-in default"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/34ad9126-rewrite-apm-spec-writer-md-built-in-defa"
created_at = "2026-05-22T23:22:22.098663Z"
updated_at = "2026-05-22T23:58:32.049061Z"
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
| 2026-05-22T23:58Z | groomed | in_design | philippepascal |