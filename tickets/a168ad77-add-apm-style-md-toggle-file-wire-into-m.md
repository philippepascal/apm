+++
id = "a168ad77"
title = "Add .apm/style.md toggle file; wire into main agent and spec-writer"
state = "in_design"
priority = 0
effort = 2
risk = 2
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/a168ad77-add-apm-style-md-toggle-file-wire-into-m"
created_at = "2026-05-01T19:39:37.765619Z"
updated_at = "2026-05-02T07:29:51.436351Z"
+++

## Spec

### Problem

APM agents currently have no mechanism for experimenting with output brevity. Conversation replies can be verbose (preambles, multi-sentence answers to one-liners, end-of-turn check-ins), and spec output can be padded (long Problem sections, AC lists that exceed what's useful). The supervisor wants to compress responses without committing to a full rewrite — trying rules one at a time and keeping only what helps.

A checkbox-toggle file at `.apm/style.md` provides that mechanism. All rules start unchecked; the supervisor checks individual boxes to activate them and unchecks them if they cause problems. The file is read by the main agent on every session (via a `CLAUDE.md` import) and by the spec-writer before writing or amending a spec (via the per-agent instruction files introduced by the wrapper epic, ticket 4312fbd4). No code parses the file; it is read directly by the agents as part of their prompt context.

**Dependency:** this ticket must land after the wrapper epic (4312fbd4), which creates the `.apm/agents/claude/` per-agent layout that the spec-writer `.md` change targets.

### Acceptance criteria

- [ ] `.apm/style.md` exists with a `## Conversation` section and a `## Specs` section; every rule is a `- [ ]` checkbox; all boxes are unchecked by default
- [ ] `CLAUDE.md` contains an `@.apm/style.md` import line alongside the existing `@.apm/agents.md` import
- [ ] A user-memory file in `.claude/projects/…/memory/` describes `.apm/style.md`, instructs the main agent to apply active `## Conversation` rules to its own output, and instructs it to prepend active Conversation rules to prompts when spawning subagents via the Agent tool
- [ ] `apm-core/src/default/agents/claude/apm.spec-writer.md` contains a paragraph instructing the spec-writer to read `.apm/style.md` (if present) before writing or amending a spec and apply every `[x]`-checked rule under `## Specs`
- [ ] `.apm/agents/claude/apm.spec-writer.md` (project file, post-wrapper-epic) contains the identical paragraph
- [ ] A new Rust test in `apm-core/tests/spec_writer_md_sync.rs` asserts the two spec-writer `.md` files are byte-identical and fails with a line-level diff if they diverge

### Out of scope

- Auto-sync between `.apm/style.md` and the agent .md files (future `apm style sync` command)\n- Applying style rules to `apm.worker.md` (impl-agent brevity is not the concern)\n- A schema or parser for `.apm/style.md` — the file is human-read, no code validation needed

### Approach

**Prerequisite:** confirm the wrapper epic (4312fbd4) is merged before starting; the `.apm/agents/claude/` directory must exist.

#### 1. Create `.apm/style.md`

New file at repo root `.apm/style.md`. Content:

```markdown
# APM Style Rules

Rules are opt-in. Check a box (`[x]`) to activate a rule. Default: all unchecked.

### Open questions


### Amendment requests

- [ ] Wiring leaks into private user-memory state. AC #3 puts the operating instructions in `.claude/projects/.../memory/project_style_rules.md` — this is the supervisor's per-machine memory, not in-repo. A second supervisor cloning the repo gets `style.md` and `CLAUDE.md @import` but no instructions on what to do with them; the rules become inert. Either (a) move the operating instructions into `CLAUDE.md` itself (or a sibling tracked file like `.apm/agents.md`) so they ship with the repo, or (b) state explicitly in Out of scope that this ticket only configures the supervisor's own machine.

- [ ] `@.apm/style.md` import behaviour with checkbox text is unspecified. The file is human-targeted Markdown with both checked and unchecked rules. When `@import` pulls it into context, the agent sees both — the spec doesn't say how the agent distinguishes "active" from "candidate". The user-memory note (AC #3) says "check which rules are marked `[x]` and apply them" but nothing prevents the agent from soft-applying unchecked ones too. Add an AC: the user-memory note must explicitly say "treat `[ ]` rules as inactive — do not apply or reference them".

- [ ] AC #6 (byte-identity check on `.apm/apm.spec-writer.md`) is brittle for a project file users may legitimately customise. After this ticket lands, any supervisor who edits `.apm/agents/claude/apm.spec-writer.md` (the entire purpose of the per-agent override layer) breaks the test. Either narrow the assertion to "the `## Style rules` section is identical" (grep both files for that section, diff just that block), or drop AC #6 / move the check to a `cargo xtask`-style optional verification not in the test suite.

- [ ] Out of scope items 1 and 2 contain literal `\n` instead of newlines — re-set with proper line breaks so the checklist renders as three items, not one paragraph. Same root cause as 40fdde3b and 9fcc94ed.

### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-05-01T19:39Z | — | new | philippepascal |
| 2026-05-02T03:07Z | new | groomed | philippepascal |
| 2026-05-02T03:46Z | groomed | in_design | philippepascal |
| 2026-05-02T03:50Z | in_design | specd | claude-0502-0346-e438 |
| 2026-05-02T07:20Z | specd | ammend | claude-0502-1300-rev1 |
| 2026-05-02T07:29Z | ammend | in_design | philippepascal |
