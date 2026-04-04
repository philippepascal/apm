+++
id = "1eeebe0b"
title = "UI search filters have white background, change to dark consistent with rest of UI"
state = "in_progress"
priority = 0
effort = 1
risk = 1
author = "apm-ui"
branch = "ticket/1eeebe0b-ui-search-filters-have-white-background-"
created_at = "2026-04-04T15:50:12.140675Z"
updated_at = "2026-04-04T17:33:28.517399Z"
+++

## Spec

### Problem

The search filter toolbar in the Supervisor view contains five controls — a text search input and four select dropdowns (state, agent, author, epic) — styled with bg-white. The rest of the Supervisor UI uses a dark theme (bg-gray-900 background, bg-gray-700/bg-gray-800 for interactive elements). This makes the filter bar visually jarring: five bright-white boxes float against a dark surface, breaking the consistency of the interface.

The PriorityQueuePanel epic filter already uses bg-gray-800 text-gray-300 border-gray-700, which is the correct dark-theme pattern. The fix is to bring the SupervisorView filter controls in line with that established pattern.

Affected file: apm-ui/src/components/supervisor/SupervisorView.tsx, lines 201, 215, 225, 235, 245.

### Acceptance criteria

- [x] The search text input in the Supervisor filter toolbar has a dark background (not white) consistent with the dark UI theme
- [x] The state filter select has a dark background consistent with the dark UI theme
- [x] The agent filter select has a dark background consistent with the dark UI theme
- [x] The author filter select has a dark background consistent with the dark UI theme
- [x] The epic filter select has a dark background consistent with the dark UI theme
- [x] Text inside all five filter controls is legible against the dark background
- [x] The filter controls retain their functional behaviour (filtering still works correctly)

### Out of scope

- Restyling any other UI component not in the SupervisorView filter toolbar
- Dark-mode toggle or theming system
- Changes to PriorityQueuePanel (its filter is already dark-themed)
- Any filter logic or behaviour changes

### Approach

Single file change: apm-ui/src/components/supervisor/SupervisorView.tsx

For each of the five filter controls, replace bg-white with bg-gray-800 text-gray-100 border-gray-600. The focus ring (focus:ring-blue-400) can stay.

Specific className changes:

1. Search text input (line 201):
   From: h-7 pl-2 pr-6 text-xs border rounded bg-white focus:outline-none focus:ring-1 focus:ring-blue-400 w-40
   To:   h-7 pl-2 pr-6 text-xs border border-gray-600 rounded bg-gray-800 text-gray-100 placeholder-gray-500 focus:outline-none focus:ring-1 focus:ring-blue-400 w-40

   Also update the clear-button hover from hover:text-gray-600 to hover:text-gray-200 (line 206) so it is visible on the dark input.

2. State filter select (line 215):
   From: h-7 px-1.5 text-xs border rounded bg-white focus:outline-none focus:ring-1 focus:ring-blue-400
   To:   h-7 px-1.5 text-xs border border-gray-600 rounded bg-gray-800 text-gray-100 focus:outline-none focus:ring-1 focus:ring-blue-400

3–5. Agent, author, epic filter selects (lines 225, 235, 245): identical change as state filter.

No logic changes. No new files. No dependency changes.

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-04-04T15:50Z | — | new | apm-ui |
| 2026-04-04T16:03Z | new | groomed | apm |
| 2026-04-04T16:03Z | groomed | in_design | philippepascal |
| 2026-04-04T16:05Z | in_design | specd | claude-0404-1603-9198 |
| 2026-04-04T17:26Z | specd | ready | apm |
| 2026-04-04T17:33Z | ready | in_progress | philippepascal |
