+++
id = "00d33441"
title = "in review panel, pressing save seemingly attempts to save front matter and history"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/00d33441-in-review-panel-pressing-save-seemingly-"
created_at = "2026-04-27T22:04:31.099252Z"
updated_at = "2026-04-27T22:17:43.455676Z"
+++

## Spec

### Problem

The ReviewEditor (`apm-ui/src/components/ReviewEditor.tsx`) initialises its CodeMirror editor with the **full raw ticket file** — TOML front matter, spec body, and the `## History` table all included. When the user clicks Save, `handleSave()` captures the entire editor document and sends it to `PUT /api/tickets/{id}/body`. From the user's perspective, pressing Save appears to be writing the front matter block and the history log, which they never intended to touch.

The backend does guard against actual corruption: `put_body()` extracts the front matter and history from both the on-disk file and the submitted content, compares them, and rejects the request if either has changed. So data is not being lost. But the UX signal is wrong — Save is doing far more work than the user expects, and any editor tooling (linters, word counts, diff previews) operates on content the user should never see.

The CLI `apm review` command already solves this correctly: it calls `split_body()` to isolate the spec section before opening the editor, then calls `apply_review()` to reconstruct the full file after the edit. The web UI should follow the same pattern — the editor should contain only the spec, and the API endpoint should accept only the spec.

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
| 2026-04-27T22:04Z | — | new | philippepascal |
| 2026-04-27T22:04Z | new | groomed | philippepascal |
| 2026-04-27T22:17Z | groomed | in_design | philippepascal |