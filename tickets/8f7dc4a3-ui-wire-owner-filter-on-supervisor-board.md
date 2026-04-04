+++
id = "8f7dc4a3"
title = "UI: wire owner filter on supervisor board and rename agent filter to owner"
state = "in_design"
priority = 0
effort = 1
risk = 0
author = "apm"
branch = "ticket/8f7dc4a3-ui-wire-owner-filter-on-supervisor-board"
created_at = "2026-04-04T06:28:20.587222Z"
updated_at = "2026-04-04T07:02:04.056414Z"
epic = "8db73240"
target_branch = "epic/8db73240-user-mgmt"
depends_on = ["2b7c4c97"]
+++

## Spec

### Problem

The supervisor board has a filter dropdown labelled "All agents" in SupervisorView.tsx. It is backed by the `agentFilter` state variable and reads `ticket.agent` to build the option list and apply the filter. However, the `Frontmatter` struct (and therefore the API response) has no `agent` field yet — that field is added by the dependency ticket #42f4b3ba. Until the dependency lands the dropdown is populated from nothing and every filter match fails silently.

Once #42f4b3ba and #2b7c4c97 land, the API will return `agent` on each ticket object. This ticket has two jobs: (1) ensure the UI is wired to that real field so the filter works, and (2) rename all user-visible labels and internal identifiers from "agent" to "owner" to match the terminology used throughout the rest of the product (the broader epic is user-management / ownership, not agent tracking).

The `Ticket` TypeScript interface in `types.ts` already has `agent?: string` which matches what the API will return, so no interface change is needed — only renaming of internal state variables, computed values, and the dropdown label.

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


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-04-04T06:28Z | — | new | apm |
| 2026-04-04T06:35Z | new | groomed | apm |
| 2026-04-04T06:59Z | groomed | in_design | philippepascal |