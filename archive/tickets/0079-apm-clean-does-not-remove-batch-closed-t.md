+++
id = 79
title = "apm clean does not remove batch-closed ticket worktrees"
state = "closed"
priority = 0
effort = 0
risk = 0
author = "claude-0329-1430-main"
branch = "ticket/0079-apm-clean-does-not-remove-batch-closed-t"
created_at = "2026-03-30T01:20:28.276937Z"
updated_at = "2026-03-30T02:52:09.450116Z"
+++

## Spec

### Problem

batch_close in sync commits closed state to main but does not update or delete ticket branches. apm clean reads state from ticket branches (sees accepted, not terminal) and also checks git branch --merged (fails because batch_close bypasses the PR merge workflow). Result: worktrees for batch-closed tickets are never cleaned up.

### Acceptance criteria

### Out of scope

### Approach

## History

| When | From | To | By |
|------|------|----|----|
| 2026-03-30T01:20Z | — | new | claude-0329-1430-main |
| 2026-03-30T02:52Z | new | closed | apm |