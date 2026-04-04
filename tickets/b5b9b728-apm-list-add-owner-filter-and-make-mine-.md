+++
id = "b5b9b728"
title = "apm list: add --owner filter and make --mine match author or owner"
state = "ammend"
priority = 0
effort = 2
risk = 2
author = "apm"
branch = "ticket/b5b9b728-apm-list-add-owner-filter-and-make-mine-"
created_at = "2026-04-04T06:28:11.099983Z"
updated_at = "2026-04-04T07:15:02.961500Z"
epic = "8db73240"
target_branch = "epic/8db73240-user-mgmt"
depends_on = ["42f4b3ba"]
+++

## Spec

### Problem

`apm list --mine` currently matches only the `author` field: it filters to tickets created by the current user. Once ticket 42f4b3ba lands and adds the `agent` ownership field to `Frontmatter`, a user who picks up a ticket created by someone else will not see it in `--mine` even though they are the active owner. The mental model of "my tickets" should include both tickets you created and tickets you are currently responsible for.

There is also no user-facing `--owner` flag to filter by who currently owns a ticket (i.e. by the `agent` field). The existing `--author` flag covers the creator dimension; the owner dimension has no equivalent.

### Acceptance criteria

- [ ] `apm list --mine` returns tickets where `author` equals the current user
- [ ] `apm list --mine` also returns tickets where `agent` equals the current user (even if `author` is someone else)
- [ ] `apm list --mine` does not return tickets where neither `author` nor `agent` matches the current user
- [ ] `apm list --owner alice` returns only tickets whose `agent` field equals `"alice"`
- [ ] `apm list --owner alice` does not return tickets authored by alice but not owned by alice
- [ ] `apm list --owner alice` with no matching tickets returns empty output and exits 0
- [ ] `--owner` and `--mine` are mutually exclusive (combining them produces an error)
- [ ] `--owner alice` and `--author bob` can be combined; both filters apply (AND logic)
- [ ] `--owner alice` and `--state ready` can be combined; both filters apply
- [ ] `apm list --help` documents the `--owner` flag

### Out of scope

- Adding the `agent` field to `Frontmatter` — handled by ticket 42f4b3ba (this ticket depends on it)
- `apm set <id> agent <name>` setter — handled by ticket 42f4b3ba
- Server API (`GET /api/tickets?owner=...`) filter — separate ticket
- UI agent/owner filter dropdown — separate ticket
- Clearing `agent` automatically on state transitions (e.g. when a ticket goes back to `specd`) — intentionally left to a future ticket
- Back-filling `agent` on existing tickets — no migration pass needed

### Approach

**Depends on ticket 42f4b3ba landing first.** That ticket adds `Frontmatter.agent: Option<String>` and the basic `--agent` filter. This ticket builds on top.

---

**1. `apm-core/src/ticket.rs` — `list_filtered` signature**

Add two new parameters after `author_filter`:

```rust
pub fn list_filtered<'a>(
    tickets: &'a [Ticket],
    config: &crate::config::Config,
    state_filter: Option<&str>,
    unassigned: bool,
    all: bool,
    supervisor_filter: Option<&str>,
    actionable_filter: Option<&str>,
    author_filter: Option<&str>,
    owner_filter: Option<&str>,   // new: filters by fm.agent (AND semantics)
    mine_user: Option<&str>,      // new: OR-matches fm.author or fm.agent
) -> Vec<&'a Ticket>
```

Inside the filter predicate, add:

```rust
let owner_ok = owner_filter.map_or(true, |o| fm.agent.as_deref() == Some(o));
let mine_ok  = mine_user.map_or(true, |me| {
    fm.author.as_deref() == Some(me) || fm.agent.as_deref() == Some(me)
});
```

Include `owner_ok && mine_ok` in the final `&&` chain. Keep `author_ok` (from `author_filter`) unchanged.

Update every existing call site that previously passed `author_filter` to also pass `None, None` for the two new parameters.

**2. `apm/src/cmd/list.rs` — handle `--mine` and `--owner`**

Change the current logic that sets `author_filter` for `--mine`:

```rust
// Before:
let author_filter: Option<String> = if mine {
    Some(identity::resolve_current_user(root))
} else {
    author
};

// After:
let mine_user: Option<String> = if mine {
    Some(identity::resolve_current_user(root))
} else {
    None
};
let author_filter = if mine { None } else { author };
// owner is passed through directly from the --owner flag
```

Update the `list_filtered` call to pass `owner.as_deref()` and `mine_user.as_deref()`.

**3. `apm/src/main.rs` — `List` subcommand**

Add the `--owner` flag, conflicting with `--mine`:

```rust
/// Show only tickets owned by USERNAME (agent field)
#[arg(long, value_name = "USERNAME", conflicts_with = "mine")]
owner: Option<String>,
```

Pass `owner` through to `cmd::list::run`, updating its signature to accept `owner: Option<String>`.

Update the `Command::List` match arm in `main()` accordingly.

**4. Tests in `apm-core/src/ticket.rs`**

Add unit tests:
- `list_filtered_by_owner`: tickets with matching `agent`, verify only those are returned
- `list_filtered_mine_matches_author`: `mine_user` matches via `author`, `agent` differs
- `list_filtered_mine_matches_agent`: `mine_user` matches via `agent`, `author` differs
- `list_filtered_mine_or_semantics`: single call returns tickets matching either field

Use the existing `make_ticket` helper; extend it or create a `make_ticket_with_agent` variant (similar to the existing pattern at line ~1249) that sets `agent` in the raw TOML.

**Order**

1. `ticket.rs`: extend `list_filtered` + tests
2. `cmd/list.rs`: update logic
3. `main.rs`: add `--owner` flag
4. `cargo test --workspace` passes

### Open questions


### Amendment requests

- [ ] Rename `agent` to `owner` throughout: the `list_filtered` parameter is `owner_filter` (not `agent_filter`), the field reference is `fm.owner` (not `fm.agent`), and the CLI flag is `--owner` (already correct in title)
- [ ] Update approach code snippets: `fm.agent.as_deref()` → `fm.owner.as_deref()`, parameter names in `list_filtered` signature

### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-04-04T06:28Z | — | new | apm |
| 2026-04-04T06:35Z | new | groomed | apm |
| 2026-04-04T06:51Z | groomed | in_design | philippepascal |
| 2026-04-04T06:54Z | in_design | specd | claude-0404-0651-s7w2 |
| 2026-04-04T07:15Z | specd | ammend | apm |
