+++
id = "b52fc7f4"
title = "Bulk owner change for epic tickets"
state = "in_progress"
priority = 0
effort = 4
risk = 3
author = "philippepascal"
branch = "ticket/b52fc7f4-bulk-owner-change-for-epic-tickets"
created_at = "2026-04-08T15:10:08.148508Z"
updated_at = "2026-04-08T22:49:18.050176Z"
epic = "18dab82d"
target_branch = "epic/18dab82d-ticket-ownership-model"
depends_on = ["b0708201"]
+++

## Spec

### Problem

When a supervisor wants to hand off an entire epic to another supervisor, they must change the owner on each ticket individually. This is tedious for epics with many tickets. A convenience command should change the owner of all non-closed tickets in an epic at once.

### Acceptance criteria

- [ ] `apm epic set <epic-id> owner <user>` changes owner on all non-closed tickets in the epic
- [ ] Closed (terminal) tickets are skipped
- [ ] Output lists each ticket that was changed and any that were skipped
- [ ] The ownership check applies: current user must be the owner of each ticket being changed
- [ ] If any ticket fails the ownership check, none are changed (atomic: all or nothing)
- [ ] Owner validation (collaborator check) applies to the new owner
- [ ] Tests cover: bulk change succeeds, closed tickets skipped, non-owner blocked

### Out of scope

Bulk owner change across multiple epics. Changing owner of the epic itself (epics do not have owners).

### Approach

Extend `run_set()` in `apm/src/cmd/epic.rs` to accept `"owner"` as a valid field alongside the existing `"max_workers"`.

**1. Extend the field guard in `run_set()`**

Change the bail at the top of `run_set()` from only allowing `"max_workers"` to also allow `"owner"`. Add a separate branch for `field == "owner"` that runs the bulk-change logic below; keep the existing `max_workers` branch unchanged.

**2. Load and filter tickets**

Use the same pattern as `run_close()` in the same file:
- `let tickets = apm_core::ticket::load_all_from_git(root, &config.tickets.dir)?;`
- Filter to tickets whose `frontmatter.epic` matches `epic_id`.
- Obtain terminal states: `let terminal = config.terminal_state_ids();` (method on `Config`, always includes `"closed"`).
- Partition into `to_change` (non-terminal state) and `skipped` (terminal state) vecs.

**3. Pre-flight checks (abort before any writes if any fail)**

Iterate `to_change` without modifying anything:
- For each ticket call `apm_core::ticket::check_owner(root, ticket)?` — this helper is added by ticket b0708201 and lives in `apm-core/src/ticket.rs`. It bails if the current user (via `resolve_identity()`) is not the ticket's owner.
- Validate the new owner against collaborators: call `apm_core::config::resolve_collaborators(&config, &local_config)` to obtain the list. If `value != "-"` and the value is not in the collaborators list, bail with `"unknown collaborator: {value}"`. Perform inline if no centralised helper exists.
- If any check fails for any ticket, return the error immediately — no tickets are modified.

**4. Apply changes**

Only reached when all pre-flight checks pass. For each ticket in `to_change`:
- `apm_core::ticket::set_field(&mut t.frontmatter, "owner", value)?`
- `let content = t.serialize()?;`
- `apm_core::git::commit_to_branch(root, &t.frontmatter.branch, &t.path_in_repo, &content, &format!("ticket({}): bulk set owner = {}", t.frontmatter.id, value))?`

**5. Output**

After all updates print:
- One line per changed ticket: `changed  <id>  <title>`
- One line per skipped ticket: `skipped  <id>  <title>  (state: <state>)`
- A summary line: `N ticket(s) changed, M skipped.`

**6. Tests** — add to `apm/tests/integration.rs` following existing owner test helpers (`write_ticket_with_owner`):
- `epic_bulk_owner_change_succeeds`: 2 non-terminal tickets owned by current user; assert both updated and output lists them as changed.
- `epic_bulk_owner_change_skips_closed`: include a closed ticket; assert it appears in skipped output and its owner is unchanged.
- `epic_bulk_owner_change_blocked_non_owner`: one ticket has a different owner; assert command fails and no ticket (including the validly-owned ones) is modified.

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-04-08T15:10Z | — | new | philippepascal |
| 2026-04-08T15:33Z | new | groomed | apm |
| 2026-04-08T16:12Z | groomed | in_design | philippepascal |
| 2026-04-08T16:16Z | in_design | specd | claude-0408-1612-17b8 |
| 2026-04-08T21:47Z | specd | ready | apm |
| 2026-04-08T22:49Z | ready | in_progress | philippepascal |
