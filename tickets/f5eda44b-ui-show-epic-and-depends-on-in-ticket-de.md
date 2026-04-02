+++
id = "f5eda44b"
title = "UI: show epic and depends_on in ticket detail panel"
state = "in_design"
priority = 2
effort = 0
risk = 0
author = "claude-0401-2145-a8f3"
agent = "23932"
branch = "ticket/f5eda44b-ui-show-epic-and-depends-on-in-ticket-de"
created_at = "2026-04-01T21:56:10.584818Z"
updated_at = "2026-04-02T00:53:14.903037Z"
+++

## Spec

### Problem

The ticket detail panel (`apm-ui/src/components/TicketDetail.tsx`) renders core fields — title, state, effort, risk, priority — but has no awareness of `epic` or `depends_on`. Engineers cannot tell from the UI which epic a ticket belongs to, or which tickets it is waiting on before it can be dispatched.

The underlying `Frontmatter` struct in `apm-core/src/ticket.rs` does not yet declare `epic` or `depends_on` fields, so they are stripped during parsing even when present in the TOML frontmatter. Adding them to the struct is the minimal server-side change needed; the `TicketDetailResponse` already flattens `Frontmatter`, so the new fields will appear in the API automatically.

On the UI side, two small features are needed in the detail panel header: a clickable epic label (that sets the epic filter on the supervisor board so engineers can quickly scope to the same epic) and a dependency list (ticket IDs that link to each dep's detail panel, with strikethrough on resolved deps).

### Acceptance criteria

- [ ] When a ticket has no `epic` field, the detail panel shows no epic row
- [ ] When a ticket has an `epic` field, the detail panel shows a labelled row with the epic ID value
- [ ] Clicking the epic label sets `epicFilter` in the layout store to that epic ID
- [ ] When `epicFilter` is set in the layout store, the supervisor board hides tickets whose `epic` field does not match (tickets with no `epic` field are also hidden)
- [ ] Clicking the epic label a second time on the same ticket while the filter is already active clears the filter (toggle behaviour)
- [ ] When a ticket has no `depends_on` field (or an empty array), the detail panel shows no dependencies row
- [ ] When a ticket has a `depends_on` field, the detail panel lists each dep ticket ID
- [ ] Clicking a dep ticket ID in the detail panel sets `selectedTicketId` in the layout store to that dep's full ID, opening its detail panel
- [ ] Dep tickets whose state is `implemented`, `accepted`, or `closed` are shown with strikethrough text
- [ ] Dep tickets whose state is any other value are shown without strikethrough
- [ ] A dep ticket ID that does not resolve to any known ticket renders as plain text (no link, no crash)
- [ ] Existing tickets without `epic` or `depends_on` in their frontmatter continue to load and display correctly

### Out of scope

- `target_branch` frontmatter field (used by `apm start` for epic branching, not a UI concern here)
- Epic creation, listing, or management commands (`apm epic new`, `apm epic list`, `apm epic show`, `apm epic close`)
- New-ticket modal epic/depends_on input fields (separate ticket)
- Lock icon on ticket cards in the queue or supervisor board for unresolved deps (separate ticket)
- Epic column in the priority queue panel (separate ticket)
- Engine scheduling changes that block dispatch on `depends_on` (separate ticket)
- A standalone epic filter dropdown control on the supervisor board — the filter is set exclusively by clicking the epic label in the detail panel
- `GET /api/epics` server routes — no epic-specific API routes are needed for this feature

### Approach

How the implementation will work.

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-04-01T21:56Z | — | new | claude-0401-2145-a8f3 |
| 2026-04-01T22:01Z | new | groomed | claude-0401-2145-a8f3 |
| 2026-04-02T00:53Z | groomed | in_design | philippepascal |