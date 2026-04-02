+++
id = "d877bd37"
title = "Add epic, target_branch, and depends_on fields to ticket frontmatter"
state = "specd"
priority = 10
effort = 3
risk = 2
author = "claude-0401-2145-a8f3"
agent = "50689"
branch = "ticket/d877bd37-add-epic-target-branch-and-depends-on-fi"
created_at = "2026-04-01T21:54:58.399434Z"
updated_at = "2026-04-02T00:48:59.247946Z"
+++

## Spec

### Problem

APM tickets currently have no way to express that they belong to a larger unit of work or that they depend on another ticket being completed first. Without these fields, all tickets are treated as independent, making it impossible to build epic-scoped workflows or enforce delivery ordering.

The full design is in `docs/epics.md` (§ Data model — Ticket frontmatter additions). Three new optional TOML frontmatter fields must be added to `TicketFrontmatter`:

- `epic = "<8-char-id>"` — associates the ticket with an epic branch
- `target_branch = "epic/<id>-<slug>"` — the branch the worktree and PR target (defaults to `main` when absent)
- `depends_on = ["<ticket-id>", ...]` — ticket IDs that must reach `implemented` before this ticket can be dispatched

All three fields are optional; omitting them preserves existing behaviour exactly. This ticket is the data-model foundation that all other epics tickets build on.

### Acceptance criteria

- [ ] A ticket file with `epic = "ab12cd34"` in frontmatter parses without error and `ticket.frontmatter.epic` equals `"ab12cd34"`
- [ ] A ticket file with `target_branch = "epic/ab12cd34-user-auth"` in frontmatter parses without error and `ticket.frontmatter.target_branch` equals `"epic/ab12cd34-user-auth"`
- [ ] A ticket file with `depends_on = ["cd56ef78", "12ab34cd"]` in frontmatter parses without error and `ticket.frontmatter.depends_on` equals `["cd56ef78", "12ab34cd"]`
- [ ] A ticket file with none of the three new fields parses without error, with all three fields absent/None (backward-compatible)
- [ ] Serialising a ticket whose `epic`, `target_branch`, and `depends_on` fields are absent produces no mention of those keys in the TOML output
- [ ] `pick_next` skips a ticket whose `depends_on` list contains at least one ID that corresponds to a ticket not yet in `implemented` or a terminal state
- [ ] `pick_next` returns a ticket whose `depends_on` entries are all in `implemented` or a terminal state
- [ ] `pick_next` does not skip a ticket whose `depends_on` references an ID that matches no known ticket (unknown dependency is treated as non-blocking)
- [ ] `apm state <id> implemented` opens the PR against `target_branch` when that field is set, instead of the configured default branch

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

**2. `apm-core/src/ticket.rs` — filter blocked tickets in `pick_next`**

Add a private helper:

```rust
fn is_implemented_or_later(state: &str, config: &crate::config::Config) -> bool {
    if config.workflow.states.iter()
        .find(|s| s.id == state)
        .map(|s| s.terminal)
        .unwrap_or(false) {
        return true;
    }
    let mut seen_implemented = false;
    for s in &config.workflow.states {
        if s.id == "implemented" { seen_implemented = true; }
        if seen_implemented && s.id == state { return true; }
    }
    false
}

fn depends_satisfied(ticket: &Ticket, all: &[Ticket], config: &crate::config::Config) -> bool {
    let Some(deps) = &ticket.frontmatter.depends_on else { return true; };
    deps.iter().all(|dep_id| {
        match all.iter().find(|t| t.frontmatter.id.starts_with(dep_id.as_str())) {
            None => true,  // unknown dep -> not blocking
            Some(t) => is_implemented_or_later(&t.frontmatter.state, config),
        }
    })
}
```

Update `pick_next` signature to accept config and filter with `depends_satisfied`:

```rust
pub fn pick_next<'a>(
    tickets: &'a [Ticket],
    actionable: &[&str],
    startable: &[&str],
    pw: f64, ew: f64, rw: f64,
    config: &crate::config::Config,
) -> Option<&'a Ticket>
```

In the `.find()` closure, add `depends_satisfied(t, tickets, config)` to the existing condition. Update both call sites in `start.rs` (`run_next` and `spawn_next_worker`) to pass `&config`.

**3. `apm-core/src/state.rs` — use `target_branch` for PR creation**

In `transition`, at the `CompletionStrategy::Pr` arm, `t` (the loaded ticket) is in scope. Change the call to `gh_pr_create_or_update` to resolve the base branch from `t.frontmatter.target_branch`, falling back to `config.project.default_branch` when absent.

**4. Tests — inline in `apm-core/src/ticket.rs`**

Add to the existing `#[cfg(test)]` block using the existing `minimal_raw` / `dummy_path` helpers:

- `parse_epic_field` — extra frontmatter `epic = "ab12cd34"`, assert `frontmatter.epic == Some("ab12cd34")`
- `parse_target_branch_field` — extra frontmatter `target_branch = "epic/ab12cd34-foo"`, assert value
- `parse_depends_on_field` — extra frontmatter `depends_on = ["cd56ef78"]`, assert vec
- `parse_omits_new_fields` — ticket with no new fields, assert all three are `None`
- `serialize_omits_absent_fields` — round-trip; serialized output must not contain the key names `epic`, `target_branch`, or `depends_on`
- `pick_next_skips_blocked_ticket` — two tickets A and B; B has `depends_on` pointing to A; A is in `ready` state; `pick_next` returns only A (the one without unresolved deps)
- `pick_next_returns_satisfied_dep` — A is in `implemented` state; `pick_next` returns B (dep satisfied)
- `pick_next_unknown_dep_not_blocking` — B's `depends_on` ID matches no ticket in the list; `pick_next` returns B

Tests for `pick_next` need a minimal `Config` — build it from a small inline TOML string via `toml::from_str`.

### 1. `apm-core/src/ticket.rs` — add three optional fields to `Frontmatter`

```rust
#[serde(skip_serializing_if = "Option::is_none")]
pub epic: Option<String>,

#[serde(skip_serializing_if = "Option::is_none")]
pub target_branch: Option<String>,

#[serde(skip_serializing_if = "Option::is_none")]
pub depends_on: Option<Vec<String>>,
```

All three use `skip_serializing_if = "Option::is_none"` so existing ticket files are unchanged on round-trip. No `#[serde(default)]` needed — missing fields deserialise as `None` by default.

### 2. `apm-core/src/ticket.rs` — filter blocked tickets in `pick_next`

Add a helper that checks whether a ticket's dependencies are all satisfied:

```rust
fn depends_satisfied(ticket: &Ticket, all: &[Ticket], config: &crate::config::Config) -> bool {
    let Some(deps) = &ticket.frontmatter.depends_on else { return true; };
    deps.iter().all(|dep_id| {
        let found = all.iter().find(|t| t.frontmatter.id.starts_with(dep_id.as_str()));
        match found {
            None => true,  // unknown dep -> not blocking
            Some(t) => is_implemented_or_later(&t.frontmatter.state, config),
        }
    })
}
```

`is_implemented_or_later` walks the ordered workflow states list and returns `true` when the state has `terminal = true` OR appears at or after the state named `"implemented"` in that list. Config-driven, not hardcoded.

Update `pick_next` to accept the full ticket slice and config:

```rust
pub fn pick_next<'a>(
    tickets: &'a [Ticket],
    actionable: &[&str],
    startable: &[&str],
    pw: f64, ew: f64, rw: f64,
    config: &crate::config::Config,
) -> Option<&'a Ticket>
```

Inside the `.find()` closure, add `&& depends_satisfied(t, tickets, config)`.

Update both call sites in `start.rs` (`run_next` and `spawn_next_worker`) to pass `&config`.

### 3. `apm-core/src/state.rs` — use `target_branch` for PR creation

In the `CompletionStrategy::Pr` arm of `transition`, `t` (the loaded ticket) is already in scope. Change the `gh_pr_create_or_update` call:

```rust
let pr_base = t.frontmatter.target_branch.as_deref()
    .unwrap_or(&config.project.default_branch);
gh_pr_create_or_update(root, &branch, pr_base, &id, &t.frontmatter.title)?;
```

### 4. Tests — inline in `apm-core/src/ticket.rs`

Add to the existing `#[cfg(test)]` block using the existing `minimal_raw` helper:

- `parse_epic_field` — parses `epic = "ab12cd34"`, asserts value equals `"ab12cd34"`
- `parse_target_branch_field` — parses `target_branch = "epic/ab12cd34-foo"`, asserts value
- `parse_depends_on_field` — parses `depends_on = ["cd56ef78"]`, asserts vec
- `parse_omits_new_fields` — ticket without any of the three new fields parses fine with all None
- `serialize_omits_absent_fields` — round-trip with None fields produces no mention of the keys
- `pick_next_skips_blocked_ticket` — two tickets, one depends_on the other which is `ready`; pick_next returns only the independent one
- `pick_next_returns_satisfied_dep` — dep ticket is `implemented`; pick_next returns the dependent ticket
- `pick_next_unknown_dep_not_blocking` — depends_on ID matches nothing; ticket is still returned

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-04-01T21:54Z | — | new | claude-0401-2145-a8f3 |
| 2026-04-01T21:59Z | new | groomed | claude-0401-2145-a8f3 |
| 2026-04-02T00:43Z | groomed | in_design | philippepascal |
| 2026-04-02T00:48Z | in_design | specd | claude-0401-2330-spec1 |
