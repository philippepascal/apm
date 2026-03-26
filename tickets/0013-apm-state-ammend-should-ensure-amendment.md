+++
id = 13
title = "apm state ammend should ensure Amendment requests section exists in spec"
state = "closed"
priority = 10
effort = 2
risk = 1
agent = "claude-0325-2043-a970"
branch = "feature/13-ammend-inserts-amendment-section"
created = "2026-03-25"
updated = "2026-03-25"
+++

## Spec

### Problem

When a supervisor runs `apm state <id> ammend`, they need a place to write their
amendment requests. The `### Amendment requests` subsection is defined in
TICKET-SPEC.md for exactly this purpose, but the `apm state ammend` command does
not add it if it is missing. The supervisor has to edit the file manually and add
the section, which is friction and not obvious. We experienced this directly on
ticket #2.

### Acceptance criteria

- [ ] When `apm state <id> ammend` is run, if `### Amendment requests` is absent from `## Spec`, it is inserted automatically
- [ ] The section is inserted after `### Out of scope` (or at the end of `## Spec` if `### Out of scope` is absent)
- [ ] If `### Amendment requests` already exists, the command does not add a duplicate
- [ ] The inserted section contains a single placeholder line: `<!-- Add amendment requests below -->`
- [ ] Existing spec content is not modified

### Out of scope

- Adding `### Open questions` automatically on `apm state question` (separate ticket if needed)
- Modifying the `apm new` template (optional sections are intentionally absent by default)

### Approach

In `cmd/state.rs`, after applying the state transition, check if the new state is
`ammend`. If so, call `ensure_amendment_section(&mut body)` which:
1. Returns early if `### Amendment requests` already present
2. Finds insertion point: end of `### Out of scope` block, or end of `## Spec` block
3. Inserts the section with placeholder

## History

| Date | Actor | Transition | Note |
|------|-------|------------|------|
| 2026-03-25 | manual | new → specd | |
| 2026-03-25 | manual | specd → ready | |
| 2026-03-25 | manual | ready → in_progress | |
| 2026-03-25 | manual | in_progress → in_progress | |
| 2026-03-25 | manual | in_progress → implemented | |
| 2026-03-25 | manual | implemented → accepted | |
| 2026-03-25 | manual | accepted → closed | |
