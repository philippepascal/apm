+++
id = 7
title = "Implement apm start (branch creation, git mutex)"
state = "closed"
priority = 10
effort = 3
risk = 2
updated_at = "2026-03-27T00:06:00.519059Z"
+++

## Spec

### Amendment requests

- [x] Please specify who (agent or supervisor) is supposed to use this. is there any guard against wrong use?

  `apm start` is an **agent command** (`actor = "agent"` per the state machine).
  The guards are: (1) `APM_AGENT_NAME` must be set — unset means bare shell, not
  an agent session; (2) ticket must be in an `actionable_state`; (3) `agent` field
  must be null — if already set, the ticket is already claimed. There is no
  role-level check distinguishing agents from engineers: a supervisor who wants to
  do the work sets their username as `APM_AGENT_NAME`, same as any other actor.
  Enforcing `actor` rules from the state machine config is a separate, future ticket.

### Problem

`apm start <id>` is the entry point for implementation. In the branch-per-ticket
model, the `ticket/<id>-<slug>` branch already exists from `apm new` — no branch
creation is needed. `apm start` claims the ticket by setting `agent`, transitions
state to `in_progress`, and checks out the branch. The guard against concurrent
claims is the `agent` field: if already set, the command fails immediately before
making any changes.

### Acceptance criteria

- [ ] `apm start <id>` fails with a clear error if `APM_AGENT_NAME` is not set
- [ ] Fails with a clear error if the ticket's `agent` field is already set: "ticket already claimed — run `apm next`"
- [ ] Fails with a clear error if the ticket is not in an actionable state (as defined by `[agents] actionable_states` in `apm.toml`)
- [ ] Sets `frontmatter.agent = APM_AGENT_NAME`, `frontmatter.state = "in_progress"`, `frontmatter.updated`
- [ ] Appends a history row
- [ ] Commits frontmatter update to the ticket's `ticket/<id>-<slug>` branch via `git::commit_to_branch`
- [ ] Checks out `ticket/<id>-<slug>` in the working tree after the commit; fetches from origin first if the branch is not present locally

### Out of scope

- `apm take` (handoff from another agent) — tracked in #9
- `apm _hook pre-push` auto-transition — tracked in #8
- Precondition enforcement from the state machine config (spec checks, etc.)

### Approach

New subcommand `apm start <id>` in `apm/src/cmd/start.rs`:

1. Load config; collect `actionable_states` from `config.agents`
2. Load ticket; fail if state not in actionable_states
3. Fail if `frontmatter.agent` is already set
4. Read `APM_AGENT_NAME` from env; fail if unset
5. Set `frontmatter.agent`, `frontmatter.state = "in_progress"`, `frontmatter.updated`
6. Append history row; serialize
7. Determine branch: `frontmatter.branch` or `git::branch_name_from_path(&t.path)`
8. Call `git::commit_to_branch(root, &branch, &rel_path, &content, &msg)`
9. If branch not present locally: `git fetch origin <branch>`
10. `git checkout <branch>`

## History

| Date | Actor | Transition | Note |
|------|-------|------------|------|
| 2026-03-25 | manual | new → specd | |
| 2026-03-25 | manual | specd → ready | |
| 2026-03-26 | manual | ready → ready | Respec for branch-per-ticket model |
| 2026-03-26 | manual | ready → specd | |
| 2026-03-26 | manual | specd → ammend | |
| 2026-03-26 | manual | ammend → specd | Amendment addressed |
| 2026-03-26 | manual | ammend → specd | |
| 2026-03-26 | manual | specd → ready | |
| 2026-03-27T00:06Z | ready | closed | apm |