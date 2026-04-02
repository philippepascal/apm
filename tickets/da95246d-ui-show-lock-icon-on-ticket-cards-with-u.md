+++
id = "da95246d"
title = "UI: show lock icon on ticket cards with unresolved depends_on"
state = "in_design"
priority = 2
effort = 3
risk = 0
author = "claude-0401-2145-a8f3"
agent = "64122"
branch = "ticket/da95246d-ui-show-lock-icon-on-ticket-cards-with-u"
created_at = "2026-04-01T21:56:15.495249Z"
updated_at = "2026-04-02T00:58:43.204473Z"
+++

## Spec

### Problem

Ticket cards on the supervisor board give no visual signal when a ticket is waiting on unresolved `depends_on` entries. An engineer looking at the board cannot tell at a glance which tickets are blocked by dependencies and why they are not being dispatched.

The `depends_on` field is not yet part of the `Frontmatter` struct in `apm-core`, so no dependency data is tracked or surfaced through the API. Adding it and exposing it to the UI unlocks this and future dependency-aware features.

The desired behaviour (per `docs/epics.md` § "Ticket cards") is: cards where `depends_on` has at least one unresolved entry show a small lock icon; hovering the icon shows a tooltip listing the blocking ticket IDs and their current states.

### Acceptance criteria

- [ ] A supervisor-board ticket card with at least one dependency whose state is not `implemented`, `merged`, or `closed` shows a lock icon
- [ ] A supervisor-board ticket card whose `depends_on` list is absent or empty shows no lock icon
- [ ] A supervisor-board ticket card where every `depends_on` entry is in state `implemented`, `merged`, or `closed` shows no lock icon
- [ ] Hovering the lock icon reveals a tooltip that lists each unresolved dependency as `<id>: <state>` (one per line)
- [ ] `GET /api/tickets` includes a `blocking_deps` array for every ticket (empty array when there are none)
- [ ] `blocking_deps` entries are computed server-side and contain only dependencies not yet in `implemented`, `merged`, or `closed`

### Out of scope

- Setting `depends_on` via the UI (creating or editing tickets with dependency lists)
- The priority queue panel (`PriorityQueuePanel.tsx`) — it renders table rows, not cards; lock icon coverage there is a separate task
- The ticket detail panel's `depends_on` display (also described in the epic, separate ticket)
- Epic-related frontmatter fields (`epic`, `target_branch`)
- Blocking the dispatch engine based on `depends_on` (separate concern)

### Approach

**1. `apm-core/src/ticket.rs` — add `depends_on` to `Frontmatter`**

Add below `focus_section`:

```rust
#[serde(default, skip_serializing_if = "Vec::is_empty")]
pub depends_on: Vec<String>,
```

Using `Vec` (not `Option<Vec>`) with `#[serde(default)]` means absent fields deserialise to an empty vec and existing tickets without the field are unaffected.

---

**2. `apm-server/src/main.rs` — expose `blocking_deps` in the API**

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

In `list_tickets`, build a state lookup map before the `.map()` call:

```rust
let state_map: std::collections::HashMap<&str, &str> = tickets
    .iter()
    .map(|t| (t.frontmatter.id.as_str(), t.frontmatter.state.as_str()))
    .collect();
```

Then compute `blocking_deps` per ticket (resolved = `implemented`, `merged`, or `closed`):

```rust
const RESOLVED: &[&str] = &["implemented", "merged", "closed"];
let blocking_deps = t.frontmatter.depends_on.iter()
    .filter_map(|dep_id| {
        state_map.get(dep_id.as_str()).and_then(|&s| {
            if RESOLVED.contains(&s) { None }
            else { Some(BlockingDep { id: dep_id.clone(), state: s.to_string() }) }
        })
    })
    .collect();
```

Unknown dep IDs (not in the map) are silently skipped.

---

**3. `apm-ui/src/components/supervisor/types.ts` — extend `Ticket` interface**

```ts
blocking_deps?: { id: string; state: string }[]
```

---

**4. `apm-ui/src/components/supervisor/TicketCard.tsx` — render the lock icon**

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

**5. Tests**

Add a test in `apm-server/src/main.rs` (alongside `list_tickets_includes_badge_fields`) asserting:

- A ticket with no `depends_on` has `blocking_deps: []`
- A ticket whose dep is in `implemented` has `blocking_deps: []`
- A ticket whose dep is in `in_progress` has `blocking_deps: [{id, state}]`

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-04-01T21:56Z | — | new | claude-0401-2145-a8f3 |
| 2026-04-01T22:01Z | new | groomed | claude-0401-2145-a8f3 |
| 2026-04-02T00:54Z | groomed | in_design | philippepascal |