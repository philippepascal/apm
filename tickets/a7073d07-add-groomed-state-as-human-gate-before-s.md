+++
id = "a7073d07"
title = "Add groomed state as human gate before spec work"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "apm"
agent = "63261"
branch = "ticket/a7073d07-add-groomed-state-as-human-gate-before-s"
created_at = "2026-04-01T20:26:40.952240Z"
updated_at = "2026-04-01T20:29:49.049728Z"
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

How the implementation will work.

### Open questions



### Amendment requests



## History

| When | From | To | By |
|------|------|----|----|
| 2026-04-01T20:26Z | — | new | apm |
| 2026-04-01T20:29Z | new | in_design | philippepascal |