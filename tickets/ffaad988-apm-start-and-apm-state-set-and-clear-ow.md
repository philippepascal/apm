+++
id = "ffaad988"
title = "apm start and apm state: set and clear owner on transitions"
state = "in_design"
priority = 0
effort = 2
risk = 2
author = "apm"
branch = "ticket/ffaad988-apm-start-and-apm-state-set-and-clear-ow"
created_at = "2026-04-04T06:28:06.049762Z"
updated_at = "2026-04-04T07:39:02.564915Z"
epic = "8db73240"
target_branch = "epic/8db73240-user-mgmt"
depends_on = ["42f4b3ba"]
+++

## Spec

### Problem

Once an `owner` field exists on tickets, it needs to be set at the right moment. Today `apm start` writes the agent name to the History section but nothing in frontmatter. The owner should persist for the entire ticket lifecycle — once someone owns a ticket, they own it through design, implementation, review, and completion. Ownership is only transferred by explicit supervisor action (`apm assign`), never cleared automatically on state transitions. `apm start` and `apm state in_design` should set the owner when claiming an unowned ticket. If the ticket already has an owner, these commands should still work (the same person resuming work) but not silently overwrite a different owner — that requires `apm assign`.

### Acceptance criteria

- [ ] `apm start <id>` sets `owner` in frontmatter when the ticket has no current owner
- [ ] `apm start <id>` sets `owner` in frontmatter when the ticket's existing owner matches the running agent (same person resuming)
- [ ] `apm start <id>` does NOT overwrite `owner` when the ticket's existing owner is a different value; the state transition still succeeds and a warning is printed to stderr
- [ ] `apm state <id> in_design` sets `owner` in frontmatter when the ticket has no current owner
- [ ] `apm state <id> in_design` sets `owner` in frontmatter when the ticket's existing owner matches the running agent
- [ ] `apm state <id> in_design` does NOT overwrite `owner` when the ticket's existing owner is a different value; the transition still succeeds and a warning is printed to stderr
- [ ] `apm start --spawn <id>`: the PID update commit (which sets owner to the spawned worker's PID) is skipped if the initial ownership guard blocked the owner set

### Out of scope

- Adding an `apm assign` command (ownership transfer by supervisor is not implemented here; use `apm set <id> owner <name>` in the meantime)
- Clearing `owner` automatically on any state transition (ownership is intentionally sticky through the full lifecycle)
- Clearing `owner` when a ticket reaches a terminal state (closed, cancelled)
- Enforcing that only one agent can hold a ticket at a time at the data-model level
- Backfilling `owner` on existing tickets in git history
- Any changes to `apm take` (it already unconditionally overwrites owner — takeover semantics are intentional)

### Approach

This ticket builds on top of `42f4b3ba` (which adds the `owner` field to `Frontmatter` and makes `apm start` / `apm state in_design` unconditionally set it). The change here is to replace those unconditional assignments with a guarded assignment.

**Helper in `apm-core/src/start.rs`** (or inline at each call site):

```rust
fn owner_can_claim(ticket: &ticket::Ticket, new_owner: &str) -> bool {
    match ticket.frontmatter.owner.as_deref() {
        None => true,
        Some(existing) => existing == new_owner,
    }
}
```

**`apm-core/src/start.rs` — `run()`**

Replace the unconditional `t.frontmatter.owner = Some(agent_name.to_string());` with:

```rust
let claimed = owner_can_claim(t, agent_name);
if claimed {
    t.frontmatter.owner = Some(agent_name.to_string());
} else {
    eprintln!(
        "warning: ticket {} is owned by {}; not overwriting (use `apm set {} owner <name>` to reassign)",
        id, t.frontmatter.owner.as_deref().unwrap_or("unknown"), id
    );
}
```

For the `--spawn` path, pass `claimed` through to the post-spawn PID update block. Skip the PID-update commit when `!claimed`.

**`apm-core/src/state.rs` — `transition()`**

The `in_design` branch that sets owner (added by `42f4b3ba`) becomes:

```rust
if new_state == "in_design" {
    let can_claim = match t.frontmatter.owner.as_deref() {
        None => true,
        Some(existing) => existing == actor.as_str(),
    };
    if can_claim {
        t.frontmatter.owner = Some(actor.clone());
    } else {
        eprintln!(
            "warning: ticket {} is owned by {}; not overwriting",
            id, t.frontmatter.owner.as_deref().unwrap_or("unknown")
        );
    }
}
```

**Tests (inline in each file)**

`start.rs`:
- `start_sets_owner_when_unowned` — owner is None before start, set after
- `start_sets_owner_when_same_owner_resumes` — owner = "alice" before, alice starts again, owner stays "alice"
- `start_does_not_overwrite_different_owner` — owner = "alice", bob starts, owner stays "alice"

`state.rs`:
- `in_design_sets_owner_when_unowned`
- `in_design_does_not_overwrite_different_owner` — owner = "alice", bob transitions to in_design, owner stays "alice"

**Order**

1. Implement `owner_can_claim` helper
2. Patch `start::run` (non-spawn and spawn paths)
3. Patch `state::transition` for `in_design`
4. Add tests
5. `cargo test --workspace` — all pass

**Gotcha**: ticket `42f4b3ba` must be merged first (it adds the `owner` field). On the `epic/8db73240-user-mgmt` branch, both land in order; the implementation of `ffaad988` should branch from there.

### Open questions


### Amendment requests

- [x] Rename `agent` to `owner` throughout: acceptance criteria, approach code snippets, test names, warning messages, and commit messages
- [ ] `fm.agent` → `fm.owner`, `agent_can_claim` → `owner_can_claim`, `apm set <id> agent` → `apm set <id> owner` in the warning message

### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-04-04T06:28Z | — | new | apm |
| 2026-04-04T06:35Z | new | groomed | apm |
| 2026-04-04T06:46Z | groomed | in_design | philippepascal |
| 2026-04-04T06:50Z | in_design | specd | claude-0403-0700-b2f1 |
| 2026-04-04T07:14Z | specd | ammend | apm |
| 2026-04-04T07:39Z | ammend | in_design | philippepascal |