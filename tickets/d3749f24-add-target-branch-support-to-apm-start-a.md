+++
id = "d3749f24"
title = "Add target_branch support to apm start and worktree provisioning"
state = "in_design"
priority = 8
effort = 0
risk = 0
author = "claude-0401-2145-a8f3"
agent = "77938"
branch = "ticket/d3749f24-add-target-branch-support-to-apm-start-a"
created_at = "2026-04-01T21:55:32.694733Z"
updated_at = "2026-04-02T00:43:35.125120Z"
+++

## Spec

### Problem

When a ticket belongs to an epic, its worktree must be provisioned from the epic branch (not `main`), and its PR must target the epic branch (not `main`). Currently `apm start` always uses `config.project.default_branch` for both: as the merge source when setting up the worktree, and as the `--base` argument when creating the PR via `gh_pr_create_or_update`.

The `docs/epics.md` design (§ Workflow integration) specifies that when `target_branch` is set in ticket frontmatter, `apm start` should merge from that branch into the worktree instead of the default branch. The PR creation call site should also pass `target_branch` as the base. Tickets without `target_branch` are completely unaffected.

The `target_branch` field does not yet exist on the `Frontmatter` struct in `apm-core/src/ticket.rs`, so it must be added before the two call-site changes can be made.

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