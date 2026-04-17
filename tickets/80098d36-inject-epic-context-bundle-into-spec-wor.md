+++
id = "80098d36"
title = "Inject epic context bundle into spec workers"
state = "in_progress"
priority = 0
effort = 4
risk = 3
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/80098d36-inject-epic-context-bundle-into-spec-wor"
created_at = "2026-04-17T07:27:05.870212Z"
updated_at = "2026-04-17T07:38:26.047357Z"
epic = "35199c7f"
target_branch = "epic/35199c7f-give-workers-cross-ticket-context"
+++

## Spec

### Problem

When a spec worker is spawned (at `in_design`, from either `groomed` or `ammend`), it sees only its own ticket. It doesn't know what the broader epic is trying to accomplish or what sibling tickets are claiming. As a result, specs drift out of scope, duplicate work that a sibling will do, or miss acceptance criteria that are only obvious from the epic's shape. This is the dominant cause of amendment cycles during the spec phase.

### Acceptance criteria

- [x] When a spec worker is spawned on a ticket that belongs to an epic, APM generates an epic context bundle (markdown) and prepends it to the worker's prompt.
- [x] The bundle contains: the epic's title, goal, and non-goals (from `EPIC.md` / the epic file); a list of sibling tickets grouped by state, each showing title, one-line Problem summary, and the full "Out of scope" section if present.
- [x] The bundle explicitly frames the purpose to the worker: "use this to scope your ticket — do not duplicate or overreach into sibling tickets' territory."
- [x] Tickets not in any epic spawn spec workers with no bundle (unchanged behaviour).
- [x] Bundle is capped at a configurable sibling count and byte size; older closed siblings are elided with a count when the cap is hit.
- [ ] Integration test assembles a bundle against a fixture epic with mixed-state siblings.

### Out of scope

- Dependency information — handled by the sibling ticket for the dependency context bundle.
- File overlap warnings — rejected at the epic level; not handled here.
- Smart selection of *which* siblings are most relevant — include all, subject to the cap.

### Approach

- Add `apm-core::context::build_epic_bundle(epic_id, current_ticket_id)` returning a `String`.
- Hook into the spawn path used at `in_design` transitions (both `groomed → in_design` and `ammend → in_design`).
- Data sources already exist in the ticket store; no new storage.

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-04-17T07:27Z | — | new | philippepascal |
| 2026-04-17T07:33Z | new | groomed | claude-0417-1430-c7a2 |
| 2026-04-17T07:33Z | groomed | in_design | claude-0417-1430-c7a2 |
| 2026-04-17T07:35Z | in_design | specd | claude-0417-1430-c7a2 |
| 2026-04-17T07:37Z | specd | ready | apm |
| 2026-04-17T07:38Z | ready | in_progress | philippepascal |