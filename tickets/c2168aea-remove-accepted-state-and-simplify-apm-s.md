+++
id = "c2168aea"
title = "Remove accepted state and simplify apm sync to hardcode merged-PR-to-closed"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "apm"
agent = "philippepascal"
branch = "ticket/c2168aea-remove-accepted-state-and-simplify-apm-s"
created_at = "2026-04-01T20:26:50.809264Z"
updated_at = "2026-04-01T20:29:57.337864Z"
+++

## Spec

### Problem

Two related changes: (1) Remove the 'accepted' state from the default state machine. It is redundant — PR approval is expressed by the merge itself. Tickets should go from 'implemented' directly to 'closed'. (2) Simplify apm sync: instead of reading 'completion' fields in transition config to decide which tickets to check, apm sync should hardcode one rule: scan all non-terminal tickets, check if the ticket's branch PR is merged on GitHub, and if so transition directly to 'closed'. This removes the config dependency entirely. The 'completion' field in TransitionConfig can stay (it drives side effects on transition), but apm sync must not rely on it to decide what to scan. Any code that checks for 'accepted' as a state or transitions to it must be removed or updated.

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
| 2026-04-01T20:26Z | — | new | apm |
| 2026-04-01T20:29Z | new | in_design | philippepascal |
