+++
id = "a168ad77"
title = "Add .apm/style.md toggle file; wire into main agent and spec-writer"
state = "ready"
priority = 0
effort = 2
risk = 2
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/a168ad77-add-apm-style-md-toggle-file-wire-into-m"
created_at = "2026-05-01T19:39:37.765619Z"
updated_at = "2026-05-02T18:21:45.424770Z"
+++

## Spec

### Problem

APM agents currently have no mechanism for experimenting with output brevity. Conversation replies can be verbose (preambles, multi-sentence answers to one-liners, end-of-turn check-ins), and spec output can be padded (long Problem sections, AC lists that exceed what's useful). The supervisor wants to compress responses without committing to a full rewrite — trying rules one at a time and keeping only what helps.

A checkbox-toggle file at `.apm/style.md` provides that mechanism. All rules start unchecked; the supervisor checks individual boxes to activate them and unchecks them if they cause problems. The file is read by the main agent on every session (via a `CLAUDE.md` import); a companion `## Style rules` paragraph in `CLAUDE.md` (committed in-repo) tells the agent which rules to apply and that unchecked rules are inactive. The spec-writer reads it before writing or amending a spec via the per-agent instruction files introduced by the wrapper epic (ticket 4312fbd4). No code parses the file; agents read it directly as prompt context.

**Dependency:** this ticket must land after the wrapper epic (4312fbd4), which creates the `.apm/agents/claude/` per-agent layout that the spec-writer `.md` change targets.

### Acceptance criteria

- [ ] `.apm/style.md` exists with a `## Conversation` section and a `## Specs` section; every rule is a `- [ ]` checkbox; all boxes are unchecked by default
- [ ] `CLAUDE.md` contains an `@.apm/style.md` import line alongside the existing `@.apm/agents.md` import
- [ ] `CLAUDE.md` contains a `## Style rules` paragraph (committed in-repo, not in user-memory) instructing the main agent to apply active `## Conversation` rules to its own output and to prepend active Conversation rules to subagent prompts when spawning via the Agent tool
- [ ] The `## Style rules` paragraph in `CLAUDE.md` explicitly states that `[ ]`-unchecked rules are inactive and must not be applied or referenced
- [ ] `apm-core/src/default/agents/claude/apm.spec-writer.md` contains a `## Style rules` paragraph instructing the spec-writer to read `.apm/style.md` (if present) before writing or amending a spec, apply every `[x]`-checked rule under `## Specs`, and treat `[ ]`-unchecked rules as inactive
- [ ] `.apm/agents/claude/apm.spec-writer.md` (project file, post-wrapper-epic) contains the identical `## Style rules` paragraph
- [ ] A new Rust test in `apm-core/tests/spec_writer_md_sync.rs` extracts the `## Style rules` section from both spec-writer `.md` files and asserts those sections are identical; the test fails with a line-level diff of that section if they diverge

### Out of scope

- Auto-sync between `.apm/style.md` and the agent .md files (future `apm style sync` command)
- Applying style rules to `apm.worker.md` (impl-agent brevity is not the concern)
- A schema or parser for `.apm/style.md` — the file is human-read, no code validation needed

### Approach

**Prerequisite:** confirm the wrapper epic (4312fbd4) is merged; `.apm/agents/claude/apm.spec-writer.md` must exist before starting.

#### 1. Create `.apm/style.md`

New file at repo root `.apm/style.md`:

```markdown
# APM Style Rules

Rules are opt-in. Check a box (`[x]`) to activate a rule. Default: all unchecked.

## Conversation

- [ ] Skip preamble — reply without restating the request or summarising what you are about to do
- [ ] No end-of-turn check-ins — omit "Let me know if …" or "Does this look right?" closings
- [ ] One-liners only for one-line questions — do not expand a simple yes/no into a paragraph
- [ ] No bullet-point rewrites of prose — if the answer is a single sentence, give a single sentence

## Specs

- [ ] Limit the Problem section to two paragraphs
- [ ] Cap the AC list at eight items — merge closely related checks rather than splitting
- [ ] Omit trivial boundary ACs when the main-path AC implies them
- [ ] Limit the Approach section to 400 words
```

#### 2. Update `CLAUDE.md`

Two edits:
- Add `@.apm/style.md` on a new line immediately after the `@.apm/agents.md` import
- Add a `## Style rules` section after the @imports block and before `## Commits`:

```markdown
## Style rules

@.apm/style.md contains opt-in output-style rules for this session. On startup:
- Read `.apm/style.md` and identify every rule marked `[x]` in `## Conversation`
- Apply those rules to your own replies for the entire session
- Rules marked `[ ]` are inactive — do not apply or reference them
- When spawning subagents via the Agent tool, prepend the text of each active `## Conversation` rule to the subagent prompt
```

#### 3. Update both spec-writer `.md` files

Append an identical `## Style rules` section to each of:
- `apm-core/src/default/agents/claude/apm.spec-writer.md`
- `.apm/agents/claude/apm.spec-writer.md`

The section to append (byte-for-byte identical in both files):

```markdown
## Style rules

Before writing or amending a spec, read `.apm/style.md` if present. Apply every rule marked `[x]` under `## Specs` to the spec you are writing. Rules marked `[ ]` are inactive — do not apply or reference them.
```

#### 4. Add Rust test `apm-core/tests/spec_writer_md_sync.rs`

The test:
1. Reads both spec-writer `.md` files:
   - Default: `env!("CARGO_MANIFEST_DIR")/src/default/agents/claude/apm.spec-writer.md`
   - Project: `env!("CARGO_MANIFEST_DIR")/../../.apm/agents/claude/apm.spec-writer.md`
2. From each file, extracts lines from the `## Style rules` heading to the next `##`-level heading or EOF
3. Asserts the two extracted slices are identical
4. On failure, prints each line with a `+`/`-` prefix to show the diff (no external crate; iterate lines with zip_longest-style comparison)

The test must not check whole-file byte identity — the project file (`.apm/agents/claude/apm.spec-writer.md`) may be legitimately customised beyond this section.

### Open questions


### Amendment requests

- [x] Wiring leaks into private user-memory state. AC #3 puts the operating instructions in `.claude/projects/.../memory/project_style_rules.md` — this is the supervisor's per-machine memory, not in-repo. A second supervisor cloning the repo gets `style.md` and `CLAUDE.md @import` but no instructions on what to do with them; the rules become inert. Either (a) move the operating instructions into `CLAUDE.md` itself (or a sibling tracked file like `.apm/agents.md`) so they ship with the repo, or (b) state explicitly in Out of scope that this ticket only configures the supervisor's own machine.

- [x] `@.apm/style.md` import behaviour with checkbox text is unspecified. The file is human-targeted Markdown with both checked and unchecked rules. When `@import` pulls it into context, the agent sees both — the spec doesn't say how the agent distinguishes "active" from "candidate". The user-memory note (AC #3) says "check which rules are marked `[x]` and apply them" but nothing prevents the agent from soft-applying unchecked ones too. Add an AC: the user-memory note must explicitly say "treat `[ ]` rules as inactive — do not apply or reference them".

- [x] AC #6 (byte-identity check on `.apm/apm.spec-writer.md`) is brittle for a project file users may legitimately customise. After this ticket lands, any supervisor who edits `.apm/agents/claude/apm.spec-writer.md` (the entire purpose of the per-agent override layer) breaks the test. Either narrow the assertion to "the `## Style rules` section is identical" (grep both files for that section, diff just that block), or drop AC #6 / move the check to a `cargo xtask`-style optional verification not in the test suite.

- [x] Out of scope items 1 and 2 contain literal `\n` instead of newlines — re-set with proper line breaks so the checklist renders as three items, not one paragraph. Same root cause as 40fdde3b and 9fcc94ed.

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
| 2026-05-02T07:40Z | in_design | ammend | philippepascal |
| 2026-05-02T07:43Z | ammend | in_design | philippepascal |
| 2026-05-02T07:50Z | in_design | specd | claude-0502-0743-90e0 |
| 2026-05-02T18:21Z | specd | ready | philippepascal |
