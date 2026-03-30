+++
id = "18c00750"
title = "apm work --dry-run: fix agent.is_none() filter and use pick_next"
state = "new"
priority = 0
effort = 0
risk = 0
author = "claude-0330-0245-main"
branch = "ticket/18c00750-apm-work-dry-run-fix-agent-is-none-filte"
created_at = "2026-03-30T06:11:15.954147Z"
updated_at = "2026-03-30T06:11:15.954147Z"
+++

## Spec

### Problem

`run_dry` in `apm/src/cmd/work.rs` has two issues:

1. It still has the `fm.agent.is_none()` filter that was removed from `next.rs`
   and `start.rs` — causing it to skip `ready` tickets whose `agent` field was
   set by spec authorship, and report fewer candidates than will actually be
   dispatched.

2. It duplicates the candidate-filtering and sorting logic instead of calling
   `ticket::pick_next`, which was extracted specifically to avoid this drift.

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
| 2026-03-30T06:11Z | — | new | claude-0330-0245-main |
