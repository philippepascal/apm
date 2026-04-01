+++
id = "a7073d07"
title = "Add groomed state as human gate before spec work"
state = "in_design"
priority = 0
effort = 3
risk = 2
author = "apm"
agent = "63261"
branch = "ticket/a7073d07-add-groomed-state-as-human-gate-before-s"
created_at = "2026-04-01T20:26:40.952240Z"
updated_at = "2026-04-01T20:32:09.562565Z"
+++

## Spec

### Problem

Currently agents pick up tickets directly from the `new` state for spec writing, with no human triage gate. A ticket creator (engineer or another agent) can create a ticket and it immediately becomes agent-actionable — an agent may start writing a spec before a supervisor has reviewed whether the ticket is worth pursuing, is well-scoped, or belongs in the current queue.

The `ready` state already serves as a human gate before implementation work (supervisor reviews the spec and explicitly promotes to `ready`). There is no equivalent gate before spec work.

Adding a `groomed` state between `new` and `in_design` mirrors the existing pattern: supervisors triage `new` tickets into `groomed` when they want spec work to begin. Agents only pick up `groomed` tickets. Tickets created with `apm new` continue to start in `new` and wait for supervisor promotion.

### Acceptance criteria

- [ ] `apm next` does not return `new` tickets as actionable for agents
- [ ] `apm next` returns `groomed` tickets as actionable for agents
- [ ] `apm start --next` picks up a `groomed` ticket and transitions it to `in_design`
- [ ] A ticket in `new` state cannot be transitioned to `in_design` via `apm start`
- [ ] A supervisor can transition a ticket from `new` to `groomed` via `apm state <id> groomed`
- [ ] `apm init` generates a `config.toml` that includes the `groomed` state
- [ ] A spawned spec-writer worker receives the spec-writer system prompt and role prefix when the ticket pre-transition state is `groomed`
- [ ] A spawned spec-writer worker still receives the spec-writer system prompt when the ticket pre-transition state is `ammend`
- [ ] After a supervisor answers a question (`question → groomed`), the ticket is agent-actionable again

### Out of scope

- Changes to the `apm new` command — tickets still start in `new`
- UI or dashboard changes to surface the `groomed` state
- Bulk-grooming commands (e.g. `apm groom --all`)
- Any change to the implementation half of the workflow (`ready`, `in_progress`, etc.)
- Renaming or removing the `question` state or its existing transitions other than the `question → new` target

### Approach

Four files change. Order does not matter — they are independent edits.

**1. `.apm/config.toml` — workflow state machine**

- Remove `actionable = ["agent"]` from the `new` state block. Keep the `new → closed` manual/supervisor transition. Remove the `new → in_design` (command:start / agent) transition entirely.
- Add a new `groomed` state block after `new` (layer = 1):
  ```toml
  [[workflow.states]]
  id           = "groomed"
  label        = "Groomed"
  color        = "#6366f1"
  layer        = 1
  actionable   = ["agent"]
  instructions = ".apm/apm.spec-writer.md"

    [[workflow.states.transitions]]
    to      = "in_design"
    trigger = "command:start"
    actor   = "agent"
    context_section = "Problem"

    [[workflow.states.transitions]]
    to      = "closed"
    trigger = "manual"
    actor   = "supervisor"
  ```
- Add a `new → groomed` transition inside the `new` state block:
  ```toml
    [[workflow.states.transitions]]
    to      = "groomed"
    trigger = "manual"
    actor   = "supervisor"
  ```
- Change the `question` state's `to = "new"` transition target to `to = "groomed"` so that answered questions return to the agent-actionable state (not the gated `new` state, which would stall the ticket).

**2. `apm-core/src/start.rs` — spec-writer role detection**

Two functions hardcode which pre-transition states map to the spec-writer role. Replace `"new"` with `"groomed"` in both arrays (the `ammend` entry is unchanged):

- `resolve_system_prompt`: line ~600 — change `["new", "ammend"]` to `["groomed", "ammend"]`
- `agent_role_prefix`: line ~613 — same change

Update the two unit tests that assert on `"new"`:
- `resolve_system_prompt_uses_spec_writer_for_new` → test with `"groomed"` instead
- `agent_role_prefix_spec_writer_for_new` → test with `"groomed"` instead

**3. `apm-core/src/init.rs` — generated config template**

`apm init` writes a starter `config.toml`. Find the hardcoded workflow template string (contains `[[workflow.states]]\nid = \"new\"`) and add the `groomed` state block after `new`, mirroring the changes in step 1. Also update the integration test at `apm/tests/integration.rs` that asserts on the list of state names (line ~187) to include `"groomed"`.

**4. `.apm/agents.md` — agent instructions**

- Under **Startup**, change the description of `apm next` / actionable states to mention `groomed` instead of `new` as the spec-writer entry point.
- Under **Working a ticket**, rename the `state = `new`` section to `state = `groomed`` and update the prose and commands accordingly (the workflow is identical; only the state name changes).
- Update the Delegator section note about which tickets are blocking to reflect that `new` is now supervisor-only and `groomed` is the agent-actionable spec state.

**Gotcha — existing tickets in `new`:** Any live ticket currently in `new` will lose agent-actionability after the config change. Supervisors will need to manually transition those tickets to `groomed`. No migration code is needed — the config change is enough and the data model (TOML frontmatter) is unaffected.

### Open questions



### Amendment requests



## History

| When | From | To | By |
|------|------|----|----|
| 2026-04-01T20:26Z | — | new | apm |
| 2026-04-01T20:29Z | new | in_design | philippepascal |