+++
id = "177b68b3"
title = "UI shows assembled worker prompt for a ticket"
state = "new"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/177b68b3-ui-shows-assembled-worker-prompt-for-a-t"
created_at = "2026-05-14T21:14:45.432859Z"
updated_at = "2026-05-14T21:21:32.702201Z"
depends_on = ["ba121f45", "de2588b4"]
+++

## Spec

### Problem

After `apm prompt` and the spawn-path integration land (tickets ba121f45 and de2588b4), expose the prompt-preview in the apm UI.

Today the UI has a ticket-detail view and a dispatch loop, but no way to see what system prompt a worker would receive. For debugging small-model behaviour (pi/phi4 etc.) and for letting a supervisor confirm 'is this what I think it is?' before clicking dispatch, the UI should be able to render the assembled prompt for the (ticket, agent, role) about to spawn.

Acceptance:
- ticket-detail page has a 'Show worker prompt' affordance.
- It renders the exact prompt that `apm prompt --ticket <id>` produces, fetched via apm-server.
- It updates when the supervisor changes the assigned agent via the UI's agent-override control.

Out of scope: editing the prompt from the UI.

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