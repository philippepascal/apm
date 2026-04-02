+++
id = "a63fc005"
title = "apm set: add depends_on field support"
state = "in_progress"
priority = 0
effort = 2
risk = 1
author = "apm"
agent = "80904"
branch = "ticket/a63fc005-apm-set-add-depends-on-field-support"
created_at = "2026-04-02T20:58:58.236289Z"
updated_at = "2026-04-02T22:55:43.410715Z"
+++

## Spec

### Problem

The `apm set` command lets callers update scalar ticket fields (priority, effort, risk, title, agent, supervisor, branch) from the CLI. The `depends_on` field is already modelled in `Frontmatter` as `Option<Vec<String>>` and is fully wired into dependency-gate logic in `pick_next`, effective-priority boosting, and TOML serialization — but `set_field` in `apm-core/src/ticket.rs` does not handle it, so `apm set <id> depends_on ...` is rejected with "unknown field".

Without CLI access to `depends_on`, callers must hand-edit ticket branch files to link tickets, which is error-prone and bypasses the normal update path (timestamp, branch commit). Adding the field to `set_field` closes this gap with a minimal, self-contained change.

### Acceptance criteria

- [x] `apm set <id> depends_on <id1>` sets depends_on to a single-element list and the ticket branch TOML contains `depends_on = ["<id1>"]`
- [x] `apm set <id> depends_on "<id1>,<id2>"` sets depends_on to a two-element list and both IDs appear in the serialized TOML
- [x] `apm set <id> depends_on -` clears depends_on so the field is absent from the serialized TOML
- [x] `apm set <id> depends_on <id1>` updates the `updated_at` timestamp on the ticket
- [x] Whitespace around comma-separated IDs is trimmed: `" id1 , id2 "` yields `["id1", "id2"]`
- [x] After `apm set <id> depends_on <blocker-id>`, `apm next` does not return the dependent ticket while the blocker's state does not satisfy the required dep gate

### Out of scope

- Validating that listed IDs correspond to existing tickets (unknown IDs are already silently ignored by pick_next; that behaviour is unchanged)
- Adding depends_on support to `apm new` (already implemented via the --depends-on flag)
- Changing dependency-gate resolution logic in pick_next or effective_priority
- A `depends_on append` or `depends_on remove` sub-command for incremental edits (set replaces the whole list)
- UI or server changes

### Approach

**One file to change for the core fix:** apm-core/src/ticket.rs — the set_field function (currently around line 757).

Add a "depends_on" match arm before the catch-all arm:

    "depends_on" => {
        if value == "-" {
            fm.depends_on = None;
        } else {
            let ids: Vec<String> = value
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect();
            fm.depends_on = if ids.is_empty() { None } else { Some(ids) };
        }
    }

This mirrors the pattern used in apm/src/cmd/new.rs (lines 49-60) for parsing the --depends-on flag.

**CLI help text:** In apm/src/main.rs, update the /// Field to update: doc comment on the field argument to include depends_on. Add a usage example showing the comma-separated form and the - clear form.

**Tests to add** in apm/tests/integration.rs (follow the pattern of set_priority_updates_frontmatter):

1. set_depends_on_single_id — set one ID, verify TOML contains depends_on = ["<id>"]
2. set_depends_on_comma_separated — set two comma-separated IDs, verify both in TOML
3. set_depends_on_clear — set then clear with -, verify field absent from TOML
4. set_depends_on_trims_whitespace — value " id1 , id2 " yields ["id1", "id2"]

The existing integration test next_skips_dep_blocked_returns_unblocked already covers the end-to-end scheduling effect; no new scheduling tests are needed.

**Order of steps:**
1. Add the match arm to set_field in apm-core/src/ticket.rs
2. Update CLI help/examples in apm/src/main.rs
3. Add the four integration tests
4. Run cargo test --workspace — all tests must pass

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-04-02T20:58Z | — | new | apm |
| 2026-04-02T22:47Z | new | groomed | apm |
| 2026-04-02T22:47Z | groomed | in_design | philippepascal |
| 2026-04-02T22:51Z | in_design | specd | claude-0402-2300-b7f2 |
| 2026-04-02T22:55Z | specd | ready | apm |
| 2026-04-02T22:55Z | ready | in_progress | philippepascal |