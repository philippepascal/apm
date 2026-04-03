+++
id = "2a4d6de7"
title = "UI: disable action buttons and show pending state during mutations"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "apm"
agent = "29072"
branch = "ticket/2a4d6de7-ui-disable-action-buttons-and-show-pendi"
created_at = "2026-04-02T23:24:43.919654Z"
updated_at = "2026-04-03T22:56:08.426220Z"
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

- [ ] While a transition mutation is in flight in `TransitionButtons`, all transition buttons and the "Keep" button are `disabled`
- [ ] While a transition mutation is in flight in `TransitionButtons`, the clicked transition button shows a spinner icon (`Loader2`) instead of its text label
- [ ] While the reassign mutation is in flight, the "Reassign to me" button is `disabled` and shows a spinner icon
- [ ] While `patchMutation` is pending in `TicketDetail`, the inline number fields (effort, risk, priority) do not allow new commits — `InlineNumberField` accepts a `disabled` prop that prevents activation
- [ ] While a batch transition is in flight in `BatchDetailPanel`, all batch transition buttons are `disabled` and the clicked button shows a spinner
- [ ] While the batch priority mutation is in flight in `BatchDetailPanel`, the priority `InlineNumberField` is disabled
- [ ] While either batch mutation (transition or priority) is pending, both batch transition buttons and the batch priority field are disabled
- [ ] The `Shift+W` keyboard handler in `WorkScreen` does not fire `startMutation.mutate()` or `stopMutation.mutate()` when the respective mutation is already pending
- [ ] After any mutation settles (success or error), the disabled state and spinner are removed and buttons return to their normal interactive state

### Out of scope

- Optimistic updates for state transitions (server must compute `valid_transitions`)
- Global loading bar or toast notifications
- Debouncing or throttling of rapid clicks (disabled state is sufficient)
- Retry logic for failed mutations
- Disabling non-action UI (e.g. markdown body, navigation) during mutations

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
| 2026-04-03T22:50Z | ready | ammend | apm |
| 2026-04-03T22:56Z | ammend | in_design | philippepascal |