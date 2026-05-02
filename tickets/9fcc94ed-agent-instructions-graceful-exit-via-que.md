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

**Should land after the wrapper epic (4312fbd4) so the per-agent .md files are at `.apm/agents/<wrapper>/apm.<role>.md` and edits land in both the default templates and the project's current configs.**

**Scope — instruction-file additions (in BOTH `apm-core/src/default/agents/<wrapper>/apm.<role>.md` AND the project's current files):**

For **`apm.spec-writer.md`**, add a new subsection or expand the existing `### Open questions` flow:

> **When you cannot complete the spec because of a capability limitation** (a tool you cannot invoke, a permission prompt you cannot answer, a file you cannot access, a command not in the allowlist), **do not improvise.** Specifically: do not invoke skills, do not edit project configuration (`.claude/settings.json`, `.apm/`, `.gitignore`), do not attempt workarounds outside your worktree.
>
> Instead, document what you needed and exit cleanly:
>
> 1. `apm spec <id> --section "Open questions" --append "<what you tried, what was denied, what would unblock you>"`
> 2. `apm state <id> question`
>
> The supervisor sees the question and decides whether to grant a permission, update the allowlist, or rewrite the ticket.

For **`apm.worker.md`**, add the analogous block referencing the `blocked` state instead of `question`:

> **When you cannot complete implementation because of a capability limitation** (a test command you cannot run, a file you cannot edit because it is outside your worktree, a permission prompt for an apm command), **do not improvise.** Same constraints as above.
>
> Instead, document and exit cleanly via the existing blocked path:
>
> 1. `apm spec <id> --section "Open questions" --append "<what you tried, what was denied, what would unblock you>"`
> 2. `apm state <id> blocked`

For **`apm.agents.md`** (the project-wide conventions file), add a single sentence under the "Roles" or near the existing supervisor-only-transitions list pointing to the per-role files for the escape hatch.

**Out of scope:**
- Auto-detecting when the worker is in a stuck pattern and forcing the transition. The agent decides; the instructions just give it a clear option.
- Tooling to highlight question/blocked tickets that arose from capability limitations vs design ambiguities. The supervisor reads the open question text.
- Per-agent prompt tightening for non-Claude wrappers. Each wrapper's per-role .md gets the same guidance; future wrappers (Codex, Aider, etc.) inherit it via `apm agents new` scaffolding.

**Acceptance pointers:**
- `apm-core/src/default/agents/claude/apm.spec-writer.md` and `apm.worker.md` contain the new guidance.
- `.apm/agents/claude/apm.spec-writer.md` and `apm.worker.md` (the project's current files, post-7f5f73d5 layout) match.
- `.apm/agents.md` references the escape hatch.
- The .md sync test (added in 498febe0) continues to pass; this ticket extends it to also cover `apm.spec-writer.md` if not already.

**Cross-ticket interaction:**
- Complements 66c51e24 (worker scope hardening: disable skills + tighten prompts). 66c51e24 removes the *option* to do off-topic work; this ticket gives the worker a *legitimate alternative* when stuck. Both are needed — without the alternative, the worker has nowhere to go and either crashes or hangs.
- Complements f06272f1 (permission-denial diagnostics). f06272f1 surfaces denials post-hoc to the supervisor; this ticket lets the worker self-report blockers up front via the question/blocked state, so the supervisor sees them in the active queue rather than having to scan logs.

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
| 2026-05-01T02:34Z | — | new | philippepascal |
| 2026-05-02T03:07Z | new | groomed | philippepascal |
| 2026-05-02T03:40Z | groomed | in_design | philippepascal |
