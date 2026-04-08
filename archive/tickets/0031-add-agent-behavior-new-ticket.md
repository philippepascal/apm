+++
id = 31
title = "add-agent-behavior-new-ticket"
state = "closed"
priority = 0
effort = 3
risk = 2
author = "apm"
agent = "claude-0327-2000-31cc"
branch = "ticket/0031-add-agent-behavior-new-ticket"
created_at = "2026-03-27T06:53:11.802207Z"
updated_at = "2026-03-30T02:02:46.501095Z"
+++

## Spec

### Problem

Agents working on a ticket regularly encounter issues outside the scope of their current task — a bug in an adjacent module, a missing test, a potential security concern. Currently there is no standard mechanism for capturing these observations. The agent either ignores them (the issue is lost) or acts on them inline (scope creep, risk to the current ticket). The desired behavior is for agents to create a lightweight side ticket so the supervisor can review and prioritize it. This keeps the discovering agent focused and ensures the observation is not lost. The behavior should be configurable so teams can turn it off.

### Acceptance criteria

- [ ] `apm new --side-note "<title>"` creates a ticket in `new` state with the creating agent as `author`
- [ ] A `--context "<text>"` flag lets the agent append a brief description to the `### Problem` section at creation time
- [ ] When `agents.side_tickets = false` in `apm.toml`, `apm new --side-note` exits with a clear error message
- [ ] When `agents.side_tickets` is omitted or `true`, the command succeeds (default-on)
- [ ] The created ticket has no `supervisor` or `agent` set — unassigned and ready for supervisor triage
- [ ] `apm.agents.md` documents the convention: when an agent notices an out-of-scope issue, run `apm new --side-note` and resume the current ticket

### Out of scope

- Auto-assigning or auto-prioritizing side tickets
- Linking side tickets back to the originating ticket (no `parent` field)
- Any integration with external issue trackers
- Notification mechanisms beyond what exists for new tickets

### Approach

**Config** (`apm-core/src/config.rs`): Add an optional `side_tickets: bool` field to `AgentsConfig`, defaulting to `true`. No new toml key required unless the user explicitly sets it to `false`.

**CLI** (`apm/src/cmd/new.rs`): Add a `--side-note` flag and a `--context` option to `apm new`. When `--side-note` is present: check `config.agents.side_tickets` first and error if false; if `--context` is provided, write it into the `### Problem` section of the new ticket body. Otherwise the command is identical to `apm new <title>`. No new subcommand needed — this keeps the surface minimal.

**Docs**: Update `apm.agents.md` to document the side-ticket convention.

## History

| When | From | To | By |
|------|------|----|----|
| 2026-03-27T06:53Z | — | new | apm |
| 2026-03-28T01:01Z | new | specd | claude-0327-1757-391b |
| 2026-03-28T01:04Z | specd | ready | apm |
| 2026-03-28T02:08Z | ready | in_progress | claude-0327-2000-31cc |
| 2026-03-28T02:11Z | in_progress | implemented | claude-0327-2000-31cc |
| 2026-03-28T07:31Z | implemented | accepted | apm sync |
| 2026-03-30T02:02Z | accepted | closed | apm-sync |