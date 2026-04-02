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

- [ ] When a ticket has `target_branch = "epic/abc"` in its frontmatter, `apm start <id>` merges `epic/abc` (or `origin/epic/abc` if the remote ref exists) into the worktree instead of the default branch
- [ ] When a ticket has no `target_branch` field, `apm start <id>` behaves identically to before this change
- [ ] When a ticket has `target_branch` set and `apm state <id> implemented` is called, the PR is created with `--base epic/abc` (the target branch), not with `--base main`
- [ ] When a ticket has no `target_branch` field and `apm state <id> implemented` is called, the PR is created with `--base main` (the default branch) as before
- [ ] The `Frontmatter` struct round-trips a ticket file that contains `target_branch` without data loss
- [ ] The `Frontmatter` struct round-trips a ticket file that does not contain `target_branch` without adding the field to the serialised output

### Out of scope

- Setting `target_branch` automatically when a ticket is created under an epic (covered by a separate epic-creation ticket)
- The `epic` and `depends_on` frontmatter fields — not added here
- Validating that `target_branch` actually exists in the repo at the time `apm start` runs
- Any UI or `apm` CLI command changes to display or filter by `target_branch`
- Epic branch lifecycle (creation, merging the epic branch back to main)

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