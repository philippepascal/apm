+++
id = "ed4838d6"
title = "add epic id in apm-server epic drop-down"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/ed4838d6-add-epic-id-in-apm-server-epic-drop-down"
created_at = "2026-05-28T05:54:59.309955Z"
updated_at = "2026-05-28T06:18:24.295451Z"
+++

## Spec

### Problem

The epic filter dropdown in `apm-server`'s SupervisorView renders each option as `ep.title || ep.id` — showing only the title, or the full raw UUID as fallback. When multiple epics have similar titles, or when the user wants to confirm which epic matches an ID they see elsewhere in the UI (e.g. the 8-char epic chip on TicketCard), the dropdown gives no way to cross-reference.

The TicketCard already displays the first 8 characters of the epic ID as a chip. The dropdown should be consistent: show the 8-char ID prefix alongside the title for every epic option.

### Acceptance criteria

- [ ] Each epic option in the dropdown shows the first 8 characters of the epic ID followed by the title (e.g. `abcd1234 · My Epic Title`)
- [ ] When an epic has no title, the option shows only the 8-char ID with no trailing separator
- [ ] The "All epics" and "No epic" options are unchanged
- [ ] Selecting an epic from the dropdown still filters tickets correctly (the option `value` remains the full ID)

### Out of scope

- The TicketCard epic chip (already shows 8-char ID; no change needed)
- Backend API response shape for `/api/epics`
- Epic filtering logic (no behaviour change, display only)
- The NewEpicModal or any other epic-related UI component

### Approach

Edit one line in `apm-ui/src/components/supervisor/SupervisorView.tsx` (line 241).

Current:
```tsx
<option key={ep.id} value={ep.id}>{ep.title || ep.id}</option>
```

Replace with:
```tsx
<option key={ep.id} value={ep.id}>{ep.id.slice(0, 8)}{ep.title ? ` · ${ep.title}` : ''}</option>
```

The `value` attribute stays `ep.id` (full UUID), so the existing filter logic (`t.epic === epicFilter`) is unaffected. The visible text becomes `abcd1234 · My Epic Title` when a title is present, or `abcd1234` when it is not — matching the convention already used in TicketCard.

No backend changes. No test changes needed (this is a pure display mutation on a React option element).

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-05-28T05:54Z | — | new | philippepascal |
| 2026-05-28T06:09Z | new | groomed | philippepascal |
| 2026-05-28T06:18Z | groomed | in_design | philippepascal |