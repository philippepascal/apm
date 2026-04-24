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

Checkboxes; each one independently testable.

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
| 2026-04-24T16:53Z | — | new | philippepascal |
| 2026-04-24T16:53Z | new | groomed | philippepascal |
| 2026-04-24T16:55Z | groomed | in_design | philippepascal |