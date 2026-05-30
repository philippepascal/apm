+++
id = "a3c34ddc"
title = "Move shell-discipline rules from apm instructions into Claude role files"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/a3c34ddc-move-shell-discipline-rules-from-apm-ins"
created_at = "2026-05-30T16:45:44.032054Z"
updated_at = "2026-05-30T17:17:19.246527Z"
+++

## Spec

### Problem

PROBLEM: the Shell Discipline section currently embedded in apm-core/src/instructions.rs (the SHELL_DISCIPLINE_BODY constant) is Claude-specific. Its examples use Claude Code's permission allow-list syntax (Bash(ls *), Bash(bash *)) and refer to Claude-specific mechanisms (the --print mode, parallel tool-use blocks). But apm instructions is shared output read by every agent — including non-Claude agents that may not use the same shell semantics. The rules are misplaced.

The parallel-tool-batching rule added by ticket 753d9ba5 lives in the same body and inherits the same misplacement.

GOAL: relocate the rules to a place where (a) Claude workers see them prominently in the role file (high attention position), and (b) other agents are not forced to see Claude-specific guidance.

SOLUTION:
1. Drop the Shell Discipline section from apm-core/src/instructions.rs. SHELL_DISCIPLINE_BODY and the parallel-batching block both removed. The remaining apm-instructions content (state machine, ticket format, session identity, command reference) is genuinely agent-agnostic.
2. Inline the same rules into each Claude role file under a clearly-named section, in this order: apm-core/src/default/agents/claude/apm.coder.md, apm.spec-writer.md, apm.main-agent.md. AND the live committed copies in .apm/agents/claude/apm.coder.md, apm.spec-writer.md, apm.main-agent.md. Both default templates and project files must stay in sync (existing worker_md_sync test enforces this).
3. The section heading should be unambiguous and the rules should be the FIRST substantive section of the role file (or close to the top, before role-specific phase guidance). Suggestion: ## Shell discipline — non-negotiable, then the rules, then ## (role-specific content). The exact heading wording is the spec-writer's choice but it must clearly mark the rules as required.
4. The two rules to include:
   - one shell command per tool call: no &&, ;, |, &,  subshells, no compound or env-prefixed forms
   - no parallel tool-use blocks emitted in a single turn (the failure mode from 753d9ba5)
5. No changes to non-Claude agent content. Non-Claude agents simply stop receiving these Claude-specific rules. If a future non-Claude agent needs its own shell rules, that is a separate concern handled per-agent in that agent's own role files.

OUT OF SCOPE:
- Reformatting apm instructions (separate ticket; the dynamic shape change is a different concern).
- Changes to the cascade that picks the role file.
- Touching .apm/agents/<other-agent>/*.md.
- The CLAUDE.md @-import pattern (the rules live in the role files directly, not a shared imported file — option 2 from the design discussion).

TESTS:
- The existing worker_md_sync.rs test that asserts the default template and the project's committed role file are byte-identical must still pass after the changes (the rules go into both).
- The instructions tests in apm-core/src/instructions.rs that snapshot or assert on SHELL_DISCIPLINE_BODY content must be updated or removed.
- A new integration test that runs apm prompt for a claude/coder ticket and asserts the shell-discipline section appears in the rendered system prompt (i.e. the rules are not lost in the relocation).
- A negative test that the apm instructions output no longer contains the chaining or parallel-batching guidance (so we know the relocation actually fired).

### Acceptance criteria

Checkboxes; each one independently testable.

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
| 2026-05-30T16:45Z | — | new | philippepascal |
| 2026-05-30T17:08Z | new | groomed | philippepascal |
| 2026-05-30T17:17Z | groomed | in_design | philippepascal |
