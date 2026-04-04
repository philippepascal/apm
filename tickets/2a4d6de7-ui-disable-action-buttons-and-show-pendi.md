+++
id = "2a4d6de7"
title = "UI: disable action buttons and show pending state during mutations"
state = "closed"
priority = 0
effort = 3
risk = 2
author = "apm"
branch = "ticket/2a4d6de7-ui-disable-action-buttons-and-show-pendi"
created_at = "2026-04-02T23:24:43.919654Z"
updated_at = "2026-04-04T17:00:15.786073Z"
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

- [x] While a transition mutation is in flight in `TransitionButtons`, all transition buttons and the "Keep" button are `disabled`
- [x] While a transition mutation is in flight in `TransitionButtons`, the clicked transition button shows a spinner icon (`Loader2`) instead of its text label
- [x] While the reassign mutation is in flight, the "Reassign to me" button is `disabled` and shows a spinner icon
- [x] While `patchMutation` is pending in `TicketDetail`, the inline number fields (effort, risk, priority) do not allow new commits — `InlineNumberField` accepts a `disabled` prop that prevents activation
- [x] While a batch transition is in flight in `BatchDetailPanel`, all batch transition buttons are `disabled` and the clicked button shows a spinner
- [x] While the batch priority mutation is in flight in `BatchDetailPanel`, the priority `InlineNumberField` is disabled
- [x] While either batch mutation (transition or priority) is pending, both batch transition buttons and the batch priority field are disabled
- [x] The `Shift+W` keyboard handler in `WorkScreen` does not fire `startMutation.mutate()` or `stopMutation.mutate()` when the respective mutation is already pending
- [x] After any mutation settles (success or error), the disabled state and spinner are removed and buttons return to their normal interactive state

### Out of scope

- Optimistic updates for state transitions (server must compute `valid_transitions`)
- Global loading bar or toast notifications
- Debouncing or throttling of rapid clicks (disabled state is sufficient)
- Retry logic for failed mutations
- Disabling non-action UI (e.g. markdown body, navigation) during mutations

### Approach

Three files change: `TicketDetail.tsx`, `InlineNumberField.tsx`, and `WorkScreen.tsx`.

#### 1. `InlineNumberField.tsx` — add `disabled` prop

- Add optional `disabled?: boolean` to `InlineNumberFieldProps`
- When `disabled` is true, the display-mode `<span>` ignores click/keydown (don't call `activate()`) and applies `opacity-50 cursor-not-allowed` instead of `cursor-pointer hover:bg-gray-100`
- When `disabled` is true and already in editing mode, the input and commit are also disabled (edge case: user clicks field, then a sibling mutation starts)

#### 2. `TicketDetail.tsx` — `TransitionButtons`

The current `TransitionButtons` uses manual `useState` for `pending` / `reassigning`. Refactor both to `useMutation`:

- Create `transitionMutation = useMutation({ mutationFn: (to: string) => fetch(...) })` replacing the manual `doTransition` function
- Create `reassignMutation = useMutation({ mutationFn: () => fetch(...) })` replacing the manual `handleReassign` function
- Derive `anyPending = transitionMutation.isPending || reassignMutation.isPending`
- All transition buttons and "Keep" button: `disabled={anyPending}`
- "Reassign to me" button: `disabled={anyPending}`
- Track which transition target is in flight (e.g. `transitionMutation.variables`) — the button whose `tr.to` matches the in-flight variable shows `<Loader2 className="w-3 h-3 animate-spin" />` instead of `{tr.label}`
- Reassign button: when `reassignMutation.isPending`, show `<Loader2>` spinner

#### 3. `TicketDetail.tsx` — inline fields

- Pass `disabled={patchMutation.isPending}` to each `<InlineNumberField>` for effort, risk, and priority

#### 4. `TicketDetail.tsx` — `BatchDetailPanel`

Refactor `doBatchTransition` and `doBatchPriority` to `useMutation`:

- `batchTransitionMutation = useMutation({ mutationFn: (to: string) => fetch(...) })`
- `batchPriorityMutation = useMutation({ mutationFn: (priority: number) => fetch(...) })`
- Derive `batchPending = batchTransitionMutation.isPending || batchPriorityMutation.isPending`
- All batch transition buttons: `disabled={batchPending}`, clicked button shows spinner via `batchTransitionMutation.variables`
- Batch priority field: `disabled={batchPending}`

#### 5. `WorkScreen.tsx` — Shift+W guard

- In the `Shift+W` handler, check `startMutation.isPending || stopMutation.isPending` before calling `fetchStatus()`. If either is pending, return early.

#### Import

- Add `Loader2` to the lucide-react import in `TicketDetail.tsx` (already declared in `lucide-react.d.ts`)

#### Spinner style

- Use `<Loader2 className="w-3 h-3 animate-spin" />` consistently for all button spinners
- Disabled buttons already have `disabled:opacity-50` in the existing Tailwind classes

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
| 2026-04-03T22:58Z | in_design | specd | claude-0403-2257-spec |
| 2026-04-04T00:30Z | specd | ready | apm |
| 2026-04-04T02:22Z | ready | specd | apm |
| 2026-04-04T06:01Z | specd | ready | apm |
| 2026-04-04T07:09Z | ready | in_progress | philippepascal |
| 2026-04-04T07:13Z | in_progress | implemented | claude-0404-0710-w2a4 |
| 2026-04-04T17:00Z | implemented | closed | apm-sync |
