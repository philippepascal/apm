+++
id = 72
title = "Fix pre-existing take test worktree path collision"
state = "new"
priority = 0
effort = 0
risk = 0
author = "claude-0329-1430-main"
branch = "ticket/0072-fix-pre-existing-take-test-worktree-path"
created_at = "2026-03-29T23:58:55.795975Z"
updated_at = "2026-03-29T23:58:55.795975Z"
+++

## Spec

### Problem

take_appends_handoff_history, take_succeeds_on_ammend_state, take_succeeds_on_blocked_state fail because worktree paths already exist at test time. Unrelated to ticket #65 — the root integration.rs also had an unclosed brace in sync_auto_accept_transitions_implemented_ticket_to_accepted which was fixed as part of #65 work.

### Acceptance criteria

### Out of scope

### Approach

## History

| When | From | To | By |
|------|------|----|----|
| 2026-03-29T23:58Z | — | new | claude-0329-1430-main |
