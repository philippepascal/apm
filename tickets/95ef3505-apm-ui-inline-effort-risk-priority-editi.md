+++
id = "95ef3505"
title = "apm-ui: inline effort/risk/priority editing in ticket detail"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "apm"
agent = "80361"
branch = "ticket/95ef3505-apm-ui-inline-effort-risk-priority-editi"
created_at = "2026-03-31T06:13:16.584261Z"
updated_at = "2026-03-31T07:14:43.852798Z"
+++

## Spec

### Problem

In the ticket detail panel (Step 6), the `effort`, `risk`, and `priority` frontmatter fields are displayed as static text. Supervisors and spec-writers need to adjust these values frequently — particularly after reviewing a spec — without opening the full CodeMirror markdown editor introduced in Step 9.

Currently the only way to change these fields is via the CLI (`apm set <id> effort <n>`). The UI should provide click-to-edit inline controls directly in the detail panel header area so supervisors can update values with a single click and a keystroke.

The backend already exposes or will expose `PATCH /api/tickets/:id` (first introduced in Step 11 for priority reordering). This ticket extends that endpoint to accept `effort` and `risk` in addition to `priority`, and adds the corresponding inline UI controls for all three fields.

### Acceptance criteria


### Out of scope

Explicit list of what this ticket does not cover.

### Approach

How the implementation will work.

### Open questions



### Amendment requests



## History

| When | From | To | By |
|------|------|----|----|
| 2026-03-31T06:13Z | — | new | apm |
| 2026-03-31T07:14Z | new | in_design | philippepascal |