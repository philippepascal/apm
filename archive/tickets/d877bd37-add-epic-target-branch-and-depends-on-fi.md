+++
id = "d877bd37"
title = "Add epic, target_branch, and depends_on fields to ticket frontmatter"
state = "closed"
priority = 10
effort = 2
risk = 1
author = "claude-0401-2145-a8f3"
agent = "19690"
branch = "ticket/d877bd37-add-epic-target-branch-and-depends-on-fi"
created_at = "2026-04-01T21:54:58.399434Z"
updated_at = "2026-04-02T02:36:54.510351Z"
+++

## Spec

### Problem

APM tickets currently have no way to express that they belong to a larger unit of work or that they depend on another ticket being completed first. Without these fields, all tickets are treated as independent, making it impossible to build epic-scoped workflows or enforce delivery ordering.

The full design is in `docs/epics.md` (§ Data model — Ticket frontmatter additions). Three new optional TOML frontmatter fields must be added to `TicketFrontmatter`:

- `epic = "<8-char-id>"` — associates the ticket with an epic branch
- `target_branch = "epic/<id>-<slug>"` — the branch the worktree and PR target (defaults to `main` when absent)
- `depends_on = ["<ticket-id>", ...]` — ticket IDs that must reach a state with `satisfies_deps = true` in workflow config before this ticket can be dispatched

All three fields are optional; omitting them preserves existing behaviour exactly. This ticket is the data-model foundation that all other epics tickets build on.

### Acceptance criteria

- [x] A ticket file with `epic = "ab12cd34"` in frontmatter parses without error and `ticket.frontmatter.epic` equals `"ab12cd34"`
- [x] A ticket file with `target_branch = "epic/ab12cd34-user-auth"` in frontmatter parses without error and `ticket.frontmatter.target_branch` equals `"epic/ab12cd34-user-auth"`
- [x] A ticket file with `depends_on = ["cd56ef78", "12ab34cd"]` in frontmatter parses without error and `ticket.frontmatter.depends_on` equals `["cd56ef78", "12ab34cd"]`
- [x] A ticket file with none of the three new fields parses without error, with all three fields absent/None (backward-compatible)
- [x] Serialising a ticket whose `epic`, `target_branch`, and `depends_on` fields are absent produces no mention of those keys in the TOML output

### Out of scope

- `apm epic` subcommands (new, list, show, close) — covered by a separate ticket
- `apm new --epic` flag and epic-aware ticket creation — separate ticket
- apm-server epic API routes (`GET/POST /api/epics`) — separate ticket
- apm-ui epic filter, ticket card lock icon, and engine epic selector — separate ticket
- `apm work --epic` exclusive-mode flag — separate ticket
- `apm epic sync` / merging epic branches — explicitly not planned
- Validation that `epic` and `target_branch` are consistent with each other

### Approach

**1. `apm-core/src/ticket.rs` — add three optional fields to `Frontmatter`**

Add after the existing optional fields:

```rust
#[serde(skip_serializing_if = "Option::is_none")]
pub epic: Option<String>,

#[serde(skip_serializing_if = "Option::is_none")]
pub target_branch: Option<String>,

#[serde(skip_serializing_if = "Option::is_none")]
pub depends_on: Option<Vec<String>>,
```

All three use `skip_serializing_if = "Option::is_none"` so existing ticket files are unchanged on round-trip. No `#[serde(default)]` needed — missing TOML fields deserialise as `None` automatically.

**2. Tests — inline in `apm-core/src/ticket.rs`**

Add to the existing `#[cfg(test)]` block using the existing `minimal_raw` / `dummy_path` helpers:

- `parse_epic_field` — extra frontmatter `epic = "ab12cd34"`, assert `frontmatter.epic == Some("ab12cd34")`
- `parse_target_branch_field` — extra frontmatter `target_branch = "epic/ab12cd34-foo"`, assert value
- `parse_depends_on_field` — extra frontmatter `depends_on = ["cd56ef78"]`, assert vec
- `parse_omits_new_fields` — ticket with no new fields, assert all three are `None`
- `serialize_omits_absent_fields` — round-trip; serialised output must not contain the key names `epic`, `target_branch`, or `depends_on`

### Open questions


### Amendment requests

- [x] Delete the duplicate sections "### 2. filter blocked tickets in pick_next", "### 3. use target_branch for PR creation", and "### 4. Tests" that remain in the Approach body — they were not removed in the previous amendment and still instruct the worker to implement dep-scheduling and PR-targeting code that belongs to c1ff90de and d3749f24. The spec must contain only the Frontmatter field addition and its five parse/round-trip tests.
- [x] The `### Problem` section says `depends_on = ["<ticket-id>", ...]` — ticket IDs that must reach `implemented` before this ticket can be dispatched. Replace `implemented` with `a state with satisfies_deps = true in workflow config` — no hardcoded state names anywhere in the spec.

### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-04-01T21:54Z | — | new | claude-0401-2145-a8f3 |
| 2026-04-01T21:59Z | new | groomed | claude-0401-2145-a8f3 |
| 2026-04-02T00:43Z | groomed | in_design | philippepascal |
| 2026-04-02T00:48Z | in_design | specd | claude-0401-2330-spec1 |
| 2026-04-02T01:36Z | specd | ammend | philippepascal |
| 2026-04-02T01:40Z | ammend | in_design | philippepascal |
| 2026-04-02T01:42Z | in_design | specd | claude-0402-0200-spec2 |
| 2026-04-02T01:55Z | specd | ammend | philippepascal |
| 2026-04-02T01:55Z | ammend | in_design | philippepascal |
| 2026-04-02T01:58Z | in_design | specd | claude-0402-0300-spec3 |
| 2026-04-02T02:03Z | specd | ammend | apm |
| 2026-04-02T02:11Z | ammend | in_design | philippepascal |
| 2026-04-02T02:11Z | in_design | specd | claude-0402-0402-spec4 |
| 2026-04-02T02:21Z | specd | ammend | apm |
| 2026-04-02T02:21Z | ammend | in_design | philippepascal |
| 2026-04-02T02:21Z | in_design | specd | claude-0402-0500-spec5 |
| 2026-04-02T02:28Z | specd | ready | apm |
| 2026-04-02T02:30Z | ready | in_progress | philippepascal |
| 2026-04-02T02:32Z | in_progress | implemented | claude-0402-0600-work1 |
| 2026-04-02T02:36Z | implemented | closed | apm-sync |