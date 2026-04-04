+++
id = "ffaad988"
title = "apm start and apm state: set and clear owner on transitions"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "apm"
branch = "ticket/ffaad988-apm-start-and-apm-state-set-and-clear-ow"
created_at = "2026-04-04T06:28:06.049762Z"
updated_at = "2026-04-04T06:46:10.393369Z"
epic = "8db73240"
target_branch = "epic/8db73240-user-mgmt"
depends_on = ["42f4b3ba"]
+++

## Spec

### Problem

Once an `owner` field exists on tickets, it needs to be set at the right moment. Today `apm start` writes the agent name to the History section but nothing in frontmatter. The owner should persist for the entire ticket lifecycle — once someone owns a ticket, they own it through design, implementation, review, and completion. Ownership is only transferred by explicit supervisor action (`apm assign`), never cleared automatically on state transitions. `apm start` and `apm state in_design` should set the owner when claiming an unowned ticket. If the ticket already has an owner, these commands should still work (the same person resuming work) but not silently overwrite a different owner — that requires `apm assign`.

### Acceptance criteria

- [ ] `apm start <id>` sets `agent` in frontmatter when the ticket has no current agent
- [ ] `apm start <id>` sets `agent` in frontmatter when the ticket's existing agent matches the running agent (same person resuming)
- [ ] `apm start <id>` does NOT overwrite `agent` when the ticket's existing agent is a different value; the state transition still succeeds and a warning is printed to stderr
- [ ] `apm state <id> in_design` sets `agent` in frontmatter when the ticket has no current agent
- [ ] `apm state <id> in_design` sets `agent` in frontmatter when the ticket's existing agent matches the running agent
- [ ] `apm state <id> in_design` does NOT overwrite `agent` when the ticket's existing agent is a different value; the transition still succeeds and a warning is printed to stderr
- [ ] `apm start --spawn <id>`: the PID update commit (which sets agent to the spawned worker's PID) is skipped if the initial ownership guard blocked the agent set

### Out of scope

- Adding an `apm assign` command (ownership transfer by supervisor is not implemented here; use `apm set <id> agent <name>` in the meantime)
- Clearing `agent` automatically on any state transition (ownership is intentionally sticky through the full lifecycle)
- Clearing `agent` when a ticket reaches a terminal state (closed, cancelled)
- Enforcing that only one agent can hold a ticket at a time at the data-model level
- Backfilling `agent` on existing tickets in git history
- Any changes to `apm take` (it already unconditionally overwrites agent — takeover semantics are intentional)

### Approach

This ticket builds on top of `42f4b3ba` (which adds the `agent` field to `Frontmatter` and makes `apm start` / `apm state in_design` unconditionally set it). The change here is to replace those unconditional assignments with a guarded assignment.

**Helper in `apm-core/src/start.rs`** (or inline at each call site):

```rust
fn agent_can_claim(ticket: &ticket::Ticket, new_agent: &str) -> bool {
    match ticket.frontmatter.agent.as_deref() {
        None => true,
        Some(existing) => existing == new_agent,
    }
}
```

**`apm-core/src/start.rs` — `run()`**

Replace the unconditional `t.frontmatter.agent = Some(agent_name.to_string());` with:

```rust
let claimed = agent_can_claim(t, agent_name);
if claimed {
    t.frontmatter.agent = Some(agent_name.to_string());
} else {
    eprintln!(
        "warning: ticket {} is owned by {}; not overwriting (use `apm set {} agent <name>` to reassign)",
        id, t.frontmatter.agent.as_deref().unwrap_or("unknown"), id
    );
}
```

For the `--spawn` path, pass `claimed` through to the post-spawn PID update block. Skip the PID-update commit when `!claimed`.

**`apm-core/src/state.rs` — `transition()`**

The `in_design` branch that sets agent (added by `42f4b3ba`) becomes:

```rust
if new_state == "in_design" {
    let can_claim = match t.frontmatter.agent.as_deref() {
        None => true,
        Some(existing) => existing == actor.as_str(),
    };
    if can_claim {
        t.frontmatter.agent = Some(actor.clone());
    } else {
        eprintln!(
            "warning: ticket {} is owned by {}; not overwriting",
            id, t.frontmatter.agent.as_deref().unwrap_or("unknown")
        );
    }
}
```

**Tests (inline in each file)**

`start.rs`:
- `start_sets_agent_when_unowned` — agent is None before start, set after
- `start_sets_agent_when_same_agent_resumes` — agent = "alice" before, alice starts again, agent stays "alice"
- `start_does_not_overwrite_different_owner` — agent = "alice", bob starts, agent stays "alice"

`state.rs`:
- `in_design_sets_agent_when_unowned`
- `in_design_does_not_overwrite_different_owner` — agent = "alice", bob transitions to in_design, agent stays "alice"

**Order**

1. Implement `agent_can_claim` helper
2. Patch `start::run` (non-spawn and spawn paths)
3. Patch `state::transition` for `in_design`
4. Add tests
5. `cargo test --workspace` — all pass

**Gotcha**: ticket `42f4b3ba` must be merged first (it adds the `agent` field). On the `epic/8db73240-user-mgmt` branch, both land in order; the implementation of `ffaad988` should branch from there.

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-04-04T06:28Z | — | new | apm |
| 2026-04-04T06:35Z | new | groomed | apm |
| 2026-04-04T06:46Z | groomed | in_design | philippepascal |