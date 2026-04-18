+++
id = "b15354a6"
title = "Expand ahead message with close-detection context and surface in UI sync"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/b15354a6-expand-ahead-message-with-close-detectio"
created_at = "2026-04-18T02:21:44.835172Z"
updated_at = "2026-04-18T02:29:02.208567Z"
+++

## Spec

### Problem

When `apm sync` finds local `<default>` ahead of `origin/<default>`, it prints:

> `<default> is ahead of origin/<default> by N commits — run `git push` when ready`

This is factually correct but omits the most important consequence: **merged tickets will not be offered for closing until the merge commits are visible on `origin/<default>`**. Users hit this as a mystery (`apm sync` said "ahead by 16 commits", didn't offer to close a merged ticket; after `git push`, sync immediately offered to close it).

**Message improvement.** Expand the string to include the causal link, e.g.:

> `<default> is ahead of origin/<default> by N commits. Merged tickets will not be detected as closeable until you push — run `git push` when ready.`

Exact wording is for the implementer; the key is that the user learns from the message *why* pushing matters here, not just the bare fact that they're ahead. The `MAIN_AHEAD` constant already exists in `apm-core/src/sync_guidance.rs:67`; this ticket updates its body. `TICKET_OR_EPIC_AHEAD` (line 73) serves the analogous message for non-checked-out ticket/epic refs and should be considered alongside.

**UI surface.** The same informational message is not currently shown by the UI sync flow. `apm-ui/src/components/...` (the Sync screen / modal) calls `POST /api/sync` (or equivalent) on the server and displays the structured result, but the "main is ahead" / "ticket is ahead" lines generated in `sync_default_branch` and `sync_non_checked_out_refs` pass through `sync_warnings` in the CLI path and are printed to stderr. The server handler needs to surface these same lines into its JSON response, and the UI sync screen needs to render them alongside "no tickets to close" / "N ticket branches visible".

Users running the UI today get no signal that their local `main` is out of sync with origin, even when that exact gap is what's blocking close detection.

Trigger: user ran `apm sync` and the UI sync at roughly the same moment; the UI showed "synced non-checked-out refs / no tickets to close / 285 ticket branch(es) visible" while the CLI reported "main is ahead of origin/main by 16 commits". Parity between the two surfaces is required for the UI to be usable as a full replacement for the CLI in this flow.

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
| 2026-04-18T02:21Z | — | new | philippepascal |
| 2026-04-18T02:23Z | new | groomed | apm |
| 2026-04-18T02:29Z | groomed | in_design | philippepascal |
