+++
id = "0fa737ae"
title = "UI: change display of max workers"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/0fa737ae-ui-change-display-of-max-workers"
created_at = "2026-04-28T19:24:12.894681Z"
updated_at = "2026-04-28T19:38:13.413510Z"
+++

## Spec

### Problem

The work engine controls UI currently shows `config: <max_concurrent>` — a single number. But the config actually carries three distinct limits: total max (`max_concurrent`), default-branch max (`max_workers_on_default`), and epic max (`max_workers_per_epic`). The display hides the per-branch and per-epic ceilings, so there is no way to tell from the UI what those values are without reading the config file directly.

Additionally, the label "workers" used for both the active static display and the editable field when the engine is stopped is ambiguous. "Override max" is more precise: it names what the control actually sets.

The fix is purely presentational: extend the API response to carry all three config values, update the config badge to show all three, and rename the "workers" label to "override max". No scheduling logic changes.

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
| 2026-04-28T19:24Z | — | new | philippepascal |
| 2026-04-28T19:32Z | new | groomed | philippepascal |
| 2026-04-28T19:38Z | groomed | in_design | philippepascal |