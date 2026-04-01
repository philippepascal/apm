+++
id = "492e9f6b"
title = "apm-ui: worker queue should not show in_design tickets"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
agent = "philippepascal"
branch = "ticket/492e9f6b-apm-ui-worker-queue-should-not-show-in-d"
created_at = "2026-04-01T06:14:29.015814Z"
updated_at = "2026-04-01T06:23:41.689841Z"
+++

## Spec

### Problem

The worker queue panel displays tickets in in_design state. These tickets are already claimed by a spec agent and are being actively worked on — they are not waiting to be picked up. Only truly queued/waiting states (e.g. new, ready) should appear in the queue panel. in_design (and likely in_progress) should be excluded. The set of states to exclude should be derived from config where possible, but at minimum in_design and in_progress must be filtered out.

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
| 2026-04-01T06:14Z | — | new | philippepascal |
| 2026-04-01T06:23Z | new | in_design | philippepascal |
