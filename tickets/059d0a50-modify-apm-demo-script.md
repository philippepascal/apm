+++
id = "059d0a50"
title = "modify apm-demo script"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/059d0a50-modify-apm-demo-script"
created_at = "2026-04-08T23:59:41.422002Z"
updated_at = "2026-04-09T00:14:56.346066Z"
+++

## Spec

### Problem

The `scripts/create-demo.sh` script builds a demo APM repository around "jot," a minimal Rust CLI notes tool. It currently creates 14 tickets covering all 11 workflow states, with one epic ("Search feature") containing 3 tickets.

The demo needs to better showcase APM's epic management, dependency graphs, and the full implemented→closed lifecycle. Specifically: there is only one epic, the ticket count is modest for a realistic project, and the `implemented` state appears only once (T3 — list notes). Users exploring the demo get an incomplete picture of a healthy project backlog.

The goal is to extend the script to add a second epic (7 tickets with intra-epic dependencies), double the count of all non-new-epic tickets from 14 to 28 (by adding 14 more standalone tickets), and ensure multiple tickets reach `implemented` state. All new content must remain coherent with the jot project.

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
| 2026-04-08T23:59Z | — | new | philippepascal |
| 2026-04-08T23:59Z | new | groomed | apm |
| 2026-04-09T00:14Z | groomed | in_design | philippepascal |