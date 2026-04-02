+++
id = "b8b91642"
title = "make detail panel more reactive when other panels are updated (state of ticket selected might have changed)"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "apm"
agent = "44819"
branch = "ticket/b8b91642-make-detail-panel-more-reactive-when-oth"
created_at = "2026-04-02T22:28:29.844602Z"
updated_at = "2026-04-02T22:47:54.439219Z"
+++

## Spec

### Problem

The detail panel (TicketDetail.tsx) fetches ticket data via React Query with the key `['ticket', id]` but has no `refetchInterval` configured. This means once it fetches a ticket, it only refreshes when:
- the user explicitly interacts with the detail panel (transitions, patches), or
- the user selects a different ticket and comes back.

Meanwhile the board (SupervisorView) polls `['tickets']` every 10 seconds. When an external agent transitions a ticket's state — moving it from, say, `ready` to `in_progress` — the board card updates within 10 seconds but the detail panel remains stale indefinitely, showing the wrong state badge and a stale set of valid transition buttons.

The same staleness occurs when the user clicks Sync (Shift+S): the sync mutation invalidates `['tickets']` to refresh the board but does not invalidate `['ticket', id]`, so the detail panel still reflects the pre-sync state.

### Acceptance criteria

Checkboxes; each one independently testable.

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
| 2026-04-02T22:28Z | — | new | apm |
| 2026-04-02T22:32Z | new | groomed | apm |
| 2026-04-02T22:47Z | groomed | in_design | philippepascal |