+++
id = "da95246d"
title = "UI: show lock icon on ticket cards with unresolved depends_on"
state = "closed"
priority = 2
effort = 4
risk = 2
author = "claude-0401-2145-a8f3"
agent = "1792"
branch = "ticket/da95246d-ui-show-lock-icon-on-ticket-cards-with-u"
created_at = "2026-04-01T21:56:15.495249Z"
updated_at = "2026-04-02T19:07:54.681126Z"
+++

## Spec

### Problem

Ticket cards on the supervisor board give no visual signal when a ticket is waiting on unresolved `depends_on` entries. An engineer looking at the board cannot tell at a glance which tickets are blocked by dependencies and why they are not being dispatched.

The `depends_on` field is not yet part of the `Frontmatter` struct in `apm-core`, so no dependency data is tracked or surfaced through the API. Adding it and exposing it to the UI unlocks this and future dependency-aware features.

The desired behaviour (per `docs/epics.md` § "Ticket cards") is: cards where `depends_on` has at least one unresolved entry show a small lock icon; hovering the icon shows a tooltip listing the blocking ticket IDs and their current states.

### Acceptance criteria

- [x] A supervisor-board ticket card with at least one dependency whose state is not a `satisfies_deps` or `terminal` state (per `config.workflow.states`) shows a lock icon
- [x] A supervisor-board ticket card whose `depends_on` field is absent (`None`) or empty shows no lock icon
- [x] A supervisor-board ticket card where every `depends_on` entry is in a `satisfies_deps` or `terminal` state shows no lock icon
- [x] Hovering the lock icon reveals a tooltip that lists each unresolved dependency as `<id>: <state>` (one per line)
- [x] `GET /api/tickets` includes a `blocking_deps` array for every ticket (empty array when there are none)
- [x] `blocking_deps` entries are computed server-side by checking each dep's state against the set of states where `satisfies_deps = true` or `terminal = true` in `config.workflow.states`; no hardcoded state-name comparisons

### Out of scope

- Setting `depends_on` via the UI (creating or editing tickets with dependency lists)
- The priority queue panel (`PriorityQueuePanel.tsx`) — it renders table rows, not cards; lock icon coverage there is a separate task
- The ticket detail panel's `depends_on` display (also described in the epic, separate ticket)
- Epic-related frontmatter fields (`epic`, `target_branch`)
- Blocking the dispatch engine based on `depends_on` (separate concern)

### Approach

**1. `apm-core/src/config.rs` — add `satisfies_deps` to `StateConfig`**

Add a new field after `terminal`:

```rust
#[serde(default)]
pub satisfies_deps: bool,
```

A dependency is resolved when its state has `satisfies_deps || terminal` set to `true`.

---

**2. `.apm/workflow.toml` — mark states that satisfy deps**

Add `satisfies_deps = true` to the `implemented` and `closed` state entries:

```toml
[[workflow.states]]
id             = "implemented"
label          = "Implemented"
satisfies_deps = true
...

[[workflow.states]]
id             = "closed"
terminal       = true
satisfies_deps = true
...
```

(`closed` already has `terminal = true`; adding `satisfies_deps` makes the intent explicit.)

---

**3. `apm-core/src/ticket.rs` — add `depends_on` to `Frontmatter`**

Add after `focus_section`, consistent with surrounding `Option<…>` fields:

```rust
#[serde(skip_serializing_if = "Option::is_none")]
pub depends_on: Option<Vec<String>>,
```

`Option<Vec<String>>` matches the pattern used by `author`, `agent`, `branch`, etc. Absent `depends_on` deserialises to `None`; existing tickets are unaffected.

---

**4. `apm-server/src/main.rs` — expose `blocking_deps` in the API**

Add a serialisable struct above `TicketResponse`:

```rust
#[derive(serde::Serialize)]
struct BlockingDep {
    id: String,
    state: String,
}
```

Add the field to `TicketResponse`:

```rust
blocking_deps: Vec<BlockingDep>,
```

In `list_tickets`, load config and derive the resolved-state set before the `.map()`:

```rust
let resolved_ids: Vec<String> = match state.git_root() {
    Some(root) => {
        let cfg = apm_core::config::Config::load(root)?;
        cfg.workflow.states.into_iter()
            .filter(|s| s.satisfies_deps || s.terminal)
            .map(|s| s.id)
            .collect()
    }
    None => vec![],
};
let resolved: std::collections::HashSet<&str> =
    resolved_ids.iter().map(|s| s.as_str()).collect();
```

Then build a state lookup map and compute `blocking_deps` per ticket:

```rust
let state_map: std::collections::HashMap<&str, &str> = tickets
    .iter()
    .map(|t| (t.frontmatter.id.as_str(), t.frontmatter.state.as_str()))
    .collect();

// inside .map(|t| { … })
let blocking_deps = t.frontmatter.depends_on
    .as_deref()
    .unwrap_or(&[])
    .iter()
    .filter_map(|dep_id| {
        state_map.get(dep_id.as_str()).and_then(|&s| {
            if resolved.contains(s) { None }
            else { Some(BlockingDep { id: dep_id.clone(), state: s.to_string() }) }
        })
    })
    .collect();
```

Unknown dep IDs (not in the map) are silently skipped.

---

**5. `apm-ui/src/components/supervisor/types.ts` — extend `Ticket` interface**

```ts
blocking_deps?: { id: string; state: string }[]
```

---

**6. `apm-ui/src/components/supervisor/TicketCard.tsx` — render the lock icon**

Import `Lock` from `lucide-react`.

Inside the badges `<div>`, after the `has_pending_amendments` block:

```tsx
{!!ticket.blocking_deps?.length && (
  <Lock
    size={12}
    title={ticket.blocking_deps.map(d => `${d.id}: ${d.state}`).join('\n')}
    className="text-gray-400 shrink-0"
  />
)}
```

The native `title` attribute matches the pattern used by the existing `?` and `A` badges.

---

**7. Tests**

Add a test in `apm-server/src/main.rs` (alongside `list_tickets_includes_badge_fields`) asserting:

- A ticket with no `depends_on` field has `blocking_deps: []`
- A ticket whose dep is in a state with `satisfies_deps = true` has `blocking_deps: []`
- A ticket whose dep is in a non-satisfies state (e.g. `in_progress`) has `blocking_deps: [{id, state}]`

The test config must include at least one state with `satisfies_deps = true` (e.g., `implemented`).

### Open questions


### Amendment requests

- [x] Remove `"merged"` from the resolved states list — that state does not exist in the workflow.
- [x] The RESOLVED set must not be a hardcoded list of state names. Use `satisfies_deps = true` or `terminal = true` from `config.workflow.states` to determine whether a dep is resolved. The server must load config and use those flags; no string comparison against state IDs. Update AC and Approach accordingly.
- [x] Change `depends_on` field type to `Option<Vec<String>>` with `#[serde(skip_serializing_if = "Option::is_none")]`, consistent with d877bd37 and ba4e8499. Remove `#[serde(default)]` from a non-Option Vec; the Option approach handles absence cleanly.

### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-04-01T21:56Z | — | new | claude-0401-2145-a8f3 |
| 2026-04-01T22:01Z | new | groomed | claude-0401-2145-a8f3 |
| 2026-04-02T00:54Z | groomed | in_design | philippepascal |
| 2026-04-02T00:58Z | in_design | specd | claude-0402-0100-spec1 |
| 2026-04-02T01:37Z | specd | ammend | philippepascal |
| 2026-04-02T01:43Z | ammend | in_design | philippepascal |
| 2026-04-02T01:46Z | in_design | specd | claude-0402-0200-spec2 |
| 2026-04-02T02:29Z | specd | ready | apm |
| 2026-04-02T06:52Z | ready | in_progress | philippepascal |
| 2026-04-02T07:00Z | in_progress | implemented | claude-0401-2145-impl1 |
| 2026-04-02T19:07Z | implemented | closed | apm-sync |