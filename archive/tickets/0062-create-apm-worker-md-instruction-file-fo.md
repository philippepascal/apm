+++
id = 62
title = "Create apm.worker.md: instruction file for implementation agents"
state = "closed"
priority = 2
effort = 2
risk = 1
author = "claude-0329-1200-a1b2"
agent = "claude-0329-1430-main"
branch = "ticket/0062-create-apm-worker-md-instruction-file-fo"
created_at = "2026-03-29T19:12:40.149419Z"
updated_at = "2026-03-30T02:02:46.501095Z"
+++

## Spec

### Problem

Implementation agents (those picking up `ready` tickets via `apm start`) receive only the generic `apm.agents.md` guidance. There is no dedicated instruction file covering implementation expectations for this project: testing requirements, PR discipline, commit message format, when to write side tickets, how to handle blocked state, etc.

This leads to inconsistent agent behaviour and requires the supervisor to give repeated corrections that could be pre-empted by richer upfront instructions.

### Acceptance criteria

- [x] A file `apm.worker.md` exists at the repo root with implementation-phase guidance for worker agents
- [x] `apm.toml` references `apm.worker.md` as the `instructions` for the `ready` and `in_progress` states (requires the `instructions` field added in ticket #61)
- [x] The content of `apm.worker.md` covers: reading the spec before coding, minimal-change discipline, commit message format, test requirements, PR creation, when to open side tickets, how to transition to `blocked`, and the `apm.agents.md` shell discipline rules
- [x] `apm-core` config parsing continues to work if `instructions` is absent on any state (already addressed by ticket #61)

### Out of scope

- Runtime enforcement of any instruction in the file
- Changes to `apm.agents.md` (the generic instructions remain unchanged)
- Creating a separate `apm.spec-writer.md` (covered by ticket #61)
- Any new CLI commands or config fields beyond what ticket #61 introduces

### Approach

This ticket depends on ticket #61 for the `instructions` field in `StateConfig` and the `apm.toml` state schema.

1. Add `instructions = "apm.worker.md"` to the `ready` and `in_progress` state entries in `apm.toml`.

2. Write `apm.worker.md` at the repo root with:
   - When this file applies (picking up `ready` tickets, working `in_progress` tickets)
   - Read the full spec before writing any code
   - Minimal-change discipline: implement exactly the acceptance criteria, no extras
   - Commit format: imperative mood, ≤ 72 chars, no co-authored trailer
   - Tests: unit tests in-crate, integration tests in `apm/tests/integration.rs`, run `cargo test --workspace`
   - PR: open against `main`, title mirrors ticket title, body includes `Closes #<n>`, approach summary, test plan
   - Side tickets: use `apm new --side-note` for out-of-scope issues found during implementation
   - Blocked: write questions in `### Open questions`, commit, then `apm state <id> blocked`
   - Shell discipline: copy the key rules from `apm.agents.md` (one command per Bash call, no `&&`, `git -C` for worktrees)

## History

| When | From | To | By |
|------|------|----|----|
| 2026-03-29T19:12Z | — | new | claude-0329-1200-a1b2 |
| 2026-03-29T22:57Z | new | in_design | claude-spec-62 |
| 2026-03-29T23:09Z | in_design | specd | claude-0329-1430-main |
| 2026-03-29T23:19Z | specd | ready | apm |
| 2026-03-29T23:37Z | ready | in_progress | claude-0329-1430-main |
| 2026-03-29T23:39Z | in_progress | implemented | claude-0329-1430-main |
| 2026-03-29T23:55Z | implemented | accepted | apm |
| 2026-03-30T02:02Z | accepted | closed | apm-sync |