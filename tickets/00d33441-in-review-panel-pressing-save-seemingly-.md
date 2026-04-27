+++
id = "00d33441"
title = "in review panel, pressing save seemingly attempts to save front matter and history"
state = "in_design"
priority = 0
effort = 4
risk = 2
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/00d33441-in-review-panel-pressing-save-seemingly-"
created_at = "2026-04-27T22:04:31.099252Z"
updated_at = "2026-04-27T22:21:44.370729Z"
+++

## Spec

### Problem

The ReviewEditor (`apm-ui/src/components/ReviewEditor.tsx`) initialises its CodeMirror editor with the **full raw ticket file** — TOML front matter, spec body, and the `## History` table all included. When the user clicks Save, `handleSave()` captures the entire editor document and sends it to `PUT /api/tickets/{id}/body`. From the user's perspective, pressing Save appears to be writing the front matter block and the history log, which they never intended to touch.

The backend does guard against actual corruption: `put_body()` extracts the front matter and history from both the on-disk file and the submitted content, compares them, and rejects the request if either has changed. So data is not being lost. But the UX signal is wrong — Save is doing far more work than the user expects, and any editor tooling (linters, word counts, diff previews) operates on content the user should never see.

The CLI `apm review` command already solves this correctly: it calls `split_body()` to isolate the spec section before opening the editor, then calls `apply_review()` to reconstruct the full file after the edit. The web UI should follow the same pattern — the editor should contain only the spec, and the API endpoint should accept only the spec.

### Acceptance criteria

- [ ] When the review panel editor opens, it contains only the spec portion of the ticket (the text between the closing `+++` and `## History`) — front matter and history are not visible in the editor
- [ ] When the user clicks Save in the review panel, the payload sent to `PUT /api/tickets/{id}/body` contains only the spec text (no `+++` delimiters, no `## History` section)
- [ ] After a successful save, the ticket file on disk still contains the original front matter unchanged
- [ ] After a successful save, the ticket file on disk still contains the original history table unchanged
- [ ] The backend `put_body` handler returns a 400 error when the submitted body contains a front matter delimiter (`+++`)
- [ ] The backend `put_body` handler returns a 400 error when the submitted body contains a `## History` section
- [ ] A valid spec-only save still results in the full ticket file being committed to git (front matter + new spec + history)

### Out of scope

- State transitions triggered from the review panel (existing logic, not changed by this ticket)\n- Markdown preview or rendering in the review panel\n- Any editor or panel other than ReviewEditor\n- The CLI `apm review` command (already correct)

### Approach

The fix narrows the contract on both sides: the frontend sends only the spec, and the backend accepts only the spec and reconstructs the full file itself.

**Backend — `apm-server/src/handlers/tickets.rs`**

1. In the `GET /api/tickets/{id}` response, add a `spec` field populated by calling `split_body()` on the serialized ticket and extracting only the spec portion. This avoids adding a new endpoint.
2. In `put_body()`, change the accepted request body from `{ content: String }` (full file) to `{ spec: String }` (spec only).
3. Add an early validation step: if the incoming `spec` contains `+++` or `\n## History`, return HTTP 400 with a descriptive message.
4. Load the existing ticket from disk. Use `split_body()` to extract its current front matter and history.
5. Reconstruct the full file: `front_matter + "\n\n" + new_spec + "\n\n## History\n" + history_rows`.
6. Write the reconstructed content to disk and commit.
7. Remove the old front matter / history extraction-and-comparison logic — it is no longer needed because the endpoint no longer accepts those sections.

**Frontend — `apm-ui/src/components/ReviewEditor.tsx`**

1. In `fetchTicket()`, read `ticket.spec` (the new field) instead of `ticket.raw` to populate `initialDoc`.
2. Remove the `EditorState.changeFilter` extension that was blocking edits to front matter and history ranges — it is no longer needed since those sections are absent from the editor.
3. In `handleSave()`, send `{ spec: viewRef.current.state.doc.toString() }` instead of `{ content: ... }`.

**Order of steps**

1. Add `spec` to the ticket API response (backend) — unblocks frontend work.
2. Update `put_body` to accept `spec` and reconstruct the full file (backend).
3. Add the two validation guards to `put_body` (backend).
4. Update ReviewEditor to use `ticket.spec` as the initial doc (frontend).
5. Update `handleSave` to send `{ spec }` (frontend).
6. Remove the `changeFilter` (frontend).
7. Update or add tests for the narrowed `put_body` contract.

**Constraints**

- The `split_body()` and `apply_review()` utilities in `apm-core/src/review.rs` already exist and can be reused directly — no new parsing logic needed.
- The change to `put_body`'s request shape (`content` → `spec`) is a breaking API change. Confirm no other callers send `content` before removing the old field; if the CLI ever calls this endpoint directly, update it too.

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-04-27T22:04Z | — | new | philippepascal |
| 2026-04-27T22:04Z | new | groomed | philippepascal |
| 2026-04-27T22:17Z | groomed | in_design | philippepascal |