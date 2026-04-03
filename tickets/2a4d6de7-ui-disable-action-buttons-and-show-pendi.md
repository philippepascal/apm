+++
id = "2a4d6de7"
title = "UI: disable action buttons and show pending state during mutations"
state = "ready"
priority = 0
effort = 0
risk = 0
author = "apm"
agent = "5072"
branch = "ticket/2a4d6de7-ui-disable-action-buttons-and-show-pendi"
created_at = "2026-04-02T23:24:43.919654Z"
updated_at = "2026-04-03T22:48:53.041030Z"
+++

## Spec

### Problem

After a button press — state transition, priority patch, batch transition, batch priority — the UI takes time to refresh while the server processes the request and the query cache is invalidated and refetched. During this window, all action buttons remain fully interactive. A user who doesn't see an immediate response can click again, triggering a duplicate transition or double-submit. A user who wants to do a second action immediately may accidentally re-fire the same one.

The fix is to propagate mutation pending state to every action surface so that buttons disable themselves for the duration of the in-flight request and give a visual signal (spinner or muted style) that work is happening.

**Strategy: disable + indicate during `isPending`**

React Query's `useMutation` already exposes `isPending` (true from the moment `mutate()` is called until the mutation settles). Pass this flag into every interactive component and use it to:
1. Set `disabled` on the button/input element — prevents re-clicks and keyboard activation
2. Show a `Loader2` spinner icon replacing or alongside the button label — gives immediate visual feedback
3. Reduce opacity or apply a muted style on the whole action area so the user knows the panel is busy

**Scope of surfaces to cover:**
- `TransitionButtons` in `TicketDetail` — each button should disable while its own mutation or any sibling transition mutation is pending (only one transition can be in flight at a time anyway)
- Inline patch fields (priority, effort, risk) in `TicketDetail` — disable commit while patch mutation is pending
- `BatchDetailPanel` transition buttons and priority field — disable all batch actions while either batch mutation is pending
- `WorkScreen` engine start/stop button (Shift+W) — already uses a mutation; the keyboard handler should check `isPending` before firing

**Why not optimistic updates:** State transitions require the server to compute the new `valid_transitions` set — the client cannot fake this reliably. Optimistic updates for transitions would show stale or incorrect transition buttons until the refetch completes. The disable+indicate pattern is simpler, always correct, and sufficient given the low latency of local server requests.

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
| 2026-04-02T23:24Z | — | new | apm |
| 2026-04-02T23:31Z | new | groomed | apm |
| 2026-04-03T00:27Z | groomed | in_design | philippepascal |
| 2026-04-03T22:48Z | in_design | ready | apm |
