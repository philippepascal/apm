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


### Out of scope

Explicit list of what this ticket does not cover.

### Approach

How the implementation will work.

### Open questions



### Amendment requests



## History

| When | From | To | By |
|------|------|----|----|
| 2026-04-01T20:26Z | — | new | apm |
| 2026-04-01T20:29Z | new | in_design | philippepascal |