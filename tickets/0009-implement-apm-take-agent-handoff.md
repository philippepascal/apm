+++
id = 9
title = "Implement apm take (agent handoff)"
state = "closed"
priority = 5
effort = 3
risk = 2
updated_at = "2026-03-27T00:06:00.834167Z"
+++

## Spec

### Amendment requests
- [x] how does an agent makes the distinction between take and a regular start? is there a difference in the state machine that needs to be added? even with that, how does an agent know if another agent is working on it?

  The distinction is the `agent` field: `apm start` requires `agent = null` (fails
  immediately if already set); `apm take` requires `agent` is set (fails if null —
  use `start` instead). `apm next` only surfaces tickets where `agent` is null, so
  an agent following normal workflow never encounters a claimed ticket. `take` is
  explicitly invoked when a supervisor directs an agent to resume someone else's work.
  No state machine change needed: `take` does not change state, only the `agent` field.

### Problem

When an agent session ends mid-ticket (crash, context limit, manual stop), another
agent needs to pick up the work. `apm take <id>` is the handoff command: it updates
the `agent` field to the new agent's name and checks out the ticket branch.
Without it, resuming in-progress work requires manual git and file editing.

### Acceptance criteria

- [ ] `apm take <id>` fails if `APM_AGENT_NAME` is not set
- [ ] Fails with a clear error if the ticket's `agent` field is null ("no agent assigned — use `apm start` instead")
- [ ] Fails with a clear error if the ticket is not in `in_progress` or `implemented` state
- [ ] No-op with a message if `APM_AGENT_NAME` already matches `frontmatter.agent`
- [ ] Updates `frontmatter.agent` to `APM_AGENT_NAME` and `frontmatter.updated`
- [ ] The commit message identifies both the outgoing and incoming agent: `ticket(<id>): agent handoff <old> → <new>`
- [ ] Commits frontmatter update to the ticket's `ticket/<id>-<slug>` branch via `git::commit_to_branch`
- [ ] If the branch is not present locally, fetches from origin first; then checks out the branch

### Out of scope

- Supervision handoff (`apm supervise`) — different field, same pattern
- Precondition checks on the state machine config

### Approach

New subcommand `apm take <id>` in `apm/src/cmd/take.rs`:
1. Load config and ticket; check state is `in_progress` or `implemented`
2. Fail if `frontmatter.agent` is null
3. Read `APM_AGENT_NAME` from env; no-op if already the current agent
4. Record outgoing agent; set `frontmatter.agent`, `frontmatter.updated`
5. Append history row; serialize
6. Determine branch from `frontmatter.branch` or `git::branch_name_from_path`
7. Call `git::commit_to_branch(root, &branch, &rel_path, &content, &msg)`
8. If branch not present locally: `git fetch origin <branch>`; then `git checkout <branch>`

## History

| Date | Actor | Transition | Note |
|------|-------|------------|------|
| 2026-03-25 | manual | new → specd | |
| 2026-03-25 | manual | specd → ammend | |
| 2026-03-25 | manual | ammend → specd | |
| 2026-03-25 | manual | specd → ready | |
| 2026-03-26 | manual | ready → ready | Respec: commit to ticket branch, not main |
| 2026-03-26 | manual | ready → specd | |
| 2026-03-26 | manual | specd → ready | |
| 2026-03-27T00:06Z | ready | closed | apm |