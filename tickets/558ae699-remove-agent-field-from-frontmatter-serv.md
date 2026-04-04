+++
id = "558ae699"
title = "Remove agent field from frontmatter, server API, and apm list/show output"
state = "closed"
priority = 0
effort = 3
risk = 2
author = "apm"
branch = "ticket/558ae699-remove-agent-field-from-frontmatter-serv"
created_at = "2026-04-02T20:53:58.923882Z"
updated_at = "2026-04-04T06:01:06.062580Z"
epic = "8db73240"
target_branch = "epic/8db73240-user-mgmt"
depends_on = ["610be42e"]
+++

## Spec

### Problem

Ticket frontmatter contains an `agent` field that records the name of the worker process currently handling the ticket. Workers are single-use processes that die when a ticket reaches `implemented`; the agent name has no durable value after that point. Despite this, the field propagates into three places:

1. **Frontmatter struct** — `Frontmatter.agent: Option<String>` is read and written via TOML. Dependent ticket 610be42e stops writing the field and changes CLI output; this ticket completes the cleanup by removing the struct field entirely and eliminating all server-side explicit references.
2. **Server API responses** — `WorkerInfo` in `apm-server/src/workers.rs` contains an explicit `agent: String` field populated from frontmatter and included in `GET /api/workers` JSON. The UI worker panel receives this but the field has no actionable meaning once a worker is not live.
3. **`take_ticket` handler** — `POST /api/tickets/:id/take` in `apm-server/src/main.rs` calls `handoff()` and has a fallback that writes `ticket.frontmatter.agent = Some(agent_name)` when no agent is set. After 610be42e updates `handoff()` to not require a prior agent value, this fallback becomes unreachable dead code; it still compiles and writes a stale value if ever hit.

The fix removes `agent` from the `Frontmatter` struct, the server worker response, and the `take_ticket` fallback — completing the cleanup started in 610be42e. Existing ticket files that still have `agent = "..."` in their TOML are unaffected: the `toml` crate silently ignores unknown fields when `deny_unknown_fields` is not set.

### Acceptance criteria

- [x] `GET /api/workers` JSON response objects do not contain an `agent` key
- [x] `GET /api/tickets/:id` JSON response objects do not contain an `agent` key
- [x] Existing ticket files containing `agent = "..."` in their TOML frontmatter are parsed without error
- [x] `POST /api/tickets/:id/take` completes without error on tickets that have no `agent` field in frontmatter
- [x] `cargo test --workspace` passes after the `agent` field is removed from the `Frontmatter` struct

### Out of scope

- Removing `agent` from `apm list` / `apm show` CLI output — handled by ticket 610be42e
- Stopping new writes of `agent` in `apm start` / `apm new` — handled by ticket 610be42e
- Updating `handoff()` to not require a prior agent value — handled by ticket 610be42e
- Changing `--unassigned` to filter by `author == "unassigned"` — handled by ticket 610be42e
- Identity resolution (`.apm/local.toml`, `apm init` prompts) — separate tickets
- UI board author filter and `/api/me` endpoint — separate tickets
- Rewriting existing ticket files to remove the `agent` key — no migration pass needed

### Approach

This ticket depends on 610be42e landing first. By that point `agent` is no longer written to frontmatter and is absent from CLI output; the changes below complete the cleanup.

**1. `apm-core/src/ticket.rs` — remove `agent` field from `Frontmatter`**
- Delete the `pub agent: Option<String>` field and its `#[serde(...)]` attribute entirely.
- The `toml` crate silently drops unknown keys on parse, so existing ticket files containing `agent = "..."` continue to load without error.
- The `handoff()` function was already updated by 610be42e to not reference `agent`; no further change needed there.

**2. Fix all `Frontmatter` struct literals that break at compile time**
- `apm-server/src/queue.rs` — `fake_ticket()` helper: remove the `agent: None` line.
- `apm-server/src/main.rs` — in-memory test helper (`make_ticket()` or similar): remove the `agent: None` line.
- Any other in-test struct literals found by the compiler; fix them the same way.

**3. `apm-server/src/workers.rs` — remove `agent` from `WorkerInfo`**
- Delete `agent: String` from the `WorkerInfo` struct.
- In `collect_workers()`, change the tuple destructure (line ~80) from `(ticket_title, branch, state, agent)` to `(ticket_title, branch, state)`, removing the `t.frontmatter.agent.clone().unwrap_or_default()` arm.
- Remove `agent` from the `results.push(WorkerInfo { ... })` call.

**4. `apm-server/src/main.rs` — clean up `take_ticket` handler**
- Remove the fallback branch (`Err(e) if e.to_string().contains("no agent assigned")`) that writes `ticket.frontmatter.agent = Some(agent_name.clone())`. After 610be42e, `handoff()` succeeds unconditionally on tickets with no agent; this branch is dead code.
- Update the commit message from `"ticket({id}): reassign agent to {agent_name}"` to `"ticket({id}): take ticket"`.
- Remove the now-unused `agent_name_clone` binding (was only referenced in the commit message format string).

**Order of operations**
1. Remove struct field and fix all struct literals (step 1 + 2) — get the code to compile.
2. Apply server-only changes (steps 3 + 4).
3. Run `cargo test --workspace` — all tests must pass before transitioning to `implemented`.

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-04-02T20:53Z | — | new | apm |
| 2026-04-02T23:22Z | new | groomed | apm |
| 2026-04-02T23:33Z | groomed | in_design | philippepascal |
| 2026-04-02T23:39Z | in_design | specd | claude-0402-2340-b7f2 |
| 2026-04-04T00:29Z | specd | ready | apm |
| 2026-04-04T02:32Z | ready | in_progress | philippepascal |
| 2026-04-04T02:40Z | in_progress | implemented | claude-0403-1200-x7k2 |
| 2026-04-04T06:01Z | implemented | closed | apm-sync |
