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

`SHELL_DISCIPLINE_BODY` in `apm-core/src/instructions.rs` contains Claude-specific guidance: it references Claude Code's permission allow-list syntax (`Bash(ls *)`, `Bash(bash *)`), the `--print` headless mode, and parallel tool-use block behaviour. These mechanics are not shared by other agent runtimes. Yet `apm instructions` emits this section to every agent regardless of type, so a non-Claude agent is forced to read rules that do not apply to it. The parallel-tool-batching rule introduced by ticket 753d9ba5 lives in the same constant and carries the same misplacement.

The fix is to move the entire `SHELL_DISCIPLINE_BODY` block out of `apm instructions` and into each Claude role file (`apm.coder.md`, `apm.spec-writer.md`, `apm.main-agent.md`), positioned before the first role-specific section so it appears early in the agent's context. `apm instructions` is left with content that is genuinely agent-agnostic: state machine, ticket format, session identity, and command reference. Non-Claude agents simply stop receiving guidance that was never relevant to them.

### Acceptance criteria

- [ ] `apm instructions` output does not contain a `## Shell Discipline` heading
- [ ] `apm instructions` output does not contain the text "Do not batch tool calls in parallel"
- [ ] `apm instructions` output does not contain `&&` as shell-discipline guidance
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