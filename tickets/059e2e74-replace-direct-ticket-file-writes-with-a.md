+++
id = "059e2e74"
title = "Replace direct ticket-file writes with apm new"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/059e2e74-replace-direct-ticket-file-writes-with-a"
created_at = "2026-05-01T20:27:29.576253Z"
updated_at = "2026-05-02T04:25:18.763612Z"
epic = "0b1c71db"
target_branch = "epic/0b1c71db-integration-tests-use-real-apm-commands"
depends_on = ["795dce11"]
+++

## Spec

### Problem

The integration test file `apm/tests/integration.rs` contains ten helper functions that build ticket TOML frontmatter as raw string literals and write them directly to disk:

- `write_ticket_to_branch` (17 call sites) — generic helper covering states `new`, `ready`, `in_progress`, `implemented`, `ammend`
- `write_closed_ticket` (21 call sites) — always state `closed`
- `write_spec_ticket` (17 call sites) — state `in_progress` with Problem and Approach content
- `write_implemented_ticket` (4 call sites) — state `implemented`, used by squash/merge tests
- `write_in_progress_ticket` (4 call sites) — state `in_progress`, optional `target_branch` field
- `write_ticket_with_amendment_requests` (5 call sites) — state `ammend` with checkbox amendment content
- `write_ticket_with_owner` (7 call sites) — any state, adds `owner` field
- `write_ticket_with_epic` (3 call sites) — any state, optional `epic` field
- `write_ticket_in_epic` (6 call sites) — any state, `epic` + `owner` fields
- `write_ticket_with_agent` (0 call sites, dead code) — writes `agent` field

Beyond the helpers, five inline ticket constructions write frontmatter directly inside specific test bodies (lines ~660, ~999, ~1318, ~1879, ~3141). Three further constructions use `apm_core::git::commit_to_branch` with `concat!`-built frontmatter strings (lines ~393–430).

All of these share the same flaw: the frontmatter is synthesised offline, so tests use legacy integer IDs (`1`, `2`) rather than the production 8-character hex format, and silently stay green when required fields are added, field names change, or branch-naming rules evolve.

The desired state is that every ticket fixture goes through the real `apm` CLI (`apm new`, `apm state`, `apm set`, `apm spec`) so the test fixtures track production behaviour. Where a test deliberately requires a state that is unreachable through normal CLI flows — a ticket whose `branch` field references a non-existent remote branch, a field with no CLI setter — the direct write is retained and annotated `// BYPASS: <specific reason>`.

### Acceptance criteria

- [ ] `cargo test -p apm --test integration` passes with no new failures after all changes
- [ ] Every migrated helper body invokes `apm new` via `env!("CARGO_BIN_EXE_apm")` instead of constructing raw `+++\n` frontmatter strings
- [ ] Ticket IDs in migrated fixtures are dynamically generated 8-char hex strings (as produced by `apm new`), not hardcoded integers or fixed string literals
- [ ] No helper function or test body calls `write_ticket_with_agent` (the function is deleted)
- [ ] Every direct TOML write that cannot be replaced has a `// BYPASS: <specific reason>` comment on the immediately preceding line
- [ ] No migrated call site passes a hardcoded integer ID or pre-computed branch name as ticket identity; callers use the `(id, branch)` tuple returned by the helper

### Out of scope

Explicit list of what this ticket does not cover.

### Approach

How the implementation will work.

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-05-01T20:27Z | — | new | philippepascal |
| 2026-05-02T03:08Z | new | groomed | philippepascal |
| 2026-05-02T04:25Z | groomed | in_design | philippepascal |