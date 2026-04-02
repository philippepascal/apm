+++
id = "a63fc005"
title = "apm set: add depends_on field support"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "apm"
agent = "37037"
branch = "ticket/a63fc005-apm-set-add-depends-on-field-support"
created_at = "2026-04-02T20:58:58.236289Z"
updated_at = "2026-04-02T22:47:47.130236Z"
+++

## Spec

### Problem

The `apm set` command lets callers update scalar ticket fields (priority, effort, risk, title, agent, supervisor, branch) from the CLI. The `depends_on` field is already modelled in `Frontmatter` as `Option<Vec<String>>` and is fully wired into dependency-gate logic in `pick_next`, effective-priority boosting, and TOML serialization — but `set_field` in `apm-core/src/ticket.rs` does not handle it, so `apm set <id> depends_on ...` is rejected with "unknown field".

Without CLI access to `depends_on`, callers must hand-edit ticket branch files to link tickets, which is error-prone and bypasses the normal update path (timestamp, branch commit). Adding the field to `set_field` closes this gap with a minimal, self-contained change.

### Acceptance criteria

- [ ] `apm set <id> depends_on <id1>` sets depends_on to a single-element list and the ticket branch TOML contains `depends_on = ["<id1>"]`
- [ ] `apm set <id> depends_on "<id1>,<id2>"` sets depends_on to a two-element list and both IDs appear in the serialized TOML
- [ ] `apm set <id> depends_on -` clears depends_on so the field is absent from the serialized TOML
- [ ] `apm set <id> depends_on <id1>` updates the `updated_at` timestamp on the ticket
- [ ] Whitespace around comma-separated IDs is trimmed: `" id1 , id2 "` yields `["id1", "id2"]`
- [ ] After `apm set <id> depends_on <blocker-id>`, `apm next` does not return the dependent ticket while the blocker's state does not satisfy the required dep gate

### Out of scope

- Validating that listed IDs correspond to existing tickets (unknown IDs are already silently ignored by pick_next; that behaviour is unchanged)
- Adding depends_on support to `apm new` (already implemented via the --depends-on flag)
- Changing dependency-gate resolution logic in pick_next or effective_priority
- A `depends_on append` or `depends_on remove` sub-command for incremental edits (set replaces the whole list)
- UI or server changes

### Approach

How the implementation will work.

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-04-02T20:58Z | — | new | apm |
| 2026-04-02T22:47Z | new | groomed | apm |
| 2026-04-02T22:47Z | groomed | in_design | philippepascal |