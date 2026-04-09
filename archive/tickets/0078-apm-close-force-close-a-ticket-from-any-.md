+++
id = 78
title = "apm close: force-close a ticket from any state"
state = "closed"
priority = 0
effort = 3
risk = 2
author = "claude-0329-1430-main"
agent = "claude-0329-1430-main"
branch = "ticket/0078-apm-close-force-close-a-ticket-from-any-"
created_at = "2026-03-30T01:01:02.690350Z"
updated_at = "2026-03-30T02:02:46.501095Z"
+++

## Spec

### Problem

The only way to close a ticket today is via `apm sync`, which detects merged branches and transitions `accepted → closed`. There is no way to close a ticket that is stuck in `in_progress`, `blocked`, `question`, `specd`, `ready`, etc. without direct frontmatter editing. Supervisors regularly need to close tickets that are cancelled, superseded, or turned out to be moot, without waiting for a merge.

### Acceptance criteria

- [x] `apm close <id>` transitions a ticket to `closed` from any current state
- [x] An optional `--reason <text>` flag appends the reason to the `## History` entry
- [x] `closed` is treated as a mandatory terminal state: it is always a valid target regardless of what transitions are defined in `apm.toml` (analogous to `new` being a mandatory initial state)
- [x] The `apm validate` command recognises `closed` as a built-in state and does not flag it as unknown even if it is absent from `[[workflow.states]]`
- [x] `apm close` is not triggerable via `command:start` — it is a supervisor-only operation
- [x] `apm close` writes `closed` to the ticket branch, pushes it, then merges it into the default branch — so the branch is a true git ancestor of main and `apm clean` can detect and remove it
- [x] `apm sync`'s `batch_close` function is removed; `apm sync` calls the same close logic for each candidate instead
- [x] After `apm sync` closes tickets, their branches are merged into main and `apm clean` removes their worktrees correctly
- [x] `cargo test --workspace` passes

### Out of scope

- Bulk close (`apm close --all --state blocked`)
- Undo / reopen (a separate ticket if needed)

### Approach

**`closed` as a mandatory state**

In `apm-core/src/config.rs`, add `closed` alongside `new` to the set of built-in reserved state names that are always valid. The `apm validate` cross-check should skip `closed` when scanning for unknown state names in ticket frontmatter.

**New `apm close` command**

Add `apm/src/cmd/close.rs`. The command:

1. Loads the ticket from the branch
2. Sets `state = "closed"` in frontmatter
3. Appends a history row: `| <timestamp> | <prev_state> | closed | <agent_name> |` and optionally appends `(reason: <text>)` if `--reason` was provided
4. Commits the change to the ticket branch and pushes
5. Merges the ticket branch into the default branch (fast-forward if possible, merge commit otherwise) — this is what makes `apm clean` work correctly

Wire it into `apm/src/main.rs` as a new `Commands::Close` variant.

**Replace `batch_close` in `apm sync`**

Extract the close logic from `close.rs` into a reusable function in `apm-core` (e.g. `ticket::close`). Remove `batch_close` from `sync.rs` and replace the call site with a loop over candidates that calls `ticket::close` for each one. The interactive prompt (`prompt_close`) stays in `sync.rs`.

**`apm validate` change**

Skip `closed` in the "unknown state" check in `validate.rs`.

## History

| When | From | To | By |
|------|------|----|----|
| 2026-03-30T01:01Z | — | new | claude-0329-1430-main |
| 2026-03-30T01:01Z | new | in_design | claude-0329-1430-main |
| 2026-03-30T01:03Z | in_design | specd | claude-0329-1430-main |
| 2026-03-30T01:05Z | specd | ready | claude-0329-1430-main |
| 2026-03-30T01:05Z | ready | in_progress | claude-0329-1430-main |
| 2026-03-30T01:52Z | in_progress | implemented | apm |
| 2026-03-30T01:55Z | implemented | accepted | apm |
| 2026-03-30T02:02Z | accepted | closed | apm-sync |