+++
id = "0083"
title = "apm.agents.md: define Delegator and Worker roles clearly"
state = "closed"
priority = 0
effort = 2
risk = 1
author = "claude-0330-0245-main"
agent = "claude-0330-0245-main"
branch = "ticket/0083-apm-agents-md-define-delegator-and-worke"
created_at = "2026-03-30T05:10:20.442934Z"
updated_at = "2026-03-30T18:07:43.300325Z"
+++

## Spec

### Problem

`apm.agents.md` is read by every Claude session in this repo — both the master
agent that orchestrates work and the worker subagents that implement tickets.
Currently it only describes how to work a single ticket; it says nothing about
the Delegator role, leaving the master agent to improvise. In practice this
causes the master agent to cherry-pick tickets, write specs itself, and narrate
instead of mechanically dispatching work via `apm start --next --spawn`.

Additionally, the Worker role is not explicitly signalled at spawn time, so a
spawned worker has no reliable way to know it is a worker rather than a new
master session.

### Acceptance criteria

- [x] `apm.agents.md` has a `## Roles` section that appears before any other
  behavioural instructions, defining the two roles and how to detect them
- [x] The role detection rule is explicit: if the session was started with a
  ticket assignment in the initial prompt → Worker; otherwise → Delegator
- [x] The Delegator section specifies the loop: call `apm start --next --spawn`
  repeatedly until it returns null (nothing ready) or max workers is reached
- [x] If max workers was not specified by the user, the Delegator must ask
  before starting the loop — it must not assume a default
- [x] The Delegator section explicitly prohibits: picking tickets manually,
  writing specs, implementing code, running `apm sync`, closing or transitioning
  tickets, or taking any action not driven by `apm start --next`
- [x] The Delegator section specifies what to do when the queue is empty or
  blocked: report back to the supervisor with a clear status — do not improvise
  or switch to worker behaviour
- [x] The Worker section specifies that the worker implements exactly one ticket
  and must not spawn further workers or act as delegator
- [x] The spawn prompt emitted by `apm start --spawn` includes a clear role
  line at the top: "You are a Worker agent assigned to ticket #<id>." so the
  worker can detect its role unambiguously
- [x] `cargo test --workspace` passes

### Out of scope

- A separate `apm.delegator.md` file — one file, two clearly separated sections
- Changing scheduling, monitoring, or concurrency limits in `apm start --spawn`

### Approach

**`apm.agents.md` — add `## Roles` section at the top**

Insert immediately after the repo structure note, before `## Ticket format`:

```markdown
## Roles

Every Claude session in this repo is either a **Delegator** (master agent) or
a **Worker** (subagent). Read your initial prompt to detect which you are.

**Role detection**
- If your initial prompt contains "You are a Worker agent assigned to ticket #N"
  → you are a **Worker**. Skip to the Worker section below.
- Otherwise → you are the **Delegator**. Follow the Delegator section below.

### Delegator

Your only job is to dispatch work to workers. You must not write specs,
implement code, choose tickets manually, run `apm sync`, close or transition
tickets, or take any action not driven by `apm start --next`.

**Before dispatching:**
1. If the user has not specified a maximum number of concurrent workers, ask.
   Do not assume a default.

**Dispatch loop:**
2. Call `apm start --next --spawn` (or `--spawn -P` for permissionless workers).
3. Repeat until `apm next` returns null (nothing ready) or max workers are running.

**When the queue is empty or all ready tickets are blocked:**
4. Report back to the supervisor with a clear status summary:
   - How many workers were spawned
   - Which tickets are blocking (specd/new/blocked) and why they can't be dispatched
   Do not improvise. Do not switch to worker behaviour.

### Worker

You have been assigned a single ticket. Implement it, run tests, open a PR,
and mark it implemented. Do not spawn further workers or act as delegator.
```

**Spawn prompt change**

In the code that builds the worker's initial prompt (in `apm-core`, the
`start --spawn` implementation), prepend the following line before any other
content:

```
You are a Worker agent assigned to ticket #<id>.
```

This makes role detection unambiguous regardless of how the worker session is
started.

## History

| When | From | To | By |
|------|------|----|----|
| 2026-03-30T05:10Z | — | new | claude-0330-0245-main |
| 2026-03-30T05:10Z | new | in_design | claude-0330-0245-main |
| 2026-03-30T05:11Z | in_design | specd | claude-0330-0245-main |
| 2026-03-30T05:18Z | specd | ready | apm |
| 2026-03-30T05:52Z | ready | in_progress | claude-0330-0245-main |
| 2026-03-30T05:55Z | in_progress | implemented | claude-0329-1200-b7k2 |
| 2026-03-30T14:26Z | implemented | accepted | apm |
| 2026-03-30T18:07Z | accepted | closed | apm-sync |