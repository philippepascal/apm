+++
id = "a168ad77"
title = "Add .apm/style.md toggle file; wire into main agent and spec-writer"
state = "specd"
priority = 0
effort = 2
risk = 2
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/a168ad77-add-apm-style-md-toggle-file-wire-into-m"
created_at = "2026-05-01T19:39:37.765619Z"
updated_at = "2026-05-02T03:50:10.416301Z"
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

## Conversation

- [ ] Limit replies to 1–3 sentences unless depth is genuinely needed
- [ ] No preamble — do not restate the task or explain what you are about to do
- [ ] No end-of-turn offers ("Let me know if…", "Feel free to ask…")
- [ ] No bullet lists for simple factual answers; use prose

## Specs

- [ ] Problem section: ≤3 paragraphs
- [ ] Acceptance criteria: ≤6 items per ticket
- [ ] Do not restate the ticket title in the Problem section
- [ ] Skip "Out of scope" when the boundary is obvious from context
```

#### 2. Add import to `CLAUDE.md`

Insert `@.apm/style.md` immediately after the existing `@.apm/agents.md` line (line 3). Result:

```
@.apm/agents.md
@.apm/style.md
```

#### 3. Create user-memory note

Create `.claude/projects/-Users-philippepascal-repos-apm/memory/project_style_rules.md`:

```markdown
# Style Rules (`.apm/style.md`)

The file `.apm/style.md` contains opt-in brevity rules for this project.

- Before every reply, check which rules under `## Conversation` are marked `[x]` and apply them to your own output.
- When spawning subagents via the Agent tool, prepend a note listing all active (checked) Conversation rules to the subagent prompt.
- When writing or amending a spec, apply every rule under `## Specs` that is marked `[x]`.
```

Also add a line to the memory index (`MEMORY.md`):

```
- [Style rules toggle file](project_style_rules.md) — `.apm/style.md` opt-in brevity rules; apply active Conversation rules to own output and subagent prompts
```

#### 4. Update both spec-writer `.md` files

Add the following new section to **both** files immediately after the opening intro paragraph (after line 6, before `## How to save spec sections`):

```markdown
## Style rules

Before writing or amending a spec, read `.apm/style.md` (if present) and apply
every rule under `## Specs` that is marked `[x]`.

---
```

Files to edit:
- `apm-core/src/default/agents/claude/apm.spec-writer.md`
- `.apm/agents/claude/apm.spec-writer.md`

Both edits must be identical. Edit the default template first, then copy the same diff to the project file.

#### 5. Add `spec_writer_md_sync.rs` test

Create `apm-core/tests/spec_writer_md_sync.rs` following the exact same pattern as `worker_md_sync.rs`:
- Paths: `apm-core/src/default/agents/claude/apm.spec-writer.md` (default) and `.apm/agents/claude/apm.spec-writer.md` (project)
- Same byte-comparison logic and line-level diff on failure
- Same panic message style, substituting "apm.spec-writer.md" for "apm.worker.md"

Run `cargo test --workspace` to confirm both the new test and `default_and_project_apm_worker_md_are_identical` pass before marking implemented.

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-05-01T19:39Z | — | new | philippepascal |
| 2026-05-02T03:07Z | new | groomed | philippepascal |
| 2026-05-02T03:46Z | groomed | in_design | philippepascal |
| 2026-05-02T03:50Z | in_design | specd | claude-0502-0346-e438 |
