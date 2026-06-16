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
| 2026-06-16T18:08Z | — | new | philippepascal |
| 2026-06-16T18:09Z | new | groomed | philippepascal |
| 2026-06-16T18:13Z | groomed | in_design | philippepascal |