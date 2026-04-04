+++
id = "9931e70f"
title = "Queue: exclude tickets owned by another user"
state = "in_progress"
priority = 0
effort = 3
risk = 2
author = "apm"
branch = "ticket/9931e70f-queue-exclude-tickets-owned-by-another-u"
created_at = "2026-04-04T06:28:25.839773Z"
updated_at = "2026-04-04T17:39:43.673711Z"
epic = "8db73240"
target_branch = "epic/8db73240-user-mgmt"
depends_on = ["ffaad988"]
+++

## Spec

### Problem

The priority queue (`/api/queue` and `apm next`) shows all tickets actionable by an agent, regardless of who owns them. Since owner persists for the entire ticket lifecycle, a `ready` ticket owned by Alice shouldn't appear in Bob's queue — Alice owns it and will pick it back up. The queue should exclude tickets where `owner` is set to someone other than the requesting user. Unowned tickets remain visible to everyone.

### Acceptance criteria

- [x] `apm next` does not return a ticket whose `owner` field is set to a user other than the running agent
- [x] `apm next` returns a ticket whose `owner` field matches the running agent (owner resuming their own work)
- [x] `apm next` returns a ticket with no `owner` field set
- [x] `apm start --next` does not pick a ticket owned by a different user
- [x] `GET /api/queue` excludes tickets whose `owner` differs from the authenticated caller
- [x] `GET /api/queue` includes tickets with no `owner` set
- [x] `GET /api/queue` includes tickets whose `owner` matches the authenticated caller
- [x] When the caller cannot be determined (no session, no localhost identity), `/api/queue` returns all tickets unchanged (no ownership filter applied)

### Out of scope

- Filtering `apm list` by ownership (that filter is `--agent` and is covered by ticket 42f4b3ba)
- Enforcing ownership at write time (tickets can still be started by anyone; the queue filter is advisory, not a lock)
- Adding auth to the `/api/queue` endpoint (authentication is handled by the broader user-mgmt epic)
- Clearing `agent` on any state transition (ownership is sticky by design, per ticket ffaad988)
- Back-filling ownership on existing tickets

### Approach

This ticket depends on ticket 42f4b3ba (which adds `owner: Option<String>` to `Frontmatter`) and ticket ffaad988 (which guards ownership assignment). Those must be merged before this ticket.

**1. `apm-core/src/ticket.rs` — add caller filter to `sorted_actionable` and `pick_next`**

Add `caller: Option<&str>` parameter to both functions.

In `sorted_actionable`, after the state filter, add:
```rust
.filter(|t| {
    match t.frontmatter.owner.as_deref() {
        None => true,
        Some(owner) => caller.map_or(true, |c| c == owner),
    }
})
```
When `caller` is `None`, the predicate is always true (no filtering — preserves current behaviour for callers without an identity).

Pass `caller` through from `pick_next` to `sorted_actionable`.

Update all existing test call sites to pass `None` as the extra argument.

**2. `apm/src/cmd/next.rs`**

Resolve the running agent name before calling `pick_next`:
```rust
let agent_name = apm_core::start::resolve_agent_name();
ticket::pick_next(&tickets, &actionable, &[], pw, ew, rw, &config, Some(&agent_name))
```

**3. `apm-core/src/start.rs` — two call sites**

In `run_next` (line ~335): `resolve_agent_name()` is already called later (line 349); hoist it above the `pick_next` call and pass `Some(&agent_name)`.

In `spawn_next_worker` (line ~469): same pattern — `resolve_agent_name()` is called at line 482; hoist it above `pick_next` and pass `Some(&agent_name)`.

**4. `apm-server/src/queue.rs`**

Add extractors to `queue_handler`:
```rust
pub async fn queue_handler(
    State(state): State<Arc<AppState>>,
    connect_info: Option<ConnectInfo<SocketAddr>>,
    headers: axum::http::HeaderMap,
) -> Result<Json<Vec<QueueEntry>>, AppError>
```

Determine the caller before spawning the blocking task:
```rust
let caller: Option<String> = if is_localhost(connect_info) {
    state.git_root().map(|root| apm_core::config::resolve_identity(root))
} else {
    find_session_username(&headers, &state.session_store)
};
```

Pass into the blocking closure and thread through to `sorted_actionable`:
```rust
let caller_ref = caller.as_deref();
let sorted = apm_core::ticket::sorted_actionable(
    &tickets, &actionable, p.priority_weight, p.effort_weight, p.risk_weight, caller_ref,
);
```

Note: `is_localhost` and `find_session_username` are private to `main.rs`; either move them to a shared module (e.g. `auth.rs`) or inline equivalent logic in `queue.rs`.

**5. Tests** (in `apm-core/src/ticket.rs` and/or `apm/tests/integration.rs`)

- `sorted_actionable_excludes_ticket_owned_by_other`: ticket with `owner = "alice"`, caller = `Some("bob")` → excluded
- `sorted_actionable_includes_ticket_owned_by_caller`: ticket with `owner = "alice"`, caller = `Some("alice")` → included
- `sorted_actionable_includes_unowned_ticket`: ticket with `owner = None`, caller = `Some("bob")` → included
- `sorted_actionable_no_caller_shows_all`: tickets with owners set, caller = `None` → all included

**Order of changes**
1. `ticket.rs`: add caller param + filter + update existing tests
2. `start.rs`: hoist resolve_agent_name, pass to pick_next
3. `next.rs`: pass caller
4. `queue.rs`: add caller extraction + pass to sorted_actionable
5. `cargo test --workspace` must pass

### Open questions


### Amendment requests

- [x] Rename `agent` to `owner` throughout: `fm.agent` → `fm.owner`, `t.frontmatter.agent` → `t.frontmatter.owner` in filter predicates, approach code snippets, and test names
- [x] Acceptance criteria: "ticket whose `agent` field" → "ticket whose `owner` field"

### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-04-04T06:28Z | — | new | apm |
| 2026-04-04T06:35Z | new | groomed | apm |
| 2026-04-04T07:02Z | groomed | in_design | philippepascal |
| 2026-04-04T07:06Z | in_design | specd | claude-0404-0702-b3f2 |
| 2026-04-04T07:15Z | specd | ammend | apm |
| 2026-04-04T07:42Z | ammend | in_design | philippepascal |
| 2026-04-04T07:43Z | in_design | specd | claude-0404-0800-c7d1 |
| 2026-04-04T15:34Z | specd | ready | apm |
| 2026-04-04T17:39Z | ready | in_progress | philippepascal |
