+++
id = "102511b9"
title = "apm-ui: worker queue shows 'crashed' for tickets in terminal states"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
agent = "philippepascal"
branch = "ticket/102511b9-apm-ui-worker-queue-shows-crashed-for-ti"
created_at = "2026-04-01T06:10:23.311626Z"
updated_at = "2026-04-01T06:13:02.387897Z"
+++

## Spec

### Problem

Workers assigned to tickets that have reached terminal states (implemented, accepted, closed, specd, etc.) show status 'crashed' in the worker queue UI. They should show 'ended' with a gray/neutral style instead.

The set of terminal states is config-dependent — it should not be hardcoded. The server or client should derive which states are terminal from the workflow config (states with no outgoing transitions, or a designated property). The worker queue panel should use this to distinguish a crashed process from one that simply completed its work.

What is broken or missing, and why it matters.

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
| 2026-04-01T06:10Z | — | new | philippepascal |
| 2026-04-01T06:13Z | new | in_design | philippepascal |
