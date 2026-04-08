+++
id = "492e9f6b"
title = "apm-ui: worker queue should not show in_design tickets"
state = "closed"
priority = 0
effort = 1
risk = 1
author = "philippepascal"
agent = "9577"
branch = "ticket/492e9f6b-apm-ui-worker-queue-should-not-show-in-d"
created_at = "2026-04-01T06:14:29.015814Z"
updated_at = "2026-04-01T07:47:31.006Z"
+++

## Spec

### Problem

The worker queue panel in apm-ui fetches tickets from the /api/queue backend endpoint, which returns all tickets whose state is marked actionable = ["agent"] in .apm/config.toml. Currently the in_design state carries this flag, so tickets that are actively being worked by a spec-writer agent appear in the queue alongside tickets that are genuinely waiting to be picked up (new, ammend, ready).

This is misleading: an in_design ticket is already claimed — another agent trying to act on it would either duplicate work or collide with the current spec-writer. The queue panel should only show tickets that are actually waiting for an agent to start work on them.

in_progress is already excluded (no actionable field in config). The fix required is removing in_design from the set of agent-actionable states so it stops appearing in the queue.

### Acceptance criteria

- [x] The worker queue panel does not display tickets in `in_design` state
- [x] The worker queue panel continues to display tickets in `new` state
- [x] The worker queue panel continues to display tickets in `ammend` state
- [x] The worker queue panel continues to display tickets in `ready` state
- [x] The worker queue panel does not display tickets in `in_progress` state (existing behaviour, must not regress)
- [x] `apm next` does not return an `in_design` ticket as the next actionable item (consistent with queue behaviour)

### Out of scope

- Changes to the UI component (PriorityQueuePanel.tsx) or the /api/queue endpoint handler (queue.rs) — the filtering is config-driven
- Filtering by agent assignment (e.g. hiding tickets claimed by a different agent's name)
- Changing how the queue panel ranks or sorts entries
- Any changes to the in_progress state (already excluded from the queue)

### Approach

Root cause: In .apm/config.toml, the in_design state has actionable = ["agent"] at line 156. The /api/queue endpoint (apm-server/src/queue.rs) calls config.actionable_states_for("agent"), which collects all states with that flag. The resulting list is passed to apm_core::ticket::sorted_actionable, which filters tickets by state. Since in_design is marked actionable for agents, in_design tickets appear in the queue.

Fix (single-file change in .apm/config.toml):

Remove the actionable = ["agent"] line from the [[workflow.states]] block where id = "in_design" (currently line 156). No other files need to change — the filtering is entirely config-driven.

Why this is safe: No agent should ever discover an in_design ticket through the queue. A spec-writer claims a new or ammend ticket and immediately transitions it to in_design themselves. All transitions out of in_design (to specd, question, ammend, closed) are triggered explicitly by the agent already holding the ticket, not by queue discovery. Removing the actionable flag makes the config accurately reflect actual workflow semantics.

Verification steps:
- apm list --state in_design should list existing in_design tickets
- apm next should not return any of those tickets
- In the apm-ui queue panel, no in_design tickets should appear

### Open questions



### Amendment requests



## History

| When | From | To | By |
|------|------|----|----|
| 2026-04-01T06:14Z | — | new | philippepascal |
| 2026-04-01T06:23Z | new | in_design | philippepascal |
| 2026-04-01T06:26Z | in_design | specd | claude-0401-0623-4e38 |
| 2026-04-01T06:28Z | specd | ready | apm |
| 2026-04-01T07:17Z | ready | in_progress | philippepascal |
| 2026-04-01T07:19Z | in_progress | implemented | claude-0401-0717-fcc8 |
| 2026-04-01T07:46Z | implemented | accepted | apm-sync |
| 2026-04-01T07:47Z | accepted | closed | apm-sync |