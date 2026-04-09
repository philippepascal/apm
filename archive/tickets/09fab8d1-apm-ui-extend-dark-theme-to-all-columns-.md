+++
id = "09fab8d1"
title = "apm-ui: extend dark theme to all columns and fix worker card regressions"
state = "closed"
priority = 0
effort = 3
risk = 2
author = "philippepascal"
agent = "17095"
branch = "ticket/09fab8d1-apm-ui-extend-dark-theme-to-all-columns-"
created_at = "2026-04-01T06:44:14.497120Z"
updated_at = "2026-04-01T07:47:21.854092Z"
+++

## Spec

### Problem

Commit 4d884371 applied a dark background (bg-gray-900) only to the left WorkerView column. The center column (SupervisorView) still uses bg-gray-50 and the right column (TicketDetail) uses bg-white, making the three-column layout visually inconsistent. The dark theme must be applied uniformly so all columns share the same dark palette.

The same commit replaced WorkerActivityPanel's table with a card layout but introduced two regressions:
1. Click-to-select removed. The old table rows called setSelectedTicketId on click; the new card divs have no onClick handler, so clicking a worker card no longer opens that ticket in the detail panel.
2. Status label removed. The old table had an explicit text badge ('running' / 'crashed'). The new cards show only a green or red dot with no label, making status harder to read at a glance.

These regressions affect every user of the UI who relies on clicking a worker to jump to its ticket, and on reading worker status without hovering.

### Acceptance criteria

- [x] SupervisorView background is dark (bg-gray-900 or equivalent) and header text is light
- [x] Swimlane lane-count badge uses dark-palette colors instead of bg-gray-100/text-gray-600
- [x] TicketCard background is dark (bg-gray-800 or equivalent) and title text is light
- [x] TicketDetail panel background is dark (bg-gray-900 or equivalent) and body text is light
- [x] TicketDetail header sub-bar uses a dark surface (bg-gray-800 or equivalent) instead of bg-gray-50
- [x] TicketDetail transition buttons use dark surface and border colors
- [x] WorkScreen top toolbar (column-toggle bar) uses a dark background instead of bg-gray-50
- [x] Clicking a WorkerActivityPanel card calls setSelectedTicketId with that card's ticket_id
- [x] Each WorkerActivityPanel card displays the status text ('running' or 'crashed') alongside the colored dot

### Out of scope

- System-level dark mode (prefers-color-scheme media queries / CSS variables / theme toggle)
- Theming infrastructure — no new theme abstraction layer; direct Tailwind class changes only
- stateColors.ts badge palette — badge pills (bg-X-100 text-X-700) carry their own background and remain readable on dark surfaces; recolouring them is a separate concern
- PriorityQueuePanel, ReviewEditor, NewTicketModal, WorkEngineControls — not mentioned in the problem statement
- WorkerView.tsx — already dark; no changes needed

### Approach

All changes are Tailwind class substitutions — no logic changes except the two WorkerActivityPanel fixes. Touch six files total.

**1. apm-ui/src/components/WorkScreen.tsx (line 179)**
- Toolbar bar: bg-gray-50 → bg-gray-900; button hover: hover:bg-gray-200 → hover:bg-gray-700; icon active color: text-gray-600 → text-gray-400

**2. apm-ui/src/components/supervisor/SupervisorView.tsx**
- Root div (line 55): bg-gray-50 → bg-gray-900 text-gray-100
- Header border (line 56): add border-gray-700
- 'New ticket' and 'Sync' buttons (lines 65–82): border → border-gray-600; bg implicit → bg-gray-800; hover:bg-gray-100 → hover:bg-gray-700; text inherits from root
- syncError span: text-red-500 stays

**3. apm-ui/src/components/supervisor/Swimlane.tsx**
- Lane-count badge (line 16): bg-gray-100 text-gray-600 → bg-gray-700 text-gray-300

**4. apm-ui/src/components/supervisor/TicketCard.tsx**
- Root div (line 17): bg-white → bg-gray-800; hover:bg-gray-50 → hover:bg-gray-700; border → border-gray-600
- Effort badge (line 24): bg-gray-100 text-gray-500 → bg-gray-700 text-gray-300
- Risk normal badge (line 34): bg-gray-100 text-gray-500 → bg-gray-700 text-gray-300
- Risk high badge (line 33): bg-red-100 text-red-700 → bg-red-900/60 text-red-300

**5. apm-ui/src/components/TicketDetail.tsx**
- Root div (line 144): bg-white → bg-gray-900 text-gray-100
- Header sub-bar (line 145): bg-gray-50 → bg-gray-800; add border-gray-700 to border-b
- Title h2 (line 149): text-gray-900 → text-gray-100
- Subtitle span (line 153): text-gray-700 → text-gray-300
- Review button (line 158): bg-white hover:bg-gray-100 → bg-gray-700 hover:bg-gray-600 border-gray-600
- ID monospace (line 170): text-gray-400 stays
- Loading skeletons (lines 207–211): bg-gray-200 → bg-gray-700
- Error panel (line 214): border-red-200 bg-red-50 text-red-700 → border-red-700 bg-red-900/30 text-red-400
- Prose wrapper (line 219): add prose-invert so ReactMarkdown renders light text on dark bg
- TransitionButtons border-t (line 68): add border-gray-700
- Transition buttons (line 72, 80): bg-white hover:bg-gray-50 → bg-gray-800 hover:bg-gray-700 border-gray-600 text-gray-200
- Keep button text-gray-500 (line 82): → text-gray-400
- patchError (line 87): text-red-600 stays (visible on dark)

**6. apm-ui/src/components/WorkerActivityPanel.tsx — two regressions**

Regression 1 — click-to-select:
- Import useLayoutStore
- Destructure selectedTicketId and setSelectedTicketId from the store
- Add onClick={() => setSelectedTicketId(w.ticket_id)} to the card div (line 55)
- Add cursor-pointer to the card div classes
- Optionally add a ring when w.ticket_id === selectedTicketId: ring-2 ring-blue-500

Regression 2 — status label:
- After the dot span, add: <span className="text-[10px] text-gray-400">{w.status}</span>
- Place it in the second row (gap-2 mt-0.5 div) so the top row stays clean, or inline after the dot — either is acceptable

### Open questions



### Amendment requests



## History

| When | From | To | By |
|------|------|----|----|
| 2026-04-01T06:44Z | — | new | philippepascal |
| 2026-04-01T06:44Z | new | in_design | philippepascal |
| 2026-04-01T06:47Z | in_design | specd | claude-0401-0644-c800 |
| 2026-04-01T07:08Z | specd | ready | apm |
| 2026-04-01T07:26Z | ready | in_progress | philippepascal |
| 2026-04-01T07:31Z | in_progress | implemented | claude-0401-0726-9c50 |
| 2026-04-01T07:46Z | implemented | accepted | apm-sync |
| 2026-04-01T07:47Z | accepted | closed | apm-sync |