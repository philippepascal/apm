+++
id = "30a468a4"
title = "in review panel, clicking change of state doesn't close review panel"
state = "in_progress"
priority = 0
effort = 2
risk = 3
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/30a468a4-in-review-panel-clicking-change-of-state"
created_at = "2026-04-27T22:04:03.732362Z"
updated_at = "2026-04-28T01:11:51.770425Z"
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

**Root cause and fix**

The problem is in `apm-ui/src/components/ReviewEditor.tsx`, `handleTransition` (lines 202–224). After a successful transition the code does:

```
setReviewMode(false)                                                   // line 218
queryClient.invalidateQueries({ queryKey: ['ticket', ticket.id] })     // line 219
queryClient.invalidateQueries({ queryKey: ['tickets'] })               // line 220
```

`setReviewMode(false)` schedules a Zustand state update. Before React commits it, `queryClient.invalidateQueries` (line 219) marks the `['ticket', ticket.id]` query stale and — because `ReviewEditor` still has an active `useQuery` observer — triggers an immediate background refetch. React may process the React Query internal state change (stale + isFetching) in the same batch, causing `ReviewEditor` to re-render. If that re-render happens before `WorkScreen` has committed `reviewMode = false`, the component stays alive and the panel never closes.

**Fix — wrap `setReviewMode(false)` in `flushSync`**

`flushSync` (from `react-dom`) forces React to synchronously flush the enclosed state update before returning. This guarantees `WorkScreen` has already switched to the normal layout (and unmounted `ReviewEditor`) before the `invalidateQueries` calls trigger any further re-renders.

Changes in `apm-ui/src/components/ReviewEditor.tsx`:

1. Add `import { flushSync } from 'react-dom'` to the imports at the top of the file.

2. In `handleTransition`, change the success block from:
   ```
   setReviewMode(false)
   queryClient.invalidateQueries(...)
   queryClient.invalidateQueries(...)
   ```
   to:
   ```
   flushSync(() => setReviewMode(false))
   queryClient.invalidateQueries(...)
   queryClient.invalidateQueries(...)
   ```

No other files need to change. `handleCancel` (line 197) and the keyboard handler (line 243) call `setReviewMode(false)` from synchronous event handlers not followed by query invalidations; they do not exhibit the race and need no change.

**Verification before applying the fix**

Add `console.log('setReviewMode(false) reached')` immediately before line 218 and reproduce the bug in the browser. If the log does NOT appear, the API is returning a non-ok status that the catch/error path silently swallows — surface the error instead of applying flushSync. If the log DOES appear, the race-condition explanation holds; apply flushSync.

**Fallback (if flushSync is insufficient)**

Add a `useEffect` in `Editor` that stores the initial ticket state in a ref when the panel opens, then calls `setReviewMode(false)` whenever `ticket.state` changes to a different value. This is more invasive but removes any dependency on the imperative call-order.

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-04-27T22:04Z | — | new | philippepascal |
| 2026-04-27T22:04Z | new | groomed | philippepascal |
| 2026-04-27T22:07Z | groomed | in_design | philippepascal |
| 2026-04-27T22:17Z | in_design | specd | claude-0427-2207-7d28 |
| 2026-04-28T00:50Z | specd | ready | philippepascal |
| 2026-04-28T01:11Z | ready | in_progress | philippepascal |
