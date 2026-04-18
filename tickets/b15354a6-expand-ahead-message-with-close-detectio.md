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

When `apm sync` detects that local `<default>` is ahead of `origin/<default>`, it prints a message that is accurate but silent about the most important consequence: **close detection is gated on origin visibility**. `apm sync` detects merged tickets by inspecting commits reachable from `origin/<default>`; unpushed local commits are invisible to that check. Users have hit this as a mystery — sync reports "ahead by 16 commits" and shows "no tickets to close", then immediately offers to close tickets after a `git push`. The causal link is missing from the message.

There is also a parity gap between the CLI and UI sync surfaces. The server handler (`apm-server/src/handlers/maintenance.rs`) discards warnings from `sync_non_checked_out_refs` entirely (the accumulator is named `_sync_warnings` and never read), and routes warnings from `sync_default_branch` to `eprintln!` (server stderr) rather than into the JSON `log` field. As a result, the UI sync modal never shows "main is ahead" or any ahead-of-origin messages for non-checked-out ticket/epic refs, even when those gaps are precisely what is blocking close detection. Users running the UI today get no signal that their local main is out of sync with origin.

### Acceptance criteria

- [ ] `MAIN_AHEAD` in `apm-core/src/sync_guidance.rs` includes a sentence explaining that merged tickets will not be detected as closeable until the user pushes
- [ ] When `apm sync` (CLI) runs and local default branch is ahead of origin, the expanded message appears on stderr
- [ ] When `POST /api/sync` runs and local default branch is ahead of origin, the `log` field in the JSON response contains the expanded `MAIN_AHEAD` message
- [ ] When `POST /api/sync` runs and one or more non-checked-out ticket or epic refs are ahead of origin, those `TICKET_OR_EPIC_AHEAD` messages appear in the `log` field (currently the warnings vector is discarded)
- [ ] The UI sync modal displays the "ahead" message when local main is ahead of origin
- [ ] The UI sync modal displays per-branch ahead warnings when non-checked-out ticket/epic refs are ahead of origin
- [ ] `apm sync` (CLI) behaviour for the happy path (no ahead condition) is unchanged

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