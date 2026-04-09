+++
id = "a6c115e1"
title = "apm-ui: markdown editor with RO/RW sections (CodeMirror 6) and save API"
state = "closed"
priority = 40
effort = 6
risk = 5
author = "apm"
agent = "87568"
branch = "ticket/a6c115e1-apm-ui-markdown-editor-with-ro-rw-sectio"
created_at = "2026-03-31T06:12:48.893575Z"
updated_at = "2026-04-01T04:55:19.164645Z"
+++

## Spec

### Problem

The ticket detail panel (right column, Step 6) shows ticket content as read-only markdown. There is no way to edit a ticket body from the UI. The review button on the detail panel (Step 8) should open a full CodeMirror 6 editor occupying the supervisor-view and ticket-detail columns. The editor must enforce read-only constraints on the frontmatter block and the History section while allowing free editing everywhere else. Checkboxes must render as interactive UI elements. A new PUT /api/tickets/:id/body endpoint commits the edited content back to the ticket branch using the existing git::commit_to_branch function in apm-core.

### Acceptance criteria

- [x] Clicking the Review button on the ticket detail panel opens the CodeMirror 6 editor spanning the supervisor-view and ticket-detail columns; the worker view remains visible
- [x] The frontmatter block (the +++ delimited block at the top of the document) is read-only: the user cannot edit, delete, or insert text within it
- [x] The History section (from the ## History heading to end of document) is read-only: the user cannot edit, delete, or insert text within it
- [x] All other sections (Problem, Acceptance criteria, Out of scope, Approach, Open questions, Amendment requests) are fully editable
- [x] Checkboxes render as interactive HTML checkbox elements; clicking a checkbox toggles its checked state and updates the underlying markdown source accordingly
- [x] The editor has a Save button that calls PUT /api/tickets/:id/body with the full edited document text
- [x] PUT /api/tickets/:id/body returns 200 on success and commits the new content to the ticket branch via git::commit_to_branch
- [x] PUT /api/tickets/:id/body returns 404 when the ticket ID is not found
- [x] PUT /api/tickets/:id/body returns 422 when the submitted content modifies the frontmatter block or the History section relative to the current branch content
- [x] The editor has a Cancel button that closes the editor and returns to the read-only detail view
- [x] If there are unsaved changes, clicking Cancel shows a confirmation dialog before discarding
- [x] The review panel shows one state-transition button per valid next state; each button's keyboard shortcut is a single letter derived from the transition-shortcut-algorithm (the same algorithm used by the global keyboard handler)
- [x] A "Keep at {state}" button is always shown in the review panel with keyboard shortcut K; clicking it closes the editor without changing state

### Out of scope

- Editing frontmatter fields (title, state, priority, effort, risk) inline — covered by Step 13b
- New ticket creation form — covered by Step 10
- Live collaboration or auto-save
- Syntax highlighting beyond the standard CodeMirror markdown mode
- Diff view or version history

### Approach

Backend — apm-server

1. Add route: PUT /api/tickets/:id/body
   - Request body: JSON { "content": "<full raw document text>" }
   - The "full raw document" is what the editor contains, including the +++ frontmatter block

2. Handler logic:
   a. Resolve ticket ID to branch name (same helper as existing GET endpoints)
   b. Read current ticket content from git via git::read_from_branch
   c. Parse current content with Ticket::parse to extract current frontmatter TOML and History section text
   d. Parse submitted content the same way
   e. Guard: if submitted frontmatter TOML differs from current → return 422 "frontmatter is read-only"
   f. Guard: extract ## History block from both current and submitted; if they differ → return 422 "history section is read-only"
   g. Write the submitted content to the ticket file via git::commit_to_branch(root, branch, rel_path, content, "ui: edit ticket body")
   h. Return 200 { "ok": true }

3. Error responses:
   - 404: ticket not found
   - 422: RO section tampered (frontmatter or History differ)
   - 500: git operation failed

4. Files to change:
   - apm-server/src/routes/tickets.rs — add put_body handler
   - apm-server/src/main.rs or router — register .put(put_body) on the /api/tickets/:id/body path

Frontend — apm-ui

5. Add ReviewEditor component (apm-ui/src/components/ReviewEditor.tsx):
   a. Uses @codemirror/lang-markdown (already in the dep list; add if not)
   b. On mount, compute two protected ranges from the document string:
      - Frontmatter range: position 0 to end of closing "+++" line (inclusive)
      - History range: from position of "\n## History" to end of document
   c. Install a changeFilter transaction extension that rejects any transaction whose changes intersect either protected range. This is the correct CodeMirror 6 approach for per-range read-only (EditorState.readOnly is document-wide; changeFilter is per-range).
   d. Install a ViewPlugin that decorates "- [ ] " and "- [x] " lines with Decoration.widget (an HTML checkbox). The widget dispatches a transaction on click that replaces the "[ ]"/"[x]" text in the source.
   e. Toolbar:
      - Save button (calls PUT /api/tickets/:id/body)
      - Cancel / "Keep at {state}" button (keyboard shortcut K): closes editor without changing state; prompts if dirty
      - One state-transition button per valid next state for the current ticket; each button's keyboard shortcut label is derived by the same transition-shortcut-algorithm used by the global keyboard handler. Clicking a transition button first saves (PUT /api/tickets/:id/body) then calls the state-transition API.
   f. Dirty tracking: compare current doc to initial doc string; if dirty and Cancel / "Keep at {state}" clicked, show window.confirm before closing

6. Modify TicketDetail (apm-ui/src/components/TicketDetail.tsx):
   - When reviewMode is false: show existing read-only markdown view
   - When reviewMode is true: render ReviewEditor in place of both supervisor-view and ticket-detail columns (hide supervisorview, expand editor to fill both)

7. Zustand store (apm-ui/src/store.ts):
   - Add reviewMode: boolean (default false)
   - Add setReviewMode(v: boolean) action

8. Review button (already added in Step 8 per the spec):
   - Wire onClick to setReviewMode(true)

9. On successful save: call queryClient.invalidateQueries for the ticket query, then setReviewMode(false)

Key constraints:
- changeFilter is the right CodeMirror 6 API for per-range read-only. Do NOT use Compartment-scoped EditorState.readOnly (that is document-wide). The changeFilter extension should be a StateField or a plain extension added at editor init.
- The PUT endpoint receives the FULL document content (frontmatter included), not just the body. The server validates and commits atomically. This avoids split/reconstruct logic on the server and matches what the editor displays.
- History section boundary: from the line matching /^## History/ to the end of the file. The guard compares this substring between current and submitted content.
- Step 8 must be merged (state: implemented) before this ticket is set to ready.

### Open questions



### Amendment requests

- [x] Remove Acceptance Criterion "Cmd+S / Ctrl+S inside the editor triggers the save action" — Ctrl+S is not a keyboard shortcut in the editor per the updated keyboard spec
- [x] Add Acceptance Criterion: the review panel shows state transition buttons with computed keyboard shortcuts (one letter per transition target, derived per the transition-shortcut-algorithm ticket)
- [x] Add Acceptance Criterion: a "Keep at {state}" button is always shown in the review panel with keyboard shortcut `K`

## History

| When | From | To | By |
|------|------|----|----|
| 2026-03-31T06:12Z | — | new | apm |
| 2026-03-31T06:49Z | new | in_design | philippepascal |
| 2026-03-31T06:53Z | in_design | specd | claude-0330-0800-s9ed |
| 2026-03-31T18:14Z | specd | ammend | claude-0331-1200-a7b9 |
| 2026-03-31T19:16Z | ammend | in_design | philippepascal |
| 2026-03-31T19:18Z | in_design | specd | claude-0331-1430-b2f7 |
| 2026-03-31T19:44Z | specd | ready | apm |
| 2026-04-01T02:35Z | ready | in_progress | philippepascal |
| 2026-04-01T02:52Z | in_progress | implemented | claude-0401-0235-aef8 |
| 2026-04-01T04:07Z | implemented | accepted | apm-sync |
| 2026-04-01T04:55Z | accepted | closed | apm-sync |