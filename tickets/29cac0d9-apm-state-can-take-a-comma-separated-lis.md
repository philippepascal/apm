+++
id = "29cac0d9"
title = "apm state can take a comma separated list of ids"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/29cac0d9-apm-state-can-take-a-comma-separated-lis"
created_at = "2026-06-11T06:35:11.889410Z"
updated_at = "2026-06-11T06:35:39.990084Z"
+++

## Spec

### Problem

`apm state <id> <state>` currently accepts a single ticket ID. When a supervisor or agent wants to batch-transition several tickets to the same state — e.g. marking a set of groomed tickets `ready` before dispatching workers — they must invoke the command once per ticket. This creates unnecessary friction in scripts and agent workflows.

The desired behaviour is that the ID argument accepts a comma-separated list (`apm state id1,id2,id3 ready`), transitions each ticket in turn, prints a result line per ticket, and exits non-zero if any transition failed.

### Acceptance criteria

- [ ] `apm state <id> <state>` with a single ID behaves identically to the current implementation (no regression).
- [ ] `apm state id1,id2 <state>` transitions both tickets and prints one `id: old → new` line per ticket.
- [ ] Whitespace around commas is trimmed: `apm state "id1, id2" <state>` works the same as `apm state id1,id2 <state>`.
- [ ] If one ticket in the list fails to transition, the command continues processing the remaining tickets.
- [ ] All errors are reported after all tickets are processed, and the command exits non-zero when any transition failed.
- [ ] The `id` argument description in `--help` output mentions comma-separated IDs.

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
| 2026-06-11T06:35Z | — | new | philippepascal |
| 2026-06-11T06:35Z | new | groomed | philippepascal |
| 2026-06-11T06:35Z | groomed | in_design | philippepascal |