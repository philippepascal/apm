+++
id = "1eeebe0b"
title = "UI search filters have white background, change to dark consistent with rest of UI"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "apm-ui"
branch = "ticket/1eeebe0b-ui-search-filters-have-white-background-"
created_at = "2026-04-04T15:50:12.140675Z"
updated_at = "2026-04-04T16:03:44.723739Z"
+++

## Spec

### Problem

The search filter toolbar in the Supervisor view contains five controls — a text search input and four select dropdowns (state, agent, author, epic) — styled with bg-white. The rest of the Supervisor UI uses a dark theme (bg-gray-900 background, bg-gray-700/bg-gray-800 for interactive elements). This makes the filter bar visually jarring: five bright-white boxes float against a dark surface, breaking the consistency of the interface.

The PriorityQueuePanel epic filter already uses bg-gray-800 text-gray-300 border-gray-700, which is the correct dark-theme pattern. The fix is to bring the SupervisorView filter controls in line with that established pattern.

Affected file: apm-ui/src/components/supervisor/SupervisorView.tsx, lines 201, 215, 225, 235, 245.

### Acceptance criteria

- [ ] The search text input in the Supervisor filter toolbar has a dark background (not white) consistent with the dark UI theme
- [ ] The state filter select has a dark background consistent with the dark UI theme
- [ ] The agent filter select has a dark background consistent with the dark UI theme
- [ ] The author filter select has a dark background consistent with the dark UI theme
- [ ] The epic filter select has a dark background consistent with the dark UI theme
- [ ] Text inside all five filter controls is legible against the dark background
- [ ] The filter controls retain their functional behaviour (filtering still works correctly)

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
| 2026-04-04T15:50Z | — | new | apm-ui |
| 2026-04-04T16:03Z | new | groomed | apm |
| 2026-04-04T16:03Z | groomed | in_design | philippepascal |