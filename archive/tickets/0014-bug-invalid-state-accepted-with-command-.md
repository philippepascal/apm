+++
id = 14
title = "apm state accepts any string as state without validation"
state = "closed"
priority = 8
effort = 2
risk = 2
updated_at = "2026-03-27T00:06:00.986851Z"
+++

## Spec

### Amendment requests

- [x] create new ticket for first item out of scope

  Created ticket #20: "apm state enforces valid transitions from state machine config"

### Problem

`apm state <id> <state>` accepts any arbitrary string as the new state with no
validation. Running `apm state 12 rready` silently writes `state = "rready"` to
the ticket file. The ticket is then invisible to `apm next` and all state-based
filtering. Silent corruption.

### Acceptance criteria

- [ ] `apm state <id> <state>` rejects state values not present in `[[workflow.states]]` in `apm.toml`
- [ ] Error message names the invalid value and lists valid states: `unknown state "rready" — valid states: new, question, specd, ammend, ready, in_progress, implemented, accepted, closed`
- [ ] Exit code is non-zero on validation failure
- [ ] No file is modified if validation fails

### Out of scope

- Enforcing valid state *transitions* (from→to rules) — that is a separate, larger ticket
- Validating states on ticket load (surfaced as a verify check in #5)

### Approach

In `cmd/state.rs`, after loading the config, collect the set of valid state ids from
`config.workflow.states`. If the requested state is not in the set, print the error
and return early before loading or modifying any ticket file.

## History

| Date | Actor | Transition | Note |
|------|-------|------------|------|
| 2026-03-25 | manual | new → specd | |
| 2026-03-26 | manual | ammend → specd | |
| 2026-03-26 | manual | specd → ready | |
| 2026-03-27T00:06Z | ready | closed | apm |