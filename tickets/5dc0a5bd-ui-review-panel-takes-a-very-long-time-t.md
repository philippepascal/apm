+++
id = "5dc0a5bd"
title = "UI review panel takes a very long time to close"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/5dc0a5bd-ui-review-panel-takes-a-very-long-time-t"
created_at = "2026-06-01T18:17:29.250149Z"
updated_at = "2026-06-01T18:17:36.453464Z"
+++

## Spec

### Problem

When a reviewer clicks a transition button in the review panel (e.g., "→ specd", "→ ammend"), the panel blocks on the HTTP response from `POST /api/tickets/{id}/transition` before calling `setReviewMode(false)`. That endpoint runs git operations inside a `spawn_blocking` task — at minimum a `commit_to_branch` call (which may create and tear down a temporary git worktree), and for completion strategies such as `Merge` also a `git push`, `git merge`, and another `git push`. End-to-end this can take 10–30 seconds or more. The panel stays open and unresponsive for the entire duration, with no progress indication. The `flushSync(() => setReviewMode(false))` call at line 194 of `ReviewEditor.tsx` is the specific statement gating the close on request completion.

The operation triggered by the button click does not need to complete before the panel can safely close. The transition is a fire-and-forget action from the reviewer's perspective — the result will be visible in the ticket list once the background git work finishes. Keeping the panel open during this wait gives the user no useful control and creates a perception of the app being frozen.

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
| 2026-06-01T18:17Z | — | new | philippepascal |
| 2026-06-01T18:17Z | new | groomed | philippepascal |
| 2026-06-01T18:17Z | groomed | in_design | philippepascal |