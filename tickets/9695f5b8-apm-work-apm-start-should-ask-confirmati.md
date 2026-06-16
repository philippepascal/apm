+++
id = "9695f5b8"
title = "apm work, apm start, should ask confirmation if a ticket in their actionable list is in an epic that needs refresh"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/9695f5b8-apm-work-apm-start-should-ask-confirmati"
created_at = "2026-06-16T18:08:19.018981Z"
updated_at = "2026-06-16T18:13:21.075624Z"
+++

## Spec

### Problem

When `apm start <id>` or `apm work` picks a ticket whose parent epic is behind the default branch (`behind_count > 0`), they proceed silently. A worker spawned under a stale epic branch may build on a snapshot that is missing recent commits, then collide with `apm epic refresh` later — creating unnecessary merge conflicts or duplicate work.

The same gap exists in the web UI. `WorkEngineControls` shows an epic dropdown and a "Start" button but gives no indication when the chosen epic (or any epic with actionable tickets, in "All" mode) is stale. A supervisor starting the work engine through the UI has no visual cue that a refresh is needed first.

### Acceptance criteria

- [ ] `apm start <id>` prints a warning and prompts for confirmation (default yes) when the ticket's epic has `behind_count > 0` and stdout is a terminal; the ticket is NOT started if the user answers "n".
- [ ] `apm start <id>` writes a warning to stderr and proceeds without prompting when stdout is not a terminal and the ticket's epic is stale.
- [ ] `apm start <id>` proceeds normally without any warning when the ticket has no epic, or the epic is up to date.
- [ ] `apm work` (non-daemon) logs a warning line to stdout when it dispatches a ticket whose epic has `behind_count > 0`, before printing the "Dispatched worker" line.
- [ ] `apm work --daemon` logs the same warning line when dispatching from a stale epic.
- [ ] The web UI `WorkEngineControls` shows a visible warning near the "Start" button when the selected epic has `behind_count > 0`.
- [ ] The web UI `WorkEngineControls` shows a visible warning near the "Start" button when "All" is selected and at least one epic has `behind_count > 0`.
- [ ] The warning message in all contexts includes the epic ID and the number of commits it is behind.

### Out of scope

- Blocking (hard-erroring) `apm start` when an epic is stale — this ticket only adds a warning and a confirmable prompt.
- Automatically running `apm epic refresh` before starting — the user must refresh manually.
- Checking freshness for tickets that have no epic (i.e., tickets on the default branch).
- Any changes to `apm work --daemon` interactive prompting — daemon mode is inherently non-interactive; it logs a warning and continues.
- Filtering which epics trigger the warning based on whether they have actionable tickets — any stale epic triggers the warning.

### Approach

How the implementation will work.

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-06-16T18:08Z | — | new | philippepascal |
| 2026-06-16T18:09Z | new | groomed | philippepascal |
| 2026-06-16T18:13Z | groomed | in_design | philippepascal |