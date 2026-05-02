+++
id = "9fcc94ed"
title = "Agent instructions: graceful exit via question/blocked state when stuck on capability limits"
state = "in_design"
priority = 0
effort = 2
risk = 1
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/9fcc94ed-agent-instructions-graceful-exit-via-que"
created_at = "2026-05-01T02:34:11.627171Z"
updated_at = "2026-05-02T07:21:28.744056Z"
+++

## Spec

### Problem

When a worker can't complete its task because of a tool limitation, permission denial, missing dependency, or unforeseen blocker, today there is no clear escape hatch. The worker either: (a) improvises off-topic work (the side-quest pattern from the 2026-04-30 incident on ticket 2803bf07, where a permission prompt led the worker to invoke the `fewer-permission-prompts` skill and try to edit settings.json); (b) gives up silently and exits without transitioning state, leaving the ticket stuck in `in_design` or `in_progress`; or (c) crashes outright.

The cleanest path is the one already in the workflow: `question` state for spec-writers, `blocked` state for impl-agents. Both are `actionable = ["supervisor"]`, so the supervisor sees them in the queue and can intervene. The `### Open questions` section is the standard place to document what was needed.

Today the agent instructions (`.apm/agents.md`, `.apm/apm.spec-writer.md`, `.apm/apm.worker.md`) cover the case of being blocked on an *ambiguity* ("write the question in Open questions, then `apm state <id> question`"). They do not cover the case of being blocked on a *capability limitation* — the kind of blocker that pushed the 2803bf07 worker into a side-quest instead of a clean exit.

This ticket adds an explicit "## Capability limitations" section to the spec-writer and worker instruction files covering exactly this case, and a pointer sentence in the project-wide `agents.md` conventions file.

### Acceptance criteria

- [ ] `apm-core/src/default/agents/claude/apm.spec-writer.md` contains a new "## Capability limitations" section placed after the existing "## Open questions" section
- [ ] The spec-writer capability-limitations section explicitly prohibits invoking skills, editing `.claude/settings.json`, editing `.apm/` files, and attempting workarounds outside the worktree
- [ ] The spec-writer capability-limitations section gives the two-step clean exit: `apm spec <id> --section "Open questions" --append "..."` then `apm state <id> question`
- [ ] `apm-core/src/default/agents/claude/apm.worker.md` contains a new "## Capability limitations" section placed after the existing "## Blocked state" section
- [ ] The worker capability-limitations section gives the two-step clean exit: `apm spec <id> --section "Open questions" --append "..."` then `apm state <id> blocked` (no manual git commit — `apm spec --append` auto-commits)
- [ ] `apm-core/src/default/apm.spec-writer.md` (flat default) is byte-for-byte identical to `apm-core/src/default/agents/claude/apm.spec-writer.md`
- [ ] `apm-core/src/default/apm.worker.md` (flat default) is byte-for-byte identical to `apm-core/src/default/agents/claude/apm.worker.md`
- [ ] `.apm/apm.spec-writer.md` is byte-for-byte identical to `apm-core/src/default/apm.spec-writer.md`
- [ ] `.apm/apm.worker.md` is byte-for-byte identical to `apm-core/src/default/apm.worker.md`
- [ ] `.apm/agents.md` contains a sentence near the `### Worker` section pointing agents to the per-role file for the capability-limitation escape hatch
- [ ] `apm-core/src/default/apm.agents.md` contains the identical sentence as `.apm/agents.md`
- [ ] `apm-core/tests/worker_md_sync.rs` contains a test function that asserts `apm-core/src/default/apm.spec-writer.md` and `.apm/apm.spec-writer.md` are byte-for-byte identical and produces a readable diff on failure
- [ ] `cargo test --workspace` passes including both the existing worker sync test and the new spec-writer sync test
- [ ] Both `## Capability limitations` sections state that the instructions assume the default ticket schema includes `### Open questions`; projects with customised schemas that omit it are explicitly noted as out of scope within the section

### Out of scope

- Auto-detecting when an agent is in a stuck loop and forcing the transition (the agent decides; instructions only provide the option)
- Tooling to distinguish capability-limitation blocks from design-ambiguity blocks in the supervisor queue (the supervisor reads the Open questions text)
- Per-agent prompt tightening for non-Claude wrappers; the guidance lands in the claude wrapper files only — future wrappers inherit it via apm agents new scaffolding
- Migrating the project config from the flat .apm/ layout to .apm/agents/claude/; that is owned by epic 4312fbd4

### Approach

#### Content to add to apm.spec-writer.md

Add a new `## Capability limitations` section immediately after the existing `## Open questions` section (before the `---` separator that precedes the `**Frontmatter agent override**` footer). Exact text:

```markdown

### Open questions


### Amendment requests

- [ ] AC #5's three-step exit is wrong. It says: `apm spec --append`, then `git -C <worktree> add/commit`, then `apm state blocked`. But `apm spec --append` already auto-commits to the ticket branch (per `.apm/agents.md`'s "each `apm spec` call auto-commits" rule). The manual `git add/commit` step in the middle is redundant and would fail (nothing to commit). Drop step 2 entirely — `apm spec --append` is enough; `apm state blocked` follows.

- [ ] Out-of-scope section has literal `\n` escapes instead of newlines, rendering as a single line. Visible in the spec output between "decides whether..." and "Tooling to distinguish...". Re-set the section with proper newlines so the supervisor can read it. Same root cause as the rendering issue flagged in 40fdde3b and a168ad77.

- [ ] No AC for what to do if `### Open questions` doesn't exist in the ticket. `apm spec --section "Open questions" --append` requires the section to be configured in `[[ticket.sections]]` — it is by default, but a project that customised sections may not have it. Add an AC noting this assumption (default ticket schema includes Open questions; non-default schemas are out of scope), or have the instructions say to use `apm new --side-note` as the fallback.

### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-05-01T02:34Z | — | new | philippepascal |
| 2026-05-02T03:07Z | new | groomed | philippepascal |
| 2026-05-02T03:40Z | groomed | in_design | philippepascal |
| 2026-05-02T03:46Z | in_design | specd | claude-0502-0340-59f0 |
| 2026-05-02T07:20Z | specd | ammend | claude-0502-1300-rev1 |
| 2026-05-02T07:21Z | ammend | in_design | philippepascal |