+++
id = "9fcc94ed"
title = "Agent instructions: graceful exit via question/blocked state when stuck on capability limits"
state = "specd"
priority = 0
effort = 2
risk = 1
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/9fcc94ed-agent-instructions-graceful-exit-via-que"
created_at = "2026-05-01T02:34:11.627171Z"
updated_at = "2026-05-02T03:46:21.273334Z"
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
- [ ] The worker capability-limitations section gives the three-step clean exit: append to Open questions, commit the update, then `apm state <id> blocked`
- [ ] `apm-core/src/default/apm.spec-writer.md` (flat default) is byte-for-byte identical to `apm-core/src/default/agents/claude/apm.spec-writer.md`
- [ ] `apm-core/src/default/apm.worker.md` (flat default) is byte-for-byte identical to `apm-core/src/default/agents/claude/apm.worker.md`
- [ ] `.apm/apm.spec-writer.md` is byte-for-byte identical to `apm-core/src/default/apm.spec-writer.md`
- [ ] `.apm/apm.worker.md` is byte-for-byte identical to `apm-core/src/default/apm.worker.md`
- [ ] `.apm/agents.md` contains a sentence near the `### Worker` section pointing agents to the per-role file for the capability-limitation escape hatch
- [ ] `apm-core/src/default/apm.agents.md` contains the identical sentence as `.apm/agents.md`
- [ ] `apm-core/tests/worker_md_sync.rs` contains a test function that asserts `apm-core/src/default/apm.spec-writer.md` and `.apm/apm.spec-writer.md` are byte-for-byte identical and produces a readable diff on failure
- [ ] `cargo test --workspace` passes including both the existing worker sync test and the new spec-writer sync test

### Out of scope

- Auto-detecting when an agent is in a stuck loop and forcing the transition (the agent decides; instructions only provide the option)\n- Tooling to distinguish capability-limitation blocks from design-ambiguity blocks in the supervisor queue (the supervisor reads the Open questions text)\n- Per-agent prompt tightening for non-Claude wrappers; the guidance lands in the claude wrapper files only — future wrappers inherit it via apm agents new scaffolding\n- Migrating the project config from the flat .apm/ layout to .apm/agents/claude/; that is owned by epic 4312fbd4

### Approach

#### Content to add to apm.spec-writer.md

Add a new `## Capability limitations` section immediately after the existing `## Open questions` section (before the `---` separator that precedes the `**Frontmatter agent override**` footer). Exact text:

```markdown
## Capability limitations

If you cannot proceed because of a tool restriction, permission denial, or inaccessible resource — a tool you cannot invoke, a command blocked by the allowlist, a file you cannot access, or a permission prompt you cannot answer — **do not improvise.** Specifically, do not:
- Invoke skills (e.g. `fewer-permission-prompts`, `update-config`)
- Edit project-level configuration (`.claude/settings.json`, `.apm/`, `.gitignore`)
- Attempt workarounds that touch files outside your worktree

Instead, document the blocker and exit cleanly:
1. `apm spec <id> --section "Open questions" --append "<what you tried, what was denied, what would unblock you>"`
2. `apm state <id> question`

The supervisor sees the `question` state and decides whether to grant a permission, update the allowlist, or revise the ticket.
```

#### Content to add to apm.worker.md

Add a new `## Capability limitations` section immediately after the existing `## Blocked state` section (before the `---` separator that precedes `## Shell discipline`). Exact text:

```markdown
## Capability limitations

If you cannot proceed because of a tool restriction, permission denial, or inaccessible resource — a test command blocked by the allowlist, a file you cannot edit because it is outside your worktree, a permission prompt for an `apm` command — **do not improvise.** Do not invoke skills, do not edit project configuration.

Instead, document the blocker and exit cleanly:
1. `apm spec <id> --section "Open questions" --append "<what you tried, what was denied, what would unblock you>"`
2. Commit the updated ticket to the worktree branch: `git -C <worktree-path> add tickets/<id>-*.md && git -C <worktree-path> commit -m "ticket(<id>): document capability blocker"`
3. `apm state <id> blocked`

The supervisor sees the `blocked` state and decides whether to grant a permission, update the allowlist, or revise the ticket.
```

#### Sentence to add to agents.md

Under the `### Worker` section (after the sentence "You have been assigned a single ticket. Implement it, run tests, and mark it implemented. Do not spawn further workers or act as delegator."), add:

> When stuck on a capability limitation (tool denied, command not in the allowlist, file outside your worktree), see `apm.spec-writer.md` or `apm.worker.md` for the clean exit via `question` or `blocked` state — do not improvise or invoke skills.

#### Files to edit (seven total)

Edit in this order so the sync test passes after the first run:

1. `apm-core/src/default/agents/claude/apm.spec-writer.md` — add capability-limitations section
2. `apm-core/src/default/apm.spec-writer.md` — reconcile pre-existing divergence first, then add the same section

   **Pre-existing divergence:** `.apm/apm.spec-writer.md` is missing the 6-line `####`-heading guidance block that both default files already contain (in the `## Approach` section, after "Write the Approach as a single pass."). Bring the project file up to the default's content before adding the new section. The diff to apply to the project file:
   - Before adding the capability-limitations section, insert after the "Write the Approach as a single pass." paragraph:
     ```
     Use `####` headings within long sections to create named subsections that
     serve as editing handles. Example: inside `### Approach`, add `#### Phase 1`
     so a future `apm spec <id> --section "Approach > Phase 1"` can update that
     block without touching the rest.
     ```
   - Then add the capability-limitations section in the same position as step 1.

3. `.apm/apm.spec-writer.md` — same reconciliation + same addition (must be byte-for-byte identical to `apm-core/src/default/apm.spec-writer.md`)
4. `apm-core/src/default/agents/claude/apm.worker.md` — add capability-limitations section
5. `apm-core/src/default/apm.worker.md` — add capability-limitations section (must be identical to per-agent file)
6. `.apm/apm.worker.md` — add capability-limitations section (must be identical to flat default)
7. `.apm/agents.md` and `apm-core/src/default/apm.agents.md` — add the pointer sentence under `### Worker`

#### Sync test extension

In `apm-core/tests/worker_md_sync.rs`, add a second test function `default_and_project_apm_spec_writer_md_are_identical` that mirrors `default_and_project_apm_worker_md_are_identical` exactly, substituting `apm.spec-writer.md` for `apm.worker.md` in all paths and strings.

#### Verification

`cargo test --workspace` must pass. The new spec-writer sync test will fail until step 3 above is done correctly — that's intentional; fix the files first, then run tests.

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-05-01T02:34Z | — | new | philippepascal |
| 2026-05-02T03:07Z | new | groomed | philippepascal |
| 2026-05-02T03:40Z | groomed | in_design | philippepascal |
| 2026-05-02T03:46Z | in_design | specd | claude-0502-0340-59f0 |
