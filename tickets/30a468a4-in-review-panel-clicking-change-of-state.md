+++
id = "30a468a4"
title = "in review panel, clicking change of state doesn't close review panel"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/30a468a4-in-review-panel-clicking-change-of-state"
created_at = "2026-04-27T22:04:03.732362Z"
updated_at = "2026-04-27T22:07:41.523971Z"
+++

## Spec

### Problem

When a user opens the review panel (via the "Review" button in `TicketDetail`) and clicks a state-change button (e.g., "→ specd", "→ accepted"), the panel should close and return the user to the normal 3-column board view. Instead, it remains open after the transition.

The review panel is controlled by the `reviewMode` boolean in `useLayoutStore` (`apm-ui/src/store/useLayoutStore.ts`). `WorkScreen` renders a 2-column layout (WorkerView + ReviewEditor) when `reviewMode` is `true`, and the normal 3-column layout (Workers, Board, TicketDetail) when `false`.

Inside `ReviewEditor.tsx`, the `handleTransition` function (lines 202–224) handles state-change button clicks. On a successful API response it calls `setReviewMode(false)` at line 218, which *should* close the panel. Despite this call being present in the code, the panel stays open.

The likely cause is a React render-batching race: `setReviewMode(false)` (a Zustand update) is immediately followed by `queryClient.invalidateQueries` calls (lines 219–220) that—because `ReviewEditor` is still mounted at call time—trigger an immediate background refetch. If React processes the React Query state change before the Zustand update, `ReviewEditor` re-renders once more with `reviewMode` still `true`, and the component survives the render cycle that was supposed to unmount it.

### Acceptance criteria

- [ ] After clicking a state-change button in the review panel and the transition API returns success, the review panel closes
- [ ] After the panel closes, the normal 3-column board view is shown
- [ ] After the panel closes, the board column for the ticket reflects its new state without a page reload
- [ ] If the transition API returns an error, the panel stays open and displays the error message
- [ ] The "Keep at [state] [K]" button closes the panel without changing the ticket's state
- [ ] The K keyboard shortcut closes the panel without changing the ticket's state
- [ ] Keyboard-shortcut transitions (letter keys mapped to valid states) also close the panel on success

### Out of scope

- Changing the transition API endpoint or its success/error semantics\n- The save-before-transition flow (handleSave behavior is not part of this bug)\n- TransitionButtons in TicketDetail.tsx — those live in the normal (non-review) layout and do not need to close a review panel

### Approach

How the implementation will work.

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-04-27T22:04Z | — | new | philippepascal |
| 2026-04-27T22:04Z | new | groomed | philippepascal |
| 2026-04-27T22:07Z | groomed | in_design | philippepascal |