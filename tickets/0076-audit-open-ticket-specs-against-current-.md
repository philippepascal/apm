+++
id = 76
title = "Audit open ticket specs against current implementation"
state = "in_progress"
priority = 0
effort = 3
risk = 1
author = "claude-0329-1430-main"
agent = "claude-0329-1430-main"
branch = "ticket/0076-audit-open-ticket-specs-against-current-"
created_at = "2026-03-30T00:59:24.141610Z"
updated_at = "2026-03-30T01:05:59.822431Z"
+++

## Spec

### Problem

Several tickets in `specd`, `ready`, and `ammend` states had their specs written before the features they depend on were implemented. Now that a large batch has landed (#52–#71), some specs may describe approaches that are already superseded, reference APIs that changed shape, or have acceptance criteria that were already satisfied as a side effect of other work. A stale spec misleads the implementing agent and leads to wasted effort or incorrect implementations.

### Acceptance criteria

- [ ] Every ticket in `specd`, `ready`, `ammend`, and `question` state has been reviewed against the current codebase
- [ ] Each ticket's `### Approach` section reflects the actual current API (types, function signatures, file locations)
- [ ] Acceptance criteria that are already satisfied by existing code are noted as such (or the ticket is closed if fully done)
- [ ] No acceptance criteria silently contradict each other or the state machine config in `apm.toml`
- [ ] A brief audit note is appended to each reviewed ticket's `### Approach` (e.g. "Audited 2026-03-30: approach still valid") so the implementing agent knows the spec is current

### Out of scope

- Rewriting specs from scratch — small targeted corrections only
- Changing the acceptance criteria scope (that requires a supervisor decision)
- Tickets in `new` state (no spec yet to audit)
- Closed tickets

### Approach

For each open ticket in `specd`, `ready`, `ammend`, or `question`:

1. `apm show <id>` — read the full spec
2. Read the relevant source files for that ticket's scope
3. Check: does the `### Approach` still describe valid types/functions/paths?
4. Check: are any acceptance criteria already met by existing code?
5. If corrections needed: edit the spec in the worktree and commit
6. Append audit note to `### Approach`
7. If the ticket is entirely moot (fully implemented already): transition to `closed` via `apm close <id>` once that command exists, or flag for supervisor

Current open tickets to audit (as of writing): #35, #38, #51, #57, #63, #70, #75, #76, #77, #78.

## History

| When | From | To | By |
|------|------|----|----|
| 2026-03-30T00:59Z | — | new | claude-0329-1430-main |
| 2026-03-30T01:01Z | new | in_design | claude-0329-1430-main |
| 2026-03-30T01:03Z | in_design | specd | claude-0329-1430-main |
| 2026-03-30T01:05Z | specd | ready | claude-0329-1430-main |
| 2026-03-30T01:05Z | ready | in_progress | claude-0329-1430-main |
