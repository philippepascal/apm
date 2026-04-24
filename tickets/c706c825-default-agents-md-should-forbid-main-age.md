+++
id = "c706c825"
title = "Default agents.md should forbid main agent from grooming"
state = "specd"
priority = 0
effort = 1
risk = 2
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/c706c825-default-agents-md-should-forbid-main-age"
created_at = "2026-04-24T06:28:59.221174Z"
updated_at = "2026-04-24T07:20:07.228807Z"
+++

## Spec

### Problem

The default `apm-core/src/default/apm.agents.md` template contains no restriction preventing the main/delegator agent from running state transitions that are reserved for the supervisor. Without this guardrail, the main agent can create a ticket and immediately advance it through states that are supposed to be supervisor review gates — including `new → groomed`, which is where the supervisor decides whether a ticket is worth speccing. This was observed in practice in the ticker repo: the main agent routinely created *and* groomed tickets in a single pass, so the supervisor never had a chance to reject or defer them.

The fix is already proven. The ticker repo's `.apm/agents.md` adds a **Supervisor-only transitions** paragraph to the `### Main Agent` section, listing every transition the main agent must never run and the narrow set it may initiate itself. That paragraph needs to be ported verbatim into the default template so every project initialized with `apm init` gets the guardrail automatically.

This ticket depends on ticket 10791dab ("Default apm init templates should be project-agnostic"), which restructures the same file. This change adds a new content block; 10791dab should land first to avoid a merge conflict.

### Acceptance criteria

- [ ] `apm-core/src/default/apm.agents.md` contains a `**Supervisor-only transitions.**` paragraph under `### Main Agent`
- [ ] The paragraph explicitly forbids `new → groomed`
- [ ] The paragraph explicitly forbids `specd → ready` and `specd → ammend`
- [ ] The paragraph explicitly forbids `implemented → ready`, `implemented → ammend`, and `implemented → closed`
- [ ] The paragraph explicitly forbids `blocked → ready`
- [ ] The paragraph explicitly forbids `apm epic close`
- [ ] The paragraph states which transitions the main agent *may* initiate: `new → closed` (cancel a ticket created in error) and any transition the workflow marks `actionable = ["agent"]` when the agent is the assignee

### Out of scope

- Enforcing the restriction at the CLI level — no changes to the apm binary; this is instruction text only\n- Modifying existing project-specific agents.md files already deployed in user repos\n- The project-agnostic defaults changes (ticket 10791dab), which touch different content in the same file and land separately\n- Changes to the workflow state machine definitions in apm.toml

### Approach

**File to change:** `apm-core/src/default/apm.agents.md` — one insertion, no other files.

**Insertion point:** after the existing single-paragraph body of `### Main Agent` (currently ending with "…unless explicitly asked by the supervisor."), add a blank line then the following block verbatim from `ticker/.apm/agents.md` lines 31–39:

```markdown
**Supervisor-only transitions.** The following state changes are reserved for the supervisor — do not run them even when the state machine allows it, and even when you just created the ticket:

- `new → groomed` — grooming is the supervisor's review gate; leave new tickets in `new` after creation
- `specd → ready` and `specd → ammend` — spec acceptance is a supervisor review
- `implemented → ready` / `implemented → ammend` / `implemented → closed` — implementation acceptance is a supervisor review
- `blocked → ready` — unblocking requires the supervisor's answer
- Any `apm epic close` — epic PRs are opened by the supervisor

Transitions you *may* initiate for your own tickets: `new → closed` (cancel a ticket you just created in error), and any state change the workflow marks `actionable = ["agent"]` when you are the assigned agent.
```

**Ordering constraint:** ticket 10791dab touches the same file and should merge first. If 10791dab has already changed the `### Main Agent` section, confirm the insertion point still makes sense before committing; the content of the block itself does not change.

**No tests required** — the file is a markdown template, not executable code.

**Commit the change** to the ticket branch via the worktree:
```bash
git -C <worktree-path> add apm-core/src/default/apm.agents.md
git -C <worktree-path> commit -m "ticket(c706c825): add supervisor-only transitions block to default agents.md"
```

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-04-24T06:28Z | — | new | philippepascal |
| 2026-04-24T07:13Z | new | groomed | philippepascal |
| 2026-04-24T07:17Z | groomed | in_design | philippepascal |
| 2026-04-24T07:20Z | in_design | specd | claude-0424-0717-5210 |
