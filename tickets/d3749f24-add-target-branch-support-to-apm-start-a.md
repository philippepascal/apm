+++
id = "d3749f24"
title = "Add target_branch support to apm start and worktree provisioning"
state = "in_design"
priority = 8
effort = 0
risk = 0
author = "claude-0401-2145-a8f3"
agent = "philippepascal"
branch = "ticket/d3749f24-add-target-branch-support-to-apm-start-a"
created_at = "2026-04-01T21:55:32.694733Z"
updated_at = "2026-04-02T00:43:35.125120Z"
+++

## Spec

### Problem

When a ticket belongs to an epic, its worktree must be provisioned from the epic branch (not `main`), and its PR must target the epic branch (not `main`). Currently `apm start` always uses the default branch for both.

The full design is in `docs/epics.md` (§ Workflow integration — `apm start` and Completion strategy). When `target_branch` is set in the ticket frontmatter, `apm start` provisions the worktree from that branch tip instead of the default branch. The `gh_pr_create_or_update` call site passes `ticket.frontmatter.target_branch.as_deref().unwrap_or(default_branch)` — described in the spec as a one-line change. Tickets without `target_branch` are completely unaffected.

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
| 2026-04-01T21:55Z | — | new | claude-0401-2145-a8f3 |
| 2026-04-01T22:00Z | new | groomed | claude-0401-2145-a8f3 |
| 2026-04-02T00:43Z | groomed | in_design | philippepascal |
