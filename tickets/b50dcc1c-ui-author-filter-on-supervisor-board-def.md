+++
id = "b50dcc1c"
title = "UI: author filter on supervisor board, default to current user from /api/me"
state = "closed"
priority = 0
effort = 3
risk = 2
author = "apm"
branch = "ticket/b50dcc1c-ui-author-filter-on-supervisor-board-def"
created_at = "2026-04-02T20:54:34.590380Z"
updated_at = "2026-04-04T06:02:07.327231Z"
epic = "8db73240"
target_branch = "epic/8db73240-user-mgmt"
depends_on = ["90ebf40b", "e2e3d958", "70d58b2d"]
+++

## Spec

### Problem

The supervisor board shows all tickets from all authors with no filtering. In a multi-collaborator project, or when automated agents have created tickets (`author = "apm"`), the board is noisy. A developer cannot focus on their own work without manually scanning through unrelated tickets.

DESIGN-users.md point 8 specifies the desired behaviour: on load, the board defaults to showing only tickets where `author` matches the current user (fetched from `GET /api/me`), with an explicit control to reveal all authors. This default is also useful for solo developers: it filters out the noise of agent-authored side notes and automated tickets.

The `author` field is already present in ticket frontmatter and will be guaranteed present in API responses by ticket #90ebf40b. The `GET /api/me` endpoint is established by #90ebf40b (localhost case) and extended by #e2e3d958 (session-authenticated case). This ticket is purely a UI change: add the author filter control to the supervisor board and wire the default to `/api/me`.

### Acceptance criteria

- [x] On load, the board fetches `GET /api/me` and uses the returned `username` as the initial author filter value
- [x] When `/api/me` returns `"unassigned"`, the author filter is left unset ("Show all authors" mode), not filtered to "unassigned"
- [x] An "Author" dropdown appears in the filter bar showing all unique author values from the currently loaded ticket set
- [x] The author dropdown has a "Show all authors" option that clears the filter
- [x] When an author is selected in the dropdown, only tickets with a matching `author` value are shown on the board
- [x] The author filter composes with the existing state, agent, epic, and search filters using AND logic
- [x] When the author filter is active (single author selected), ticket cards do not display the author label
- [x] When "Show all authors" is active, ticket cards display the author value in small subdued text
- [x] The `Ticket` TypeScript interface includes an `author` field (string)
- [x] If `/api/me` fails (network error or non-OK response), the board falls back to "Show all authors" mode with no console error visible to the user

### Out of scope

- Backend changes: `author` always-present in API responses, `GET /api/tickets?author=`, `GET /api/me` endpoint — all covered by tickets #90ebf40b, #e2e3d958, and #70d58b2d
- Priority queue panel — no author filter applied there (queue is for the work engine, all actionable tickets regardless of author)
- Worker activity panel — no change
- Epic filter persistence-level author filter (the author filter uses local component state, same as the existing agent and state filters)
- `apm list --mine` and `apm list --author` CLI flags — separate ticket
- WebAuthn authentication UI — separate tickets
- Persisting the author filter selection across browser sessions (beyond the current page load)

### Approach

All changes are in `apm-ui/src/`. No backend changes.

**`apm-ui/src/components/supervisor/types.ts`**
- Add `author?: string` to the `Ticket` interface

**`apm-ui/src/components/supervisor/SupervisorView.tsx`**

1. Fetch `/api/me` once on mount; initialise `authorFilter` from the result:
   ```typescript
   const [authorFilter, setAuthorFilter] = useState<string | null>(null)
   useEffect(() => {
     fetch('/api/me')
       .then(r => r.ok ? r.json() : Promise.reject())
       .then((data: { username: string }) => {
         if (data.username && data.username !== 'unassigned') {
           setAuthorFilter(data.username)
         }
       })
       .catch(() => { /* leave authorFilter null — show all */ })
   }, [])
   ```

2. Build `availableAuthors` from loaded tickets (same pattern as existing `availableAgents`):
   ```typescript
   const availableAuthors = useMemo(() => {
     const authors = new Set<string>()
     for (const t of tickets) {
       if (t.author) authors.add(t.author)
     }
     return Array.from(authors).sort()
   }, [tickets])
   ```

3. Apply author filter in the `columns` useMemo after existing filters:
   ```typescript
   if (authorFilter !== null) {
     filtered = filtered.filter(t => t.author === authorFilter)
   }
   ```
   Add `authorFilter` to the dependency array of the useMemo.

4. Add author dropdown to the filter bar (after the agent dropdown, same DOM structure):
   - A `<select>` or equivalent with a "Show all authors" option (`value=""`) and one option per entry in `availableAuthors`
   - Selecting "Show all authors" calls `setAuthorFilter(null)`
   - Selecting an author calls `setAuthorFilter(value)`
   - Current `authorFilter` value drives the `value` prop of the control

5. Pass `showAuthor={authorFilter === null}` down to each `<TicketCard>` instance

**`apm-ui/src/components/supervisor/TicketCard.tsx`**
- Add `showAuthor?: boolean` prop
- When `showAuthor` is true and `ticket.author` is set (and not empty), render the author value below the title/ID in small, gray/muted text (e.g. `text-xs text-gray-400` or equivalent using the existing Tailwind classes in the file)
- When `showAuthor` is false (or `ticket.author` is absent), render nothing

**Order of work**
1. Add `author` to the `Ticket` type
2. Update `TicketCard` to accept and display `showAuthor`
3. Update `SupervisorView` — fetch, state, filter logic, dropdown, pass `showAuthor` to cards

**Constraints**
- No new dependencies; use the existing fetch pattern and Tailwind classes already in the codebase
- The `/api/me` call is fire-and-forget; it must not block rendering the board
- Do not rename or restructure existing filter state variables — only add alongside them

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-04-02T20:54Z | — | new | apm |
| 2026-04-02T23:23Z | new | groomed | apm |
| 2026-04-03T00:10Z | groomed | in_design | philippepascal |
| 2026-04-03T00:14Z | in_design | specd | claude-0402-2010-spec1 |
| 2026-04-04T00:29Z | specd | ready | apm |
| 2026-04-04T03:38Z | ready | in_progress | philippepascal |
| 2026-04-04T03:41Z | in_progress | implemented | claude-0403-0340-w1b5 |
| 2026-04-04T06:02Z | implemented | closed | apm-sync |
