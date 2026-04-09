+++
id = 19
title = "apm list shows closed tickets by default"
state = "closed"
priority = 7
effort = 2
risk = 1
agent = "claude-0326-2222-8071"
branch = "ticket/0019-apm-list-shows-closed-tickets-by-default"
updated_at = "2026-03-30T02:02:46.501095Z"
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
| 2026-03-27T04:09Z | specd | ready | apm |
| 2026-03-27T05:30Z | ready | in_progress | claude-0326-2222-8071 |
| 2026-03-27T05:32Z | in_progress | implemented | claude-0326-2222-8071 |
| 2026-03-27T06:33Z | implemented | accepted | apm sync |
| 2026-03-30T02:02Z | accepted | closed | apm-sync |