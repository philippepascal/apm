+++
id = "717f9b6b"
title = "UI filter error"
state = "specd"
priority = 0
effort = 2
risk = 1
author = "apm-ui"
branch = "ticket/717f9b6b-ui-filter-error"
created_at = "2026-04-04T16:05:39.699619Z"
updated_at = "2026-04-04T16:43:30.012724Z"
+++

## Spec

### Problem

On every mount (including browser refresh), `SupervisorView` fires a `useEffect` that calls `/api/me` and auto-sets `authorFilter` to the current user's username. Because `authorFilter` is local React state it always initialises to `null` on refresh, then the effect overwrites it with the username.

When the detected username does not appear as the `author` field on any ticket — common when the supervisor oversees work authored by agents (`apm`, `apm-ui`, etc.) — the `columns` memo produces zero results and the panel renders the empty state ('No tickets match the current filters') even though tickets exist.

The user's workaround is to manually change the author filter select, which clears the auto-applied value and restores visibility. Desired behaviour: the supervisor panel should default to showing all tickets on load; any author filter the user sets manually should survive a page refresh.

### Acceptance criteria

- [ ] After a browser refresh, the supervisor panel shows all tickets (no author filter applied) when no filter preference has been stored
- [ ] If the user manually sets the author filter, that choice is preserved across a browser refresh
- [ ] If the user explicitly clears the author filter (selects 'All authors'), that cleared state is preserved across a browser refresh
- [ ] The panel never shows the 'No tickets match' empty state immediately after refresh when tickets do exist in the backend

### Out of scope

- Persisting any other filter (state filter, agent filter, search text) — only author filter is the source of the reported bug
- Server-side user preferences or cross-device filter sync
- Changes to the `/api/me` endpoint or its response shape
- Changing the default behaviour of the epic filter (already in `useLayoutStore`, not affected)

### Approach

File: `apm-ui/src/components/supervisor/SupervisorView.tsx`

1. **Remove the `/api/me` auto-init effect** (lines 77-86). This is the sole source of the bug; the auto-detected username often mismatches ticket `author` values and the supervisor view should default to showing all tickets.

2. **Persist `authorFilter` in `localStorage`** so a manually-chosen filter survives refresh:
   - Change the `useState` initialiser to read from `localStorage`:
     ```ts
     const [authorFilter, setAuthorFilter] = useState<string | null>(() => {
       return localStorage.getItem('apm.authorFilter') ?? null
     })
     ```
   - Wrap the setter (or add a `useEffect` on `authorFilter`) to write back:
     ```ts
     useEffect(() => {
       if (authorFilter === null) localStorage.removeItem('apm.authorFilter')
       else localStorage.setItem('apm.authorFilter', authorFilter)
     }, [authorFilter])
     ```

No other files need to change. The `availableAuthors` memo and select rendering are already correct once the bad auto-init is removed.

**Order of changes:**
1. Delete the `useEffect` block that imports nothing (the `/api/me` call)
2. Update `useState` initialiser to lazy-read `localStorage`
3. Add the persistence `useEffect`

**Constraint:** `localStorage` access in the `useState` initialiser runs only once on mount (lazy init), so there is no SSR concern and no extra re-render.

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-04-04T16:05Z | — | new | apm-ui |
| 2026-04-04T16:39Z | new | groomed | apm |
| 2026-04-04T16:40Z | groomed | in_design | philippepascal |
| 2026-04-04T16:43Z | in_design | specd | claude-0404-1640-s7w2 |
