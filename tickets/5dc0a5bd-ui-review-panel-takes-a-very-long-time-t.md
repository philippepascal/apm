+++
id = "5dc0a5bd"
title = "UI review panel takes a very long time to close"
state = "implemented"
priority = 6
effort = 2
risk = 2
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/5dc0a5bd-ui-review-panel-takes-a-very-long-time-t"
created_at = "2026-06-01T18:17:29.250149Z"
updated_at = "2026-06-01T18:30:20.847148Z"
+++

## Spec

### Problem

When a reviewer clicks a transition button in the review panel (e.g., "→ specd", "→ ammend"), the panel blocks on the HTTP response from `POST /api/tickets/{id}/transition` before calling `setReviewMode(false)`. That endpoint runs git operations inside a `spawn_blocking` task — at minimum a `commit_to_branch` call (which may create and tear down a temporary git worktree), and for completion strategies such as `Merge` also a `git push`, `git merge`, and another `git push`. End-to-end this can take 10–30 seconds or more. The panel stays open and unresponsive for the entire duration, with no progress indication. The `flushSync(() => setReviewMode(false))` call at line 194 of `ReviewEditor.tsx` is the specific statement gating the close on request completion.

The operation triggered by the button click does not need to complete before the panel can safely close. The transition is a fire-and-forget action from the reviewer's perspective — the result will be visible in the ticket list once the background git work finishes. Keeping the panel open during this wait gives the user no useful control and creates a perception of the app being frozen.

### Acceptance criteria

- [x] Clicking a transition button closes the review panel within ~200ms (after any pending save completes)
- [x] The transition request is still sent to the server; closing the panel does not cancel the in-flight request
- [x] The ticket list and ticket detail refresh automatically once the transition request completes
- [x] If the transition request fails (non-2xx response or network error), an error message is visible to the user after the panel has closed
- [x] The error message is dismissible by the user
- [x] If there are unsaved edits, the save is still awaited before the panel closes
- [x] Pressing K / "Keep at…" still closes the panel immediately without a network request (unchanged behaviour)

### Out of scope

- Speeding up server-side git operations (commit, push, merge, PR creation)
- Streaming or real-time progress reporting for in-flight transitions
- Making the save operation non-blocking (save still awaits before close)
- Changes to the server handler or `apm-core`

### Approach

Three files change, all in `apm-ui/src/`. No server changes.

#### useLayoutStore.ts

Add `transitionError: string | null` (initialised to `null`) and `setTransitionError: (msg: string | null) => void` (sets `{ transitionError: msg }`) to the store interface and `create` call.

#### ReviewEditor.tsx — handleTransition

Replace the current flow:

```
(if dirty) await handleSave() → await fetch(transition) → setReviewMode(false)
```

with:

```
(if dirty) await handleSave()  ← still blocking; save must succeed before close
setReviewMode(false)            ← close immediately
fetch(transition)               ← fire without await
  .then: invalidate ['ticket', id] and ['tickets']
  .catch / non-ok: setTransitionError(msg), then invalidate both query keys
```

Remove the `flushSync` wrapper — it is only needed when the close follows async work in the same microtask, which is no longer the case. Remove the `setError` call in the transition error path; that state is local to the (now closed) Editor component. Import `setTransitionError` from `useLayoutStore`.

Also clear any stale error at the top of `handleTransition`: call `setTransitionError(null)` before doing anything.

#### WorkScreen.tsx

Read `transitionError` and `setTransitionError` from `useLayoutStore`. In both render branches (the `if (reviewMode)` early-return branch and the normal branch), render a dismissible banner when `transitionError` is set:

```tsx
{transitionError && (
  <div className="fixed top-2 right-2 z-50 max-w-sm bg-red-50 border border-red-300 text-red-700 text-sm px-3 py-2 rounded shadow flex items-center gap-2">
    <span className="flex-1">Transition failed: {transitionError}</span>
    <button onClick={() => setTransitionError(null)} className="shrink-0 hover:text-red-900">✕</button>
  </div>
)}
```

Place it as the first child inside each top-level `<div>` so it appears above the panel layout but respects `fixed` positioning.

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-06-01T18:17Z | — | new | philippepascal |
| 2026-06-01T18:17Z | new | groomed | philippepascal |
| 2026-06-01T18:17Z | groomed | in_design | philippepascal |
| 2026-06-01T18:22Z | in_design | specd | claude |
| 2026-06-01T18:25Z | specd | ready | philippepascal |
| 2026-06-01T18:25Z | ready | in_progress | philippepascal |
| 2026-06-01T18:30Z | in_progress | implemented | claude |
