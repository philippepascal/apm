+++
id = "e36379d6"
title = "UI: add new epic button"
state = "closed"
priority = 0
effort = 2
risk = 1
author = "apm"
agent = "2364"
branch = "ticket/e36379d6-ui-add-new-epic-button"
created_at = "2026-04-02T20:47:05.242823Z"
updated_at = "2026-04-02T22:27:06.901966Z"
+++

## Spec

### Problem

The web UI provides no way to create epics. The only paths to epic creation are the CLI (`apm epic new`) and direct API calls. The SupervisorView toolbar already has a "New ticket" button that opens a modal, but there is no parallel affordance for epics, forcing supervisors to drop out of the UI whenever they need to define a new epic grouping.

### Acceptance criteria

- [x] The SupervisorView toolbar shows a "New epic" button next to the existing "New ticket" button
- [x] Clicking "New epic" opens a modal with a title input field
- [x] Submitting the modal with a non-empty title sends POST /api/epics and closes the modal on success
- [x] After successful creation the new epic appears in the SupervisorView epic-filter dropdown without a page refresh
- [x] Submitting the modal with an empty title shows a validation error and does not send a request
- [x] The modal can be dismissed by pressing Escape
- [x] The modal can be dismissed by clicking the backdrop

### Out of scope

- Editing or renaming an existing epic
- Closing or archiving an epic from the UI
- An epic detail view in the UI
- A keyboard shortcut to open the new-epic modal
- Adding tickets to an epic during epic creation

### Approach

**1. `apm-ui/src/store/useLayoutStore.ts`**
- Add `newEpicOpen: boolean` field (default `false`) to the store interface and initial state
- Add `setNewEpicOpen: (v: boolean) => void` action

**2. `apm-ui/src/components/NewEpicModal.tsx` (new file)**
Follow the same pattern as `NewTicketModal.tsx`:
- Props: `{ open: boolean; onOpenChange: (v: boolean) => void }`
- Single required `title` field with a `titleRef` for auto-focus
- On submit: validate non-empty title, then POST /api/epics with the title
- On success: `queryClient.invalidateQueries` on the `epics` query key, then close
- On error: show inline error message
- Escape key and backdrop click both dismiss the modal
- Reset state when `open` transitions to `false`

**3. `apm-ui/src/components/supervisor/SupervisorView.tsx`**
- Import `setNewEpicOpen` from `useLayoutStore`
- Add a "New epic" button in the toolbar div immediately after the existing "New ticket" button, matching its className

**4. `apm-ui/src/components/WorkScreen.tsx`**
- Import `NewEpicModal`
- Destructure `newEpicOpen` and `setNewEpicOpen` from `useLayoutStore()`
- Render `<NewEpicModal open={newEpicOpen} onOpenChange={setNewEpicOpen} />` alongside `<NewTicketModal>` in both render paths (resizable-panel branch and fallback branch)

No server-side changes needed — POST /api/epics already exists in `apm-server`.

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-04-02T20:47Z | — | new | apm |
| 2026-04-02T20:50Z | new | groomed | apm |
| 2026-04-02T20:56Z | groomed | in_design | philippepascal |
| 2026-04-02T21:13Z | in_design | specd | claude-0402-2100-spec1 |
| 2026-04-02T21:17Z | specd | ready | apm |
| 2026-04-02T21:17Z | ready | in_progress | philippepascal |
| 2026-04-02T21:19Z | in_progress | implemented | claude-0402-2130-impl1 |
| 2026-04-02T22:27Z | implemented | closed | apm-sync |