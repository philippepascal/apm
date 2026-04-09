+++
id = "a88c5096"
title = "UI: assign buttons and window"
state = "in_design"
priority = 0
effort = 3
risk = 2
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/a88c5096-ui-assign-buttons-and-window"
created_at = "2026-04-09T05:16:55.238687Z"
updated_at = "2026-04-09T05:35:46.604856Z"
+++

## Spec

### Problem

The ticket detail panel has a "Reassign to me" button that only assigns a ticket to the current user. There is no way in the UI to assign a ticket to another collaborator. This is limiting in a team setting where a user may want to hand off a ticket or triage work to others.\n\nAdditionally, the current button calls POST /api/tickets/{id}/take, an endpoint that does not exist on the server, so the feature is currently broken end-to-end.\n\nThe desired behaviour: clicking an "Assign" button opens a small picker listing all project collaborators plus an "Unassigned" option. Selecting an entry assigns or clears the owner and dismisses the picker. The existing PATCH /api/tickets/:id endpoint already accepts an owner field, so the main work is a new collaborators API endpoint and a frontend picker component.

### Acceptance criteria

- [ ] The "Reassign to me" button is replaced by an "Assign" button in the TransitionButtons component\n- [ ] Clicking "Assign" opens a picker listing all project collaborators\n- [ ] The picker includes an "Unassigned" entry at the top to clear the owner\n- [ ] The current user is always present in the picker list\n- [ ] Selecting a collaborator calls PATCH /api/tickets/:id with { owner: username } and dismisses the picker\n- [ ] Selecting "Unassigned" calls PATCH /api/tickets/:id with { owner: "" } and dismisses the picker\n- [ ] After a successful assignment the ticket detail panel reflects the updated owner without a full page reload\n- [ ] While the assignment request is in-flight, the Assign button is disabled\n- [ ] If the assignment request fails, an error message is shown near the button\n- [ ] Pressing Escape or clicking outside the picker dismisses it without making any change\n- [ ] GET /api/collaborators returns the project collaborator list (from GitHub API or config.toml fallback)

### Out of scope

- Creating a POST /api/tickets/:id/take endpoint (the existing PATCH /api/tickets/:id owner field is sufficient)\n- Inline owner editing in the ticket header (InlineOwnerField already handles that; this ticket only changes the TransitionButtons area)\n- Batch assignment across multiple tickets\n- Role-based restrictions on who can assign to whom\n- Assignee validation on the server beyond what apm-core already enforces

### Approach

### Server — new GET /api/collaborators endpoint\n\nFile: apm-server/src/main.rs\n\n1. Add a handler function collaborators_handler that calls apm_core::config::resolve_collaborators() on the loaded AppState config and returns a JSON array of strings: ["alice", "bob", ...].\n2. Register the route inside build_app() under the protected router: GET /api/collaborators -> collaborators_handler.\n3. No new structs needed — the response is Vec<String> serialized directly.\n\n### Frontend — AssignPicker component\n\nFile: apm-ui/src/components/AssignPicker.tsx (new file)\n\nProps:\n  ticketId: string\n  onDone: () => void   // called after successful mutation or dismiss\n\nBehaviour:\n- On mount, fetch GET /api/collaborators and GET /api/me in parallel (both via useQuery).\n- Merge results into a deduplicated list; prepend the "Unassigned" sentinel ("").\n- Render a small absolutely-positioned box (role="listbox") listing each name as a clickable row.\n- On row click: fire useMutation calling PATCH /api/tickets/:id with { owner: selectedName }, then call onDone().\n- On Escape keydown or click-outside (useEffect with mousedown listener), call onDone() without mutating.\n- Show a Loader2 spinner while the mutation is pending; disable all rows.\n- Show an inline error message on mutation failure; keep the picker open so the user can retry or dismiss.\n\n### Frontend — TransitionButtons changes\n\nFile: apm-ui/src/components/TicketDetail.tsx\n\n1. Remove the reassignMutation (useMutation calling /take) and the reassignError state.\n2. Add showAssignPicker: boolean state, default false.\n3. Replace the "Reassign to me" button with:\n   <div className="relative">\n     <button onClick={() => setShowAssignPicker(true)} disabled={anyPending}>Assign</button>\n     {showAssignPicker && (\n       <AssignPicker\n         ticketId={ticket.id}\n         onDone={() => {\n           setShowAssignPicker(false)\n           queryClient.invalidateQueries({ queryKey: ['ticket', ticket.id] })\n           queryClient.invalidateQueries({ queryKey: ['tickets'] })\n         }}\n       />\n     )}\n   </div>\n4. The anyPending check no longer needs to include a reassignMutation state; remove it from the anyPending expression.\n5. Remove reassignError display.\n\n### Styling\n- Picker box: bg-gray-900 border border-gray-600 rounded shadow-lg p-1, min-w-[12rem], z-50.\n- Each row: px-3 py-1 text-sm rounded hover:bg-gray-700 cursor-pointer text-gray-200.\n- "Unassigned" row: italic text-gray-400.

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-04-09T05:16Z | — | new | philippepascal |
| 2026-04-09T05:17Z | new | groomed | apm |
| 2026-04-09T05:29Z | groomed | in_design | philippepascal |