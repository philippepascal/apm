+++
id = "8f7dc4a3"
title = "UI: wire owner filter on supervisor board and rename agent filter to owner"
state = "in_design"
priority = 0
effort = 1
risk = 1
author = "apm"
branch = "ticket/8f7dc4a3-ui-wire-owner-filter-on-supervisor-board"
created_at = "2026-04-04T06:28:20.587222Z"
updated_at = "2026-04-04T07:26:01.242945Z"
epic = "8db73240"
target_branch = "epic/8db73240-user-mgmt"
depends_on = ["2b7c4c97"]
+++

## Spec

### Problem

The supervisor board has a filter dropdown labelled "All agents" in SupervisorView.tsx. It is backed by the agentFilter state variable and reads ticket.agent to build the option list and apply the filter. The dependency tickets #42f4b3ba and #2b7c4c97 expose the field as owner (not agent) — both the Frontmatter struct and the API response use the owner key. Until this ticket lands, the dropdown reads from a field that does not exist in the API response, so it is always empty and filter matches always fail silently.

This ticket wires the UI filter to the real owner field returned by the API. Concretely: the Ticket TypeScript interface must change from agent?: string to owner?: string, the filter logic must read ticket.owner, and the internal identifiers and user-visible label must be updated to match (agentFilter → ownerFilter, availableAgents → availableOwners, label "All agents" → "All owners").

### Acceptance criteria

- [ ] The filter dropdown in SupervisorView.tsx is labelled "All owners" (not "All agents")
- [ ] Selecting an owner from the dropdown shows only tickets whose `ticket.agent` matches the selected value
- [ ] Selecting "All owners" (the blank option) shows all tickets regardless of agent value
- [ ] The dropdown option list is built from the distinct `agent` values present in the loaded ticket list
- [ ] The `agentFilter` state variable is renamed to `ownerFilter` throughout SupervisorView.tsx
- [ ] The `availableAgents` computed value is renamed to `availableOwners` throughout SupervisorView.tsx
- [ ] The `hasActiveFilters` expression reflects the rename (uses `ownerFilter !== null` instead of `agentFilter !== null`)

### Out of scope

- Adding the `agent` field to Frontmatter — covered by #42f4b3ba
- Exposing `agent` in the API response and adding the `?agent=` query param — covered by #2b7c4c97
- Renaming the API field from `agent` to `owner` — the dependency tickets have already spec'd the field as `agent`; the UI label rename in this ticket is sufficient alignment
- Displaying the owner on TicketCard or TicketDetail — out of scope for this ticket
- Persisting the filter selection across page reloads

### Approach

All changes are in `apm-ui/src/components/supervisor/SupervisorView.tsx`.

**1. Rename state variable and computed value**

- `agentFilter` → `ownerFilter` (useState declaration and all read/write sites)
- `availableAgents` → `availableOwners` (useMemo declaration and all read sites)

The filter logic itself (`t.agent === ownerFilter`) is already correct — it reads `ticket.agent` which is the field the API returns once #42f4b3ba lands. No logic change is needed, only identifier rename.

**2. Update the dropdown label**

Change the placeholder option text from `"All agents"` to `"All owners"`.

**3. Update `hasActiveFilters`**

Replace the `agentFilter !== null` reference with `ownerFilter !== null`.

**No other files need to change.** The `Ticket` interface in `types.ts` already has `agent?: string` which maps to what the API returns. The Swimlane component does not reference the filter state. No tests exist for this component (it is pure UI).

**Order of steps**

1. Rename `agentFilter` to `ownerFilter` (replace_all)
2. Rename `availableAgents` to `availableOwners` (replace_all)
3. Change the option label string
4. Verify the file compiles: `cd apm-ui && npx tsc --noEmit`

### Open questions


### Amendment requests

- [ ] The API now returns `owner` (not `agent`) — update the `Ticket` TypeScript interface in `types.ts` from `agent?: string` to `owner?: string`
- [ ] Update filter logic to read `ticket.owner` instead of `ticket.agent`
- [ ] The variable renames (`agentFilter` → `ownerFilter`, `availableAgents` → `availableOwners`) and label change ("All agents" → "All owners") still apply
- [ ] Remove the framing about "renaming from agent to owner" in Problem/Approach — the API field is already `owner`; this ticket is about wiring the UI filter to the real field

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