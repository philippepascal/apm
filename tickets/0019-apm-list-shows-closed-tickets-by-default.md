+++
id = 19
title = "apm list shows closed tickets by default"
state = "implemented"
priority = 7
effort = 2
risk = 1
branch = "ticket/0019-apm-list-shows-closed-tickets-by-default"
created = "2026-03-26"
updated = "2026-03-26"
+++

## Spec

### Problem

`apm list` shows all tickets including those in terminal states (e.g. `closed`).
As tickets accumulate, the board becomes noisy with done work. The spec says
terminal-state tickets are excluded from the board by default; they should only
appear when explicitly requested.

### Acceptance criteria

- [ ] `apm list` excludes tickets whose state matches a `terminal = true` state in `[[workflow.states]]`
- [ ] `apm list --all` includes terminal-state tickets
- [ ] When `[[workflow.states]]` is empty (no config), all tickets are shown (safe fallback)
- [ ] `--state <s>` filter still works alongside the terminal exclusion

### Out of scope

- Archiving closed tickets to a separate directory

### Approach

In `cmd/list.rs`, after loading config, collect the set of terminal state ids from
`config.workflow.states`. In the filter closure, add `!terminal_states.contains(&fm.state)`
unless `--all` is passed. Add `--all: bool` to the `List` variant in `main.rs`.

## History

| Date | Actor | Transition | Note |
|------|-------|------------|------|
| 2026-03-26 | manual | new → specd | |
| 2026-03-26 | agent | ready → implemented | |
