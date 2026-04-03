+++
id = "b50dcc1c"
title = "UI: author filter on supervisor board, default to current user from /api/me"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "apm"
agent = "43395"
branch = "ticket/b50dcc1c-ui-author-filter-on-supervisor-board-def"
created_at = "2026-04-02T20:54:34.590380Z"
updated_at = "2026-04-03T00:10:33.348507Z"
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

- [ ] On load, the board fetches `GET /api/me` and uses the returned `username` as the initial author filter value
- [ ] When `/api/me` returns `"unassigned"`, the author filter is left unset ("Show all authors" mode), not filtered to "unassigned"
- [ ] An "Author" dropdown appears in the filter bar showing all unique author values from the currently loaded ticket set
- [ ] The author dropdown has a "Show all authors" option that clears the filter
- [ ] When an author is selected in the dropdown, only tickets with a matching `author` value are shown on the board
- [ ] The author filter composes with the existing state, agent, epic, and search filters using AND logic
- [ ] When the author filter is active (single author selected), ticket cards do not display the author label
- [ ] When "Show all authors" is active, ticket cards display the author value in small subdued text
- [ ] The `Ticket` TypeScript interface includes an `author` field (string)
- [ ] If `/api/me` fails (network error or non-OK response), the board falls back to "Show all authors" mode with no console error visible to the user

### Out of scope

- Backend changes: `author` always-present in API responses, `GET /api/tickets?author=`, `GET /api/me` endpoint — all covered by tickets #90ebf40b, #e2e3d958, and #70d58b2d
- Priority queue panel — no author filter applied there (queue is for the work engine, all actionable tickets regardless of author)
- Worker activity panel — no change
- Epic filter persistence-level author filter (the author filter uses local component state, same as the existing agent and state filters)
- `apm list --mine` and `apm list --author` CLI flags — separate ticket
- WebAuthn authentication UI — separate tickets
- Persisting the author filter selection across browser sessions (beyond the current page load)

### Approach

How the implementation will work.

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-04-02T20:54Z | — | new | apm |
| 2026-04-02T23:23Z | new | groomed | apm |
| 2026-04-03T00:10Z | groomed | in_design | philippepascal |