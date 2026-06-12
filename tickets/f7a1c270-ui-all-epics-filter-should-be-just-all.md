+++
id = "f7a1c270"
title = "UI: All Epics filter should be just All"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/f7a1c270-ui-all-epics-filter-should-be-just-all"
created_at = "2026-06-10T02:50:51.566252Z"
updated_at = "2026-06-12T08:13:10.880459Z"
+++

## Spec

### Problem

The epic filter dropdown in the board's toolbar shows "All epics" as its default (unfiltered) option. This label is inconsistent with the intent: selecting it shows every ticket regardless of epic, which corresponds to `apm list --all` semantics — all tickets including those in terminal (closed) states. However, the current implementation only shows non-closed tickets when the option is selected, requiring a separate "Show closed" checkbox to surface closed tickets. The board therefore diverges from `apm list --all` when the epic filter is in its default state.

The fix has two parts: rename the option label to "All" (dropping the redundant "epics" qualifier), and ensure that selecting "All" includes closed tickets automatically, matching the full set `apm list --all` returns.

### Acceptance criteria

- [ ] The epic filter dropdown default option reads "All" (not "All epics")
- [ ] When "All" is selected, the board fetches and displays closed/terminal tickets without requiring the "Show closed" checkbox to be checked
- [ ] When a specific epic is selected, closed tickets are hidden unless "Show closed" is also checked
- [ ] When "No epic" is selected, closed tickets are hidden unless "Show closed" is also checked
- [ ] The "Show closed" checkbox remains visible and continues to function as a way to include closed tickets when a specific epic or "No epic" is active

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
| 2026-06-10T02:50Z | — | new | philippepascal |
| 2026-06-12T07:52Z | new | groomed | philippepascal |
| 2026-06-12T08:13Z | groomed | in_design | philippepascal |