+++
id = "8f7dc4a3"
title = "UI: wire owner filter on supervisor board and rename agent filter to owner"
state = "in_progress"
priority = 0
effort = 2
risk = 1
author = "apm"
branch = "ticket/8f7dc4a3-ui-wire-owner-filter-on-supervisor-board"
created_at = "2026-04-04T06:28:20.587222Z"
updated_at = "2026-04-04T17:04:26.149300Z"
epic = "8db73240"
target_branch = "epic/8db73240-user-mgmt"
depends_on = ["2b7c4c97"]
+++

## Spec

### Problem

The supervisor board has a filter dropdown labelled "All agents" in SupervisorView.tsx. It is backed by the agentFilter state variable and reads ticket.agent to build the option list and apply the filter. The dependency tickets #42f4b3ba and #2b7c4c97 expose the field as owner (not agent) — both the Frontmatter struct and the API response use the owner key. Until this ticket lands, the dropdown reads from a field that does not exist in the API response, so it is always empty and filter matches always fail silently.

This ticket wires the UI filter to the real owner field returned by the API. Concretely: the Ticket TypeScript interface must change from agent?: string to owner?: string, the filter logic must read ticket.owner, and the internal identifiers and user-visible label must be updated to match (agentFilter → ownerFilter, availableAgents → availableOwners, label "All agents" → "All owners").

### Acceptance criteria

- [x] The filter dropdown in SupervisorView.tsx is labelled "All owners" (not "All agents")
- [x] Selecting an owner from the dropdown shows only tickets whose ticket.owner matches the selected value
- [x] Selecting "All owners" (the blank option) shows all tickets regardless of owner value
- [ ] The dropdown option list is built from the distinct owner values present in the loaded ticket list
- [ ] The Ticket TypeScript interface in types.ts has owner?: string (not agent?: string)
- [ ] The agentFilter state variable is renamed to ownerFilter throughout SupervisorView.tsx
- [ ] The availableAgents computed value is renamed to availableOwners throughout SupervisorView.tsx
- [ ] The hasActiveFilters expression uses ownerFilter !== null (not agentFilter !== null)

### Out of scope

- Adding the owner field to Frontmatter — covered by #42f4b3ba
- Exposing owner in the API response and adding the ?owner= query param — covered by #2b7c4c97
- Displaying the owner on TicketCard or TicketDetail
- Persisting the filter selection across page reloads

### Approach

Two files change.

**1. apm-ui/src/types.ts**

Change the Ticket interface field from agent?: string to owner?: string. This aligns the TypeScript model with what the API returns (the owner key, as defined by #42f4b3ba and exposed by #2b7c4c97).

**2. apm-ui/src/components/supervisor/SupervisorView.tsx**

Rename state variable and computed value:
- agentFilter → ownerFilter (useState declaration and all read/write sites, use replace_all)
- availableAgents → availableOwners (useMemo declaration and all read sites, use replace_all)

Update filter logic: change t.agent === ownerFilter to t.owner === ownerFilter (and the useMemo that builds the list: t.owner instead of t.agent).

Update the dropdown label: change "All agents" to "All owners".

hasActiveFilters is updated automatically by the agentFilter → ownerFilter rename.

**No other files need to change.**

**Order of steps**
1. Edit types.ts: agent?: string → owner?: string
2. Edit SupervisorView.tsx: replace_all agentFilter → ownerFilter
3. Edit SupervisorView.tsx: replace_all availableAgents → availableOwners
4. Edit SupervisorView.tsx: replace t.agent with t.owner in filter logic and option-list memo
5. Edit SupervisorView.tsx: change label string "All agents" → "All owners"
6. Verify: cd apm-ui && npx tsc --noEmit

### Open questions


### Amendment requests

- [x] The API now returns `owner` (not `agent`) — update the `Ticket` TypeScript interface in `types.ts` from `agent?: string` to `owner?: string`
- [x] Update filter logic to read `ticket.owner` instead of `ticket.agent`
- [x] The variable renames (`agentFilter` → `ownerFilter`, `availableAgents` → `availableOwners`) and label change ("All agents" → "All owners") still apply
- [x] Remove the framing about "renaming from agent to owner" in Problem/Approach — the API field is already `owner`; this ticket is about wiring the UI filter to the real field

### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-04-04T06:28Z | — | new | apm |
| 2026-04-04T06:35Z | new | groomed | apm |
| 2026-04-04T06:59Z | groomed | in_design | philippepascal |
| 2026-04-04T07:02Z | in_design | specd | claude-0403-0700-b2e4 |
| 2026-04-04T07:15Z | specd | ammend | apm |
| 2026-04-04T07:26Z | ammend | in_design | philippepascal |
| 2026-04-04T07:28Z | in_design | specd | claude-0404-0730-spec1 |
| 2026-04-04T15:34Z | specd | ready | apm |
| 2026-04-04T17:04Z | ready | in_progress | philippepascal |