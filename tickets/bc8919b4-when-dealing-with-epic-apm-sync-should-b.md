+++
id = "bc8919b4"
title = "when dealing with epic, apm sync should be more advance"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/bc8919b4-when-dealing-with-epic-apm-sync-should-b"
created_at = "2026-09-01T18:00:23.105706Z"
updated_at = "2026-09-01T18:03:53.553660Z"
+++

## Spec

### Problem

After closing tickets and pushing tickets to origin.
if some of the tickets being closed by syn belong to an epic, it should check if that epic still has work to be done.
if not (all ticket of epic are closed) it should ask the user if they want to submit that epic to main, and how (merge, PR, auto). it should do it for each epic one by one, as choice may vary.
once done, for epics that have been submitted, if should ask if the user wants to close the epic.

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
| 2026-09-01T18:00Z | — | new | philippepascal |
| 2026-09-01T18:03Z | new | groomed | philippepascal |
| 2026-09-01T18:03Z | groomed | in_design | philippepascal |
