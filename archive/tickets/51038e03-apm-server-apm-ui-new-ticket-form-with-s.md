+++
id = "51038e03"
title = "apm-server + apm-ui: new ticket form with section pre-population"
state = "closed"
priority = 35
effort = 4
risk = 2
author = "apm"
agent = "82839"
branch = "ticket/51038e03-apm-server-apm-ui-new-ticket-form-with-s"
created_at = "2026-03-31T06:12:50.437393Z"
updated_at = "2026-04-01T06:20:57.465991Z"
+++

## Spec

### Problem

There is no way to create a ticket from the UI. Steps 1–9 deliver the backend server, ticket list/detail API, and the markdown editor; Step 10 adds ticket creation.

A '+ New ticket' button (and keyboard shortcut) must open a modal form with a required title field and optional fields for each standard spec section (Problem, Acceptance criteria, Out of scope, Approach). Submitting the form calls a new POST /api/tickets endpoint in apm-server, which delegates atomically to ticket::create in apm-core — the same function the CLI uses. All provided sections are written to the ticket body at creation time (parity with the --section/--set CLI feature).

Without this, supervisors using the web UI have no way to capture new work items without switching to the command line.

### Acceptance criteria

- [x] A '+ New ticket' button is visible in the supervisorview column header area
- [x] Pressing the '+ New ticket' button opens a modal dialog
- [x] Pressing the 'n' key (when no text input is focused) opens the new ticket modal
- [x] The modal contains a Title field that is required
- [x] The modal contains optional textarea fields for Problem, Acceptance criteria, Out of scope, and Approach
- [x] Attempting to submit the form with an empty title shows a validation error and does not call the API
- [x] Submitting a valid form calls POST /api/tickets with the title and any non-empty section content
- [x] POST /api/tickets returns 201 with the created ticket as JSON on success
- [x] POST /api/tickets returns 400 when title is absent or empty
- [x] After successful creation, the new ticket appears in the supervisor swimlanes (TanStack Query cache is invalidated)
- [x] Pressing Escape or clicking Cancel closes the modal without creating a ticket
- [x] The form shows a loading indicator while the mutation is in flight
- [x] The form shows an inline error message if the API call fails

### Out of scope

- Setting effort, risk, or priority at creation time (covered by Step 13b inline editing)
- Choosing a custom author or supervisor at creation time (defaults to server-configured author)
- Attaching files or images
- Auto-saving draft form state across page reloads
- Editing or deleting tickets from the modal (edit is Step 9; delete is not planned)
- Any section beyond the four standard spec sections (Problem, Acceptance criteria, Out of scope, Approach)

### Approach

**Prerequisite:** Step 9 (ticket a6c115e1) must be implemented — apm-server and apm-ui are both present, PUT /api/tickets/:id/body and the CodeMirror editor exist, AppState carries `root` and `config`.

---

**1. Backend — apm-server/src/routes/tickets.rs**

Add a `create_ticket` handler for `POST /api/tickets`:

Request body:
```rust
#[derive(serde::Deserialize)]
struct CreateTicketRequest {
    title: String,
    problem: Option<String>,
    acceptance_criteria: Option<String>,
    out_of_scope: Option<String>,
    approach: Option<String>,
}
```

Handler logic:
1. Deserialise the request body; return 400 if `title` is empty or missing.
2. Build `section_sets: Vec<(String, String)>` from the optional fields — include only non-empty values:
   - ("Problem", problem)
   - ("Acceptance criteria", acceptance_criteria)
   - ("Out of scope", out_of_scope)
   - ("Approach", approach)
3. Determine `author`: use `state.config.apm.author.clone()` (or a fallback string "apm-ui" if the field is absent from config).
4. Call `apm_core::ticket::create` with the verified signature (confirmed in `apm-core/src/ticket.rs:392`):
   ```rust
   tokio::task::spawn_blocking(move || {
       apm_core::ticket::create(
           &root,
           &config,
           title,
           author,
           None,   // context
           None,   // context_section
           false,  // aggressive — controls remote push after creation; always false for the server
           section_sets,
       )
   })
   ```
   The `aggressive: bool` parameter (line 399 in apm-core) pushes the new ticket branch to a remote when `true`. The server operates on the local repo and must not push automatically, so pass `false`.
5. On `Ok(ticket)`: return `Json(TicketResponse::from(&ticket))` with status 201.
6. On `Err(e)`: return 500 with `Json(serde_json::json!({"error": e.to_string()}))`.

Register the route in the router (main.rs or router setup):
```rust
.route("/api/tickets", get(list_tickets).post(create_ticket))
```

`TicketResponse` is the same struct already used by the GET endpoints (frontmatter + body). No new type needed.

---

**2. Frontend — apm-ui/src/components/NewTicketModal.tsx** (new file)

A modal component built with shadcn/ui `Dialog`:
- Title input: `<Input>` marked required; focus on open.
- Four optional `<Textarea>` fields: Problem, Acceptance Criteria, Out of Scope, Approach. Each labelled clearly.
- Submit button: disabled + shows spinner while mutation is in flight.
- Cancel button + Escape key: close without submitting.
- Client-side validation: if title is empty on submit, set an error state and display an inline message; do not call the API.
- Use TanStack Query `useMutation`:
  ```ts
  useMutation({
    mutationFn: (data) => fetch('/api/tickets', { method: 'POST', body: JSON.stringify(data), headers: { 'Content-Type': 'application/json' } }).then(r => { if (!r.ok) throw new Error(...); return r.json(); }),
    onSuccess: () => { queryClient.invalidateQueries({ queryKey: ['tickets'] }); setOpen(false); },
  })
  ```
- On API error: display `error.message` inline above the submit button.

---

**3. Zustand store — apm-ui/src/store.ts**

Add two fields:
```ts
newTicketOpen: boolean;        // default false
setNewTicketOpen: (v: boolean) => void;
```

---

**4. Wire up in layout — apm-ui/src/App.tsx (or WorkScreen.tsx)**

- Render `<NewTicketModal open={newTicketOpen} onOpenChange={setNewTicketOpen} />` at the root level.
- Add a "+ New ticket" button in the supervisorview column header; `onClick` calls `setNewTicketOpen(true)`.
- Add a `keydown` listener (on `document`, or via the existing keyboard shortcut system from Step 4/6): when key is `'n'` and `document.activeElement` is not an input/textarea/contenteditable, call `setNewTicketOpen(true)`.

---

**Files to change:**
- `apm-server/src/routes/tickets.rs` — add `create_ticket` handler
- `apm-server/src/main.rs` (or router file) — register `.post(create_ticket)` on `/api/tickets`
- `apm-ui/src/components/NewTicketModal.tsx` — new file
- `apm-ui/src/store.ts` — add `newTicketOpen` + `setNewTicketOpen`
- `apm-ui/src/App.tsx` or `WorkScreen.tsx` — render modal, add button, wire keyboard shortcut

**Order of changes:**
1. Backend endpoint (can be tested with curl independently)
2. Zustand store field
3. NewTicketModal component
4. Wire modal + button + shortcut into layout

### Open questions



### Amendment requests

- [x] Verify the `ticket::create` function signature before writing the handler — specifically whether it takes an `aggressive: bool` parameter added in a recent refactor. Update the example code in the Approach to match the actual current signature.

## History

| When | From | To | By |
|------|------|----|----|
| 2026-03-31T06:12Z | — | new | apm |
| 2026-03-31T06:53Z | new | in_design | philippepascal |
| 2026-03-31T06:57Z | in_design | specd | claude-0330-0700-b2e4 |
| 2026-03-31T18:15Z | specd | ammend | claude-0331-1200-a7b9 |
| 2026-03-31T19:10Z | ammend | in_design | philippepascal |
| 2026-03-31T19:13Z | in_design | specd | claude-0331-1910-8c40 |
| 2026-03-31T19:44Z | specd | ready | apm |
| 2026-04-01T05:14Z | ready | in_progress | philippepascal |
| 2026-04-01T05:22Z | in_progress | implemented | claude-0401-0514-2920 |
| 2026-04-01T05:26Z | implemented | accepted | apm |
| 2026-04-01T06:20Z | accepted | closed | apm-sync |