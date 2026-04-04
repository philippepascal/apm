+++
id = "01dbdaad"
title = "Replace apm take with apm assign: set owner on any ticket"
state = "in_progress"
priority = 0
effort = 3
risk = 2
author = "apm"
branch = "ticket/01dbdaad-replace-apm-take-with-apm-assign-set-own"
created_at = "2026-04-04T06:33:40.535848Z"
updated_at = "2026-04-04T17:47:15.421950Z"
epic = "8db73240"
target_branch = "epic/8db73240-user-mgmt"
depends_on = ["ffaad988"]
+++

## Spec

### Problem

`apm take` exists today as a "takeover" command — it writes a handoff entry to History and provisions a worktree, but it does not track the old or new owner in frontmatter (always logs "unknown"). It also only works for the current agent taking over, not for a supervisor assigning someone else.

With the `owner` field, `apm take` becomes redundant and underspecified. What is needed instead is `apm assign <id> <username>` — a supervisor action that sets the `owner` field on any ticket regardless of state. This replaces both the self-takeover use case (I want to own this ticket) and the delegation use case (supervisor assigns a ticket to someone). The old `apm take` command, its CLI entry, server endpoint (`/api/tickets/:id/take`), and `handoff()` function should be removed.

### Acceptance criteria

- [x] `apm assign <id> <username>` sets the `owner` field in frontmatter to `<username>` for any ticket, regardless of its current state
- [x] `apm assign <id> -` clears the `owner` field (sets it to absent in frontmatter)
- [x] `apm assign <id> <username>` commits the change to the ticket's branch with message `ticket(<id>): assign owner = <username>`
- [x] `apm assign <id> <username>` prints `<id>: owner = <username>` to stdout on success
- [x] `apm assign` with a nonexistent or ambiguous ticket ID exits non-zero and prints an error
- [x] `apm take` is no longer a recognised CLI subcommand
- [ ] `POST /api/tickets/:id/take` returns 404 or 405 (the route no longer exists)
- [ ] `pub fn handoff` is removed from `apm-core` (it no longer compiles if referenced)

### Out of scope

- Provisioning a worktree (`apm worktrees --add <id>` remains the right tool for that)
- State transitions — `apm assign` only sets the `owner` field, never changes state
- A server-side `assign` endpoint — the CLI is sufficient; no REST route is added
- Clearing `owner` automatically on state transitions (covered by ticket ffaad988)
- Enforcing single-ownership at the data-model level
- Backfilling `owner` on existing tickets in git history
- Any changes to how `apm start` or `apm state in_design` handle ownership guards (ticket ffaad988)

### Approach

This ticket depends on `ffaad988`, which adds the `owner` field to `Frontmatter`. All changes branch from `epic/8db73240-user-mgmt` after that ticket is merged.

**1. `apm-core/src/ticket.rs` — extend `set_field` and remove `handoff`**

Add `owner` as a settable field in `set_field()`, following the same pattern as `supervisor` (use `"-"` to clear):

```rust
"owner" => fm.owner = if value == "-" { None } else { Some(value.to_string()) },
```

Delete `pub fn handoff()` and its two inline unit tests (`handoff_no_agent_uses_unknown_placeholder` and `handoff_successful`).

**2. `apm/src/cmd/assign.rs` — new file**

Identical structure to `set.rs`: load tickets, resolve id, call `ticket::set_field(&mut t.frontmatter, "owner", &username)`, update `updated_at`, serialize, commit to the ticket branch with message `ticket(<id>): assign owner = <username>`, push if aggressive. Print `<id>: owner = <username>` on success (or `<id>: owner cleared` when `-` is passed).

**3. `apm/src/main.rs` — register `Assign`, remove `Take`**

Add an `Assign` variant with `id: String`, `username: String`, `no_aggressive: bool`. Remove the `Take { id, no_aggressive }` variant and its dispatch arm. Delete `apm/src/cmd/take.rs` entirely. Update `apm/src/cmd/mod.rs` to remove `pub mod take;` and add `pub mod assign;`.

**4. `apm-server/src/main.rs` — remove take endpoint**

Delete the `take_ticket` async function and the route `.route("/api/tickets/:id/take", post(take_ticket))`.

**5. `apm/tests/integration.rs` — replace take tests with assign tests**

Delete the four `take_*` test functions. Add:

- `assign_sets_owner_field`: create a ticket in any state, run `cmd::assign::run(p, "1", "alice", true)`, read back the branch content, assert `owner = "alice"` appears in frontmatter.
- `assign_clears_owner_field`: start with `owner = "alice"`, run `cmd::assign::run(p, "1", "-", true)`, assert `owner` key is absent from frontmatter.
- `assign_unknown_id_errors`: run `cmd::assign::run(p, "9999", "alice", true)`, assert it returns `Err`.

**Order**

1. Extend `set_field` in `apm-core/src/ticket.rs`; delete `handoff` and its tests
2. Add `apm/src/cmd/assign.rs`; update `mod.rs`
3. Patch `apm/src/main.rs`: add `Assign`, remove `Take`
4. Delete `apm/src/cmd/take.rs`
5. Patch `apm-server/src/main.rs`: remove `take_ticket` and its route
6. Replace take integration tests with assign tests
7. `cargo test --workspace`

**Gotcha**: the `owner` field must exist on `Frontmatter` before step 1 compiles. This ticket must be implemented on top of the `epic/8db73240-user-mgmt` branch after `ffaad988` is merged.

### Open questions


### Amendment requests

- [x] Rename `agent` to `owner` throughout: `apm assign` sets the `owner` field (not `agent`), commit message says `assign owner = <username>`, stdout says `<id>: owner = <username>`, `set_field` arm is `"owner"`, acceptance criteria reference `owner` field
- [x] The `handoff()` removal and `apm take` deletion are unchanged

### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-04-04T06:33Z | — | new | apm |
| 2026-04-04T06:35Z | new | groomed | apm |
| 2026-04-04T07:06Z | groomed | in_design | philippepascal |
| 2026-04-04T07:09Z | in_design | specd | claude-0404-0710-b7f2 |
| 2026-04-04T07:15Z | specd | ammend | apm |
| 2026-04-04T07:43Z | ammend | in_design | philippepascal |
| 2026-04-04T07:45Z | in_design | specd | claude-0404-0800-c9f1 |
| 2026-04-04T15:34Z | specd | ready | apm |
| 2026-04-04T17:47Z | ready | in_progress | philippepascal |