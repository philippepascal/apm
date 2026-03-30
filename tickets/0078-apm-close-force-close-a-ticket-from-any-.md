+++
id = 78
title = "apm close: force-close a ticket from any state"
state = "specd"
priority = 0
effort = 3
risk = 2
author = "claude-0329-1430-main"
branch = "ticket/0078-apm-close-force-close-a-ticket-from-any-"
created_at = "2026-03-30T01:01:02.690350Z"
updated_at = "2026-03-30T01:03:26.532168Z"
+++

## Spec

### Problem

The only way to close a ticket today is via `apm sync`, which detects merged branches and transitions `accepted → closed`. There is no way to close a ticket that is stuck in `in_progress`, `blocked`, `question`, `specd`, `ready`, etc. without direct frontmatter editing. Supervisors regularly need to close tickets that are cancelled, superseded, or turned out to be moot, without waiting for a merge.

### Acceptance criteria

- [ ] `apm close <id>` transitions a ticket to `closed` from any current state
- [ ] An optional `--reason <text>` flag appends the reason to the `## History` entry
- [ ] `closed` is treated as a mandatory terminal state: it is always a valid target regardless of what transitions are defined in `apm.toml` (analogous to `new` being a mandatory initial state)
- [ ] The `apm validate` command recognises `closed` as a built-in state and does not flag it as unknown even if it is absent from `[[workflow.states]]`
- [ ] `apm close` is not triggerable via `command:start` — it is a supervisor-only operation
- [ ] `apm state <id> closed` continues to work for the normal `accepted → closed` path defined in `apm.toml`; `apm close` is the escape hatch for all other states
- [ ] `cargo test --workspace` passes

### Out of scope

- Bulk close (`apm close --all --state blocked`)
- Undo / reopen (a separate ticket if needed)
- Changing how `apm sync` closes accepted tickets

### Approach

**`closed` as a mandatory state**

In `apm-core/src/config.rs`, add `closed` alongside `new` to the set of built-in reserved state names that are always valid. The `apm validate` cross-check should skip `closed` when scanning for unknown state names in ticket frontmatter.

**New `apm close` command**

Add `apm/src/cmd/close.rs`. The command:

1. Loads the ticket from the branch
2. Sets `state = "closed"` in frontmatter
3. Appends a history row: `| <timestamp> | <prev_state> | closed | <agent> |` and optionally `(reason: <text>)` in the By column or as a note row
4. Commits to the ticket branch and pushes

Wire it into `apm/src/main.rs` as a new `Commands::Close` variant.

**`apm validate` change**

Skip `closed` in the "unknown state" check in `validate.rs`.

## History

| When | From | To | By |
|------|------|----|----|
| 2026-03-30T01:01Z | — | new | claude-0329-1430-main |
| 2026-03-30T01:01Z | new | in_design | claude-0329-1430-main |
| 2026-03-30T01:03Z | in_design | specd | claude-0329-1430-main |
