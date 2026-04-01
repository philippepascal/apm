+++
id = "492e9f6b"
title = "apm-ui: worker queue should not show in_design tickets"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
agent = "24073"
branch = "ticket/492e9f6b-apm-ui-worker-queue-should-not-show-in-d"
created_at = "2026-04-01T06:14:29.015814Z"
updated_at = "2026-04-01T06:23:41.689841Z"
+++

## Spec

### Problem

The worker queue panel in apm-ui fetches tickets from the /api/queue backend endpoint, which returns all tickets whose state is marked actionable = ["agent"] in .apm/config.toml. Currently the in_design state carries this flag, so tickets that are actively being worked by a spec-writer agent appear in the queue alongside tickets that are genuinely waiting to be picked up (new, ammend, ready).

This is misleading: an in_design ticket is already claimed — another agent trying to act on it would either duplicate work or collide with the current spec-writer. The queue panel should only show tickets that are actually waiting for an agent to start work on them.

in_progress is already excluded (no actionable field in config). The fix required is removing in_design from the set of agent-actionable states so it stops appearing in the queue.

### Acceptance criteria

- [ ] The worker queue panel does not display tickets in `in_design` state
- [ ] The worker queue panel continues to display tickets in `new` state
- [ ] The worker queue panel continues to display tickets in `ammend` state
- [ ] The worker queue panel continues to display tickets in `ready` state
- [ ] The worker queue panel does not display tickets in `in_progress` state (existing behaviour, must not regress)
- [ ] `apm next` does not return an `in_design` ticket as the next actionable item (consistent with queue behaviour)

### Out of scope

- Changes to the UI component (PriorityQueuePanel.tsx) or the /api/queue endpoint handler (queue.rs) — the filtering is config-driven
- Filtering by agent assignment (e.g. hiding tickets claimed by a different agent's name)
- Changing how the queue panel ranks or sorts entries
- Any changes to the in_progress state (already excluded from the queue)

### Approach

How the implementation will work.

### Open questions



### Amendment requests



## History

| When | From | To | By |
|------|------|----|----|
| 2026-04-01T06:14Z | — | new | philippepascal |
| 2026-04-01T06:23Z | new | in_design | philippepascal |