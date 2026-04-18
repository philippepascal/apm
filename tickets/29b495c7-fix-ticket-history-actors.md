+++
id = "29b495c7"
title = "fix ticket history actors"
state = "new"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/29b495c7-fix-ticket-history-actors"
created_at = "2026-04-18T02:20:26.518634Z"
updated_at = "2026-04-18T02:20:26.518634Z"
+++

## Spec

### Problem

In this example:

History

When	From	To	By
2026-04-18T01:16Z	—	new	philippepascal
2026-04-18T01:16Z	new	groomed	apm
2026-04-18T01:16Z	groomed	in_design	philippepascal
2026-04-18T01:19Z	in_design	specd	claude-0418-0116-80b8
2026-04-18T02:03Z	specd	ready	apm
2026-04-18T02:03Z	ready	in_progress	philippepascal
2026-04-18T02:06Z	in_progress	implemented	claude-0418-0203-b318
2026-04-18T02:10Z	implemented	closed	apm-sync


"new groomed apm" should be "new groomed philippepascal"
"specd ready apm" should be "specd ready philippepascal"
"implemented closed apm-sync" should be "implemented closed philippepascal(apm-sync)"

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
| 2026-04-18T02:20Z | — | new | philippepascal |
