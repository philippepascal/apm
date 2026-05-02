+++
id = "a168ad77"
title = "Add .apm/style.md toggle file; wire into main agent and spec-writer"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/a168ad77-add-apm-style-md-toggle-file-wire-into-m"
created_at = "2026-05-01T19:39:37.765619Z"
updated_at = "2026-05-02T03:46:35.921856Z"
+++

## Spec

### Problem

Add a checkbox-toggle file at `.apm/style.md` for experimenting with brevity rules in conversation and spec output. Wire it into the main agent (via `CLAUDE.md` import) and into the spec-writer worker (via the per-agent .md files that the wrapper epic introduces).

**Context:** see the 2026-05-01 conversation. Goal is to compress responses and specs without going to caveman levels; the toggle file lets the supervisor activate rules one at a time and keep the ones that work.

**Should land after the wrapper epic (4312fbd4)** so the spec-writer changes land in the per-agent layout (`.apm/agents/<wrapper>/apm.spec-writer.md`).

**Scope:**

- Create `.apm/style.md` with two sections (Conversation, Specs), each rule a single `- [ ]` line. Default: every box unchecked. Initial rule list as drafted in the conversation (1-3 sentence default, no preambles, no end-of-turn offers, ≤6 AC items, ≤3 paragraph Problem, etc.).
- Add `@.apm/style.md` import to project `CLAUDE.md` so the main agent loads it every session.
- Save a user-memory note pointing at `.apm/style.md` so the rule applies even when CLAUDE.md isn't reloaded.
- Update `.apm/agents/claude/apm.spec-writer.md` (project file, post-epic) AND `apm-core/src/default/agents/claude/apm.spec-writer.md` (default template) to add a paragraph: "Before writing or amending a spec, read `.apm/style.md` (if present) and apply every rule under '## Specs' that is marked `[x]`." The .md sync rule from earlier (changes land in both src default and project file) applies.
- The main agent's responsibility for subagent prompts: documented in the memory note, not enforced by code. Memory note says: "when spawning subagents (Agent tool), prepend active Conversation rules from `.apm/style.md` to the prompt."

**Out of scope:**

- Auto-sync between `.apm/style.md` and the .md files (a future `apm style sync` command, or wrapper-layer injection at spawn time). Manual sync is fine while toggles are still being tested.
- Applying style rules to `apm.worker.md` (impl-agent). Conversation/spec brevity is mostly about supervisor-facing output; impl agents write code and commit messages where brevity isn't the load-bearing concern.
- A schema for `.apm/style.md` parsing. The file is read by humans (the supervisor, me) and inspected by anyone reviewing — no code parses it; no validation needed.

**Acceptance criteria:**

- [ ] `.apm/style.md` exists with Conversation and Specs sections, all rules unchecked by default.
- [ ] `CLAUDE.md` imports `.apm/style.md` via `@.apm/style.md`.
- [ ] User-memory file references `.apm/style.md` and instructs me to apply active rules to my own output and to subagent prompts.
- [ ] `.apm/agents/claude/apm.spec-writer.md` (project) and `apm-core/src/default/agents/claude/apm.spec-writer.md` (default template) both contain the read-style.md instruction; their content is byte-identical except for project-specific overrides.
- [ ] The .md sync test from `498febe0` still passes (confirms the two files stay aligned).

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
| 2026-05-01T19:39Z | — | new | philippepascal |
| 2026-05-02T03:07Z | new | groomed | philippepascal |
| 2026-05-02T03:46Z | groomed | in_design | philippepascal |
