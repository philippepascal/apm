+++
id = "9fcc94ed"
title = "Agent instructions: graceful exit via question/blocked state when stuck on capability limits"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/9fcc94ed-agent-instructions-graceful-exit-via-que"
created_at = "2026-05-01T02:34:11.627171Z"
updated_at = "2026-05-02T03:40:13.248519Z"
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

Explicit list of what this ticket does not cover.

### Approach

How the implementation will work.

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-05-01T02:34Z | — | new | philippepascal |
| 2026-05-02T03:07Z | new | groomed | philippepascal |
| 2026-05-02T03:40Z | groomed | in_design | philippepascal |