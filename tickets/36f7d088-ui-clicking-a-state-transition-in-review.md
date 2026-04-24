+++
id = "36f7d088"
title = "UI: clicking a state transition in review editor doesn't close editor pane"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/36f7d088-ui-clicking-a-state-transition-in-review"
created_at = "2026-04-24T16:53:04.340859Z"
updated_at = "2026-04-24T16:55:32.621177Z"
+++

## Spec

### Problem

When a user clicks a state transition button in the ReviewEditor (e.g. to move a ticket from `in_review` to `specd`), the editor pane should close automatically — the same way it closes when the user clicks "Keep at [state] [K]". It does not.

The root cause is in `handleTransition` inside the `Editor` component in `apm-ui/src/components/ReviewEditor.tsx`. The function calls `handleSave()` unconditionally at the start, and returns early if the save fails — which prevents `setReviewMode(false)` from ever being called:

```
async function handleTransition(to: string) {
  const saved = await handleSave()
  if (!saved) return           // exits without closing the pane
  ...
  setReviewMode(false)         // only reached when save AND transition both succeed
}
```

`handleSave()` makes a `PUT /api/tickets/{id}/body` request. That endpoint reads the ticket's current content from the ticket's git branch via `read_from_branch`, then validates that the submitted frontmatter parses to the same `toml::Value` as the branch copy. Because `get_ticket` returns `raw = ticket.serialize()` — a re-serialised form of the parsed ticket — subtle TOML representation differences (string vs. datetime types, field ordering) can cause the comparison to fail even when the user has made no edits. The backend returns `422 "frontmatter is read-only"`, `handleSave()` returns `false`, and the pane stays open. The inline error span in the header shows the error message, but the user's primary expectation (pane closes after clicking a transition) is not met.

A secondary ordering issue compounds the problem even in the success path: `setReviewMode(false)` currently sits *after* two `queryClient.invalidateQueries` calls. Those invalidations can synchronously notify React Query subscribers (via `useSyncExternalStore` in React 18) before the Zustand store update commits, introducing a micro-render race.

### Acceptance criteria

- [ ] Clicking a valid state transition button closes the ReviewEditor pane when the transition API returns success
- [ ] If the editor has unsaved changes (`isDirty === true`), those changes are saved before the transition is attempted; the pane does not close if the save fails
- [ ] If the editor has no unsaved changes (`isDirty === false`), clicking a transition skips the save and calls the transition API directly
- [ ] If the transition API returns an error, the pane remains open and the error is displayed in the header
- [ ] Keyboard shortcuts for transitions (non-K keys) behave identically to clicking the transition buttons
- [ ] "Keep at [state] [K]" button behaviour is unchanged (no regression)

### Out of scope

- Fixing the TOML comparison logic in the `put_body` backend endpoint (the mismatch between `ticket.serialize()` and the raw branch file is a separate issue)
- Any changes to frontmatter or history-section validation in the backend
- Making save errors silent — they must still be displayed when they occur
- Changes to the keyboard shortcut registration or shortcut assignment logic

### Approach

Single-file change: `apm-ui/src/components/ReviewEditor.tsx`, the `handleTransition` function (currently lines 202-222). No backend changes, no other frontend files.

**Change 1 — guard `handleSave()` with `isDirtyRef.current`**

Only call `handleSave()` when the editor actually has unsaved changes. When there is nothing to save, skip directly to the transition API call. This eliminates the PUT request that was unconditionally blocking the transition even on a clean editor.

**Change 2 — move `setReviewMode(false)` before `invalidateQueries`**

Call `setReviewMode(false)` immediately after confirming the transition succeeded (`res.ok`), before either `queryClient.invalidateQueries` call. This ensures the pane unmounts synchronously before React Query notifies its subscribers, removing the micro-render race.

Replace the current `handleTransition` body with:

```tsx
async function handleTransition(to: string) {
  if (isDirtyRef.current) {
    const saved = await handleSave()
    if (!saved) return
  }
  try {
    const res = await fetch(`/api/tickets/${ticket.id}/transition`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ to }),
    })
    if (!res.ok) {
      const data = await res.json().catch(() => ({}))
      setError((data as { error?: string }).error ?? `Transition failed: ${res.status}`)
      return
    }
    setReviewMode(false)
    queryClient.invalidateQueries({ queryKey: ['ticket', ticket.id] })
    queryClient.invalidateQueries({ queryKey: ['tickets'] })
  } catch (e) {
    setError(String(e))
  }
}
```

No other logic changes. The keyboard-shortcut path (`handleTransitionRef.current(tr.to)` in the `useEffect`) calls the same function, so it inherits the fix automatically.

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-04-24T16:53Z | — | new | philippepascal |
| 2026-04-24T16:53Z | new | groomed | philippepascal |
| 2026-04-24T16:55Z | groomed | in_design | philippepascal |