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

- [ ] The script creates exactly 2 epics
- [ ] The new epic contains exactly 7 tickets (assigned via `--epic`)
- [ ] The new epic tickets include at least 3 intra-epic dependency edges (via `--depends-on`)
- [ ] The total ticket count after the script runs is 35 (28 non-new-epic + 7 new-epic)
- [ ] At least 4 tickets across the whole demo are in `implemented` state
- [ ] Every new ticket title describes a plausible jot feature or fix
- [ ] Every ticket in `specd`, `implemented`, or `closed` state has all four spec sections populated (Problem, Acceptance criteria, Out of scope, Approach)
- [ ] The script runs end-to-end without errors on a clean GitHub account
- [ ] The README is updated to reflect 35 tickets and 2 epics

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