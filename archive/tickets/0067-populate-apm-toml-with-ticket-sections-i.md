+++
id = 67
title = "Populate apm.toml with ticket.sections, instructions, context_section, focus_section"
state = "closed"
priority = 5
effort = 1
risk = 1
author = "claude-0329-1430-main"
agent = "claude-0329-1430-main"
branch = "ticket/0067-populate-apm-toml-with-ticket-sections-i"
created_at = "2026-03-29T23:26:16.251460Z"
updated_at = "2026-03-30T02:02:46.501095Z"
+++

## Spec

### Problem

The config schema supports `[[ticket.sections]]`, `instructions` on states, `context_section` and `focus_section` on transitions — all parsed correctly — but none are present in `apm.toml`. The fields are dead config until declared.

Without `[[ticket.sections]]`, `apm new` uses a hardcoded body template and `apm spec` rejects "Amendment requests" and "Code review". Without `instructions`, `apm start --next` cannot pass the right system prompt to spawned agents. Without `context_section`, `apm new --context` always routes to "Problem". Without `focus_section` on `implemented → ready`, the supervisor's code review feedback has no structured path back to the worker agent.

### Acceptance criteria

- [x] `apm.toml` defines 7 `[[ticket.sections]]` entries: Problem (free, required), Acceptance criteria (tasks, required), Out of scope (free, required), Approach (free, required), Open questions (qa, optional), Amendment requests (tasks, optional), Code review (tasks, optional); required sections have a `placeholder`
- [x] The `in_design` state entry has `instructions = "apm.spec-writer.md"`
- [x] The `in_progress` state entry has `instructions = "apm.worker.md"`
- [x] The `new → in_design` transition has `context_section = "Problem"`
- [x] The `implemented → ready` transition has `focus_section = "Code review"`
- [x] `cargo test --workspace` passes after the changes

### Out of scope

- Creating `apm.spec-writer.md` or `apm.worker.md` (tickets #61 and #62)
- Runtime behaviour changes (tickets #64, #65, #66)
- Changing transition actors, preconditions, or side effects

### Approach

Edit `apm.toml` directly and commit to `main`:

1. Add 7 `[[ticket.sections]]` entries after the `[tickets]` block.
2. Add `instructions = "apm.spec-writer.md"` to the `[[workflow.states]]` entry where `id = "in_design"`.
3. Add `instructions = "apm.worker.md"` to the entry where `id = "in_progress"`.
4. Add `context_section = "Problem"` to the `[[workflow.states.transitions]]` under `new` where `to = "in_design"`.
5. Add `focus_section = "Code review"` to the `[[workflow.states.transitions]]` under `implemented` where `to = "ready"`.

## History

| When | From | To | By |
|------|------|----|----|
| 2026-03-29T23:26Z | — | new | claude-0329-1430-main |
| 2026-03-29T23:26Z | new | in_design | claude-0329-1430-main |
| 2026-03-29T23:31Z | in_design | specd | claude-0329-1430-main |
| 2026-03-29T23:52Z | specd | ready | apm |
| 2026-03-29T23:56Z | ready | in_progress | claude-0329-1430-main |
| 2026-03-30T00:00Z | in_progress | implemented | claude-0329-1430-main |
| 2026-03-30T00:50Z | implemented | accepted | apm |
| 2026-03-30T02:02Z | accepted | closed | apm-sync |