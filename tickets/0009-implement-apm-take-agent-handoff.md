+++
id = 9
title = "Implement apm take (agent handoff)"
state = "specd"
priority = 5
effort = 3
risk = 2
created = "2026-03-25"
updated = "2026-03-25"
+++

## Spec

### Problem

When an agent session ends mid-ticket (crash, context limit, manual stop), another
agent needs to pick up the work. `apm take <id>` is the handoff command: it checks
out the feature branch and updates the `agent` field to the new agent's name.
Without it, resuming in-progress work requires manual git and file editing.

### Acceptance criteria

- [ ] `apm take <id>` checks out the ticket's `branch` field (fails clearly if branch field is null)
- [ ] `agent` field is updated to `APM_AGENT_NAME` and committed to main
- [ ] The commit message identifies both the outgoing and incoming agent names
- [ ] `apm take` works on tickets in `in_progress` or `implemented` state; fails on others with a clear error
- [ ] If the branch does not exist locally, it is fetched from origin first
- [ ] Running `apm take` as the current agent (same `APM_AGENT_NAME`) is a no-op with a message

### Out of scope

- Supervision handoff (`apm supervise`) — different field, same pattern
- Precondition checks on the state machine config

### Approach

New subcommand `apm take <id>` in `apm/src/cmd/take.rs`:
1. Load ticket; check state is `in_progress` or `implemented`
2. Check `APM_AGENT_NAME` env var; no-op if already the current agent
3. Record outgoing agent name, update `frontmatter.agent`, `frontmatter.updated`
4. Append history row; save; `git add`, `git commit`, `git push origin main`
5. `git fetch origin` if branch not present locally; `git checkout <branch>`

## History

| Date | Actor | Transition | Note |
|------|-------|------------|------|
| 2026-03-25 | manual | new → specd | |
