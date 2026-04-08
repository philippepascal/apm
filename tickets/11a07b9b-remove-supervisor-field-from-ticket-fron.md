+++
id = "11a07b9b"
title = "Remove supervisor field from ticket frontmatter"
state = "specd"
priority = 0
effort = 2
risk = 2
author = "philippepascal"
branch = "ticket/11a07b9b-remove-supervisor-field-from-ticket-fron"
created_at = "2026-04-08T15:09:32.454090Z"
updated_at = "2026-04-08T15:42:10.761025Z"
epic = "18dab82d"
target_branch = "epic/18dab82d-ticket-ownership-model"
+++

## Spec

### Problem

The ticket frontmatter has a `supervisor` field that is not part of the ownership model. The ownership spec defines only two fields: `author` (immutable, who created) and `owner` (who manages). The `supervisor` field adds confusion and overlaps with `owner`. It must be removed from the Frontmatter struct, all set_field handling, list filters, and any UI references.

### Acceptance criteria

- [ ] `supervisor` field removed from `Frontmatter` struct in `ticket.rs`
- [ ] `apm set <id> supervisor` no longer accepted (returns error)
- [ ] `apm list --supervisor` filter removed
- [ ] Any server/UI references to supervisor field removed
- [ ] Existing tickets with supervisor field still parse without error (field ignored)
- [ ] All tests pass

### Out of scope

- Migrating existing tickets that have supervisor set — they will parse without error (serde ignores unknown fields by default).
- Removing the "supervisor" role concept from the workflow system — workflow states can still declare actionable = ["supervisor"] and the server's supervisor_states API remains unchanged. Only the frontmatter field is being removed.

### Approach

Files to change and what to do in each:

**`apm-core/src/ticket.rs`**
- Remove `pub supervisor: Option<String>` field from `Frontmatter` struct (~line 47). No special serde annotation needed — serde ignores unknown TOML keys by default, so old tickets with `supervisor = "…"` will parse cleanly.
- Remove `supervisor: None` from the struct literal in `create()` (~line 472).
- Remove the `"supervisor"` arm from `set_field()` (~line 768).
- Remove `supervisor_filter: Option<&str>` parameter from `list_filtered()` (~line 723), remove the `supervisor_ok` binding (~line 746), and remove `supervisor_ok &&` from the final filter predicate (~line 756).
- Remove `supervisor: None` from the `fake_ticket()` test helper (~line 1400).

**`apm/src/cmd/list.rs`**
- Remove `supervisor_filter: Option<String>` from `run()` signature.
- Remove the `supervisor_filter.as_deref()` argument passed to `list_filtered()`.

**`apm/src/main.rs`**
- Remove the `--supervisor` CLI argument block (~lines 122–124) from the `List` subcommand definition.
- Remove `supervisor` from the `Command::List { … }` destructure and the matching `cmd::list::run(…)` call (~line 742).
- Remove `supervisor` from the `set` command help text (~line 255) and field list (~line 270).

**`apm-core/src/queue.rs`**
- Remove `supervisor: None` from the ticket struct literal (~line 126).

**`apm-server/src/main.rs`**
- Remove `supervisor: None` from the test helper struct literal (~line 1852).
- Leave `supervisor_states` in `TicketsEnvelope` and `list_tickets()` untouched — it describes which workflow states require supervisor *role* action, not the removed frontmatter field.

**Tests**
- After the above changes, run `cargo test` to confirm all tests pass. Fix any remaining compilation errors from missed struct literal fields.

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-04-08T15:09Z | — | new | philippepascal |
| 2026-04-08T15:33Z | new | groomed | apm |
| 2026-04-08T15:39Z | groomed | in_design | philippepascal |
| 2026-04-08T15:42Z | in_design | specd | claude-0408-1539-b840 |
