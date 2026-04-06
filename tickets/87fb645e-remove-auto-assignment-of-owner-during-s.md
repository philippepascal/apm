+++
id = "87fb645e"
title = "Remove auto-assignment of owner during state transitions"
state = "specd"
priority = 0
effort = 3
risk = 1
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/87fb645e-remove-auto-assignment-of-owner-during-s"
created_at = "2026-04-06T20:57:32.658671Z"
updated_at = "2026-04-06T21:51:47.372852Z"
+++

## Spec

### Problem

When a ticket transitions to in_design (state.rs:113-125) or when work starts via apm start (start.rs:145-201), the owner field is automatically set to the acting agent or user if currently unset. This conflates two separate concerns: who is working on a ticket right now (the agent) and who is responsible for it (the owner). In practice this means agent workers silently claim ownership of tickets they are only implementing, which confuses the supervisor's view of who owns what. Owner assignment should be a deliberate supervisor action — only changeable via explicit commands like 'apm assign' or 'apm set owner', never as a side-effect of state transitions or starting work.

### Acceptance criteria

- [ ] `apm state <id> in_design` does not set or modify the `owner` field when the ticket is unowned
- [ ] `apm state <id> in_design` does not set or modify the `owner` field when the ticket is already owned by a different actor
- [ ] `apm start <id>` does not set or modify the `owner` field when the ticket is unowned
- [ ] `apm start <id>` does not set or modify the `owner` field when the ticket is already owned by a different actor
- [ ] `apm start <id> --spawn` does not set or modify the `owner` field (spawned worker name is not written to the ticket)
- [ ] `apm assign <id> <username>` continues to set the owner field as before
- [ ] `apm set <id> owner <username>` continues to set the owner field as before

### Out of scope

- Changes to `apm assign` or `apm set owner` behavior — deliberate assignment is unchanged
- Introducing a new field (e.g. `assignee`) to track who is currently working — that is a separate concern
- Any UI or display changes to `apm show` or `apm list`
- The `--aggressive` flag behavior on `apm assign` / `apm set`

### Approach

Three removal sites and one dead-code cleanup, plus test updates.

**`apm-core/src/state.rs` (lines 112-125)**
Remove the entire `if new_state == "in_design" { ... }` block that conditionally sets `t.frontmatter.owner`. Nothing replaces it — the owner field is left as-is.

**`apm-core/src/start.rs` (lines 193-201)**
Remove the block that calls `owner_can_claim()` and sets `t.frontmatter.owner = Some(agent_name.to_string())`. The `claimed` variable can also be removed if it is only used to gate this assignment; if it gates other logic, leave the variable but drop the owner-assignment branch.

**`apm-core/src/start.rs` (lines 317-323, `--spawn` path)**
Remove the block that sets `t.frontmatter.owner = Some(worker_name.clone())` and the git commit tied solely to that ownership change (`"ticket({id}): set owner to spawned worker"`).

**`apm-core/src/start.rs` (lines 145-150)**
Delete the `owner_can_claim()` helper function — it becomes dead code once the two call sites above are removed. Confirm no other callers exist before deleting.

**`apm/tests/integration.rs` (lines 1628-1694)**
Delete or convert the five integration tests that asserted old auto-assignment behaviour:
- `start_sets_owner_when_unowned` — delete
- `start_sets_owner_when_same_owner_resumes` — delete
- `start_does_not_overwrite_different_owner` — repurpose as a test asserting owner is unchanged after `apm start` regardless of existing value
- `in_design_sets_owner_when_unowned` — delete
- `in_design_does_not_overwrite_different_owner` — repurpose as a test asserting owner is unchanged after `apm state <id> in_design`

**`apm-core/src/start.rs` unit tests (lines 626-655)**
Delete the three `owner_can_claim_*` unit tests alongside the helper function.

**Order of steps**
1. Remove the three auto-assignment blocks and the helper function.
2. Verify the codebase compiles (`cargo build`).
3. Update / delete tests as described above.
4. Run `cargo test` — all tests must pass.

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-04-06T20:57Z | — | new | philippepascal |
| 2026-04-06T21:22Z | new | groomed | apm |
| 2026-04-06T21:47Z | groomed | in_design | philippepascal |
| 2026-04-06T21:51Z | in_design | specd | claude-0406-2147-3dc8 |
