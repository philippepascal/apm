+++
id = "177b68b3"
title = "UI shows assembled worker prompt for a ticket"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/177b68b3-ui-shows-assembled-worker-prompt-for-a-t"
created_at = "2026-05-14T21:14:45.432859Z"
updated_at = "2026-05-15T01:46:03.322394Z"
depends_on = ["ba121f45", "de2588b4"]
+++

## Spec

### Problem

The apm UI's ticket-detail view has no way to inspect the system prompt a worker would receive before dispatch. The only path is to launch a live worker, which is slow and gives no chance to catch misconfigured agents or instructions before they consume compute. After ba121f45 and de2588b4 land, `build_system_prompt()` is deterministic and accessible via `apm prompt <id>` — but only from the CLI.\n\nThis ticket wires that capability into the UI. The goal is twofold: supervisors can verify "is this really the prompt my worker will see?" before clicking a transition button, and they can experiment with different agent-name overrides without committing to them, which is the primary debugging path for small-model agents (pi, phi4, etc.) that misbehave unexpectedly.

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
| 2026-05-14T21:14Z | — | new | philippe|philippepascal |
| 2026-05-14T21:22Z | new | groomed | philippe |
| 2026-05-15T01:46Z | groomed | in_design | philippe |