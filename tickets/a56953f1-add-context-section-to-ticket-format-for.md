+++
id = "a56953f1"
title = "Add Context section to ticket format for delegator handoff"
state = "in_design"
priority = 0
effort = 2
risk = 1
author = "claude-0401-2145-a8f3"
branch = "ticket/a56953f1-add-context-section-to-ticket-format-for"
created_at = "2026-04-01T22:09:53.033510Z"
updated_at = "2026-04-01T22:12:18.472159Z"
+++

## Spec

### Problem

When a delegator creates a ticket and promotes it to `groomed`, the spec-writer worker receives nothing beyond the ticket title. There is no sanctioned place in the ticket format for the delegator to record the relevant design document, the relevant section, or known constraints (e.g. "the `accepted` state has been removed").

The existing sections (`### Problem`, `### Acceptance criteria`, etc.) are worker-owned. Pre-filling them creates ambiguity about whether the worker should preserve or replace the content.

The result: spec-writers must guess intent from the title alone and often produce specs that miss the design or require amendment cycles.

### Acceptance criteria

- [ ] `apm spec <id> --section Context --set "..."` writes a `### Context` section to the ticket
- [ ] `apm show <id>` renders the Context section immediately before `### Problem`
- [ ] `### Context` is optional — tickets without it parse and validate successfully
- [ ] Context is not required for `apm state <id> specd` (the quality bar still checks only Problem, Acceptance criteria, Out of scope, Approach)
- [ ] New ticket skeletons produced by `apm new` include an empty `### Context` placeholder before `### Problem`
- [ ] `apm spec <id> --section Context` (get) returns the current context value
- [ ] `.apm/agents.md` Delegator section instructs the delegator to populate `### Context` after each `apm new` call, before promoting to `groomed`
- [ ] `.apm/agents.md` Worker `state = groomed` section instructs the worker to read `### Context` before writing any spec section

### Out of scope

- Making `### Context` required or validated against a schema
- Parsing Context content programmatically — it is purely human/agent-readable prose
- Changes to `apm new` CLI flags
- UI changes
- Migrating existing tickets to add empty Context sections

### Approach

Four changes, each independent.

**1. `apm-core/src/ticket.rs` — add `context` field to `TicketDocument`**

Add `pub context: Option<String>` to the `TicketDocument` struct (line ~503).

In `extract_sections` (already parses all `###` headings into a HashMap — no change needed there).

In `TicketDocument::parse` (line ~562): after the required-section check, add:
```rust
context: sections.get("Context").cloned(),
```

In `TicketDocument::serialize` (line ~588): emit Context before Problem:
```rust
if let Some(ctx) = &self.context {
    out.push_str("\n### Context\n\n");
    out.push_str(ctx);
    out.push('\n');
}
```

New ticket skeleton (line ~389): add `### Context\n\n` before `### Problem`:
```rust
format!("## Spec\n\n### Context\n\n### Problem\n\n### Acceptance criteria\n\n### Out of scope\n\n### Approach\n\n{history_footer}")
```

**2. `apm-core/src/spec.rs` — teach `get_section` / `set_section` / `is_doc_field`**

In `get_section` (line ~5): add arm:
```rust
"context" => doc.context.clone(),
```

In `set_section` (line ~29): add arm:
```rust
"context" => doc.context = if value.is_empty() { None } else { Some(value) },
```

In `is_doc_field` (line ~134): add `"context"` to the match list.

**3. `.apm/agents.md` — Delegator section**

After the existing dispatch loop step ("Call `apm start --next --spawn`..."), add a note to the ticket-creation flow. The clearest place is a new bullet under the **Before dispatching** heading or as a callout after step 2:

> After `apm new --no-edit "<title>"`, immediately populate `### Context` before promoting to `groomed`:
> ```bash
> apm spec <id> --section Context --set "See <doc-path> § <section>. <1–2 sentence intent and key constraints.>"
> ```
> Then `apm state <id> groomed`.

**4. `.apm/agents.md` — Worker `state = groomed` section**

Add as step 1 (before the existing "1. `apm show <id>`"):

> 0. Read `### Context` — the delegator has recorded the relevant design document and section. Locate that document before writing any spec section.

(Renumber existing steps 1–N to 2–N+1.)

**Tests to add in `apm-core/src/ticket.rs` (alongside existing `TicketDocument` tests):**
- Parse a body with a `### Context` section → `doc.context` is `Some(...)`
- Parse a body without `### Context` → `doc.context` is `None`
- Serialize a doc with context → `### Context` appears before `### Problem`
- Serialize a doc without context → no `### Context` heading in output

**Tests to add in `apm-core/src/spec.rs`:**
- `get_section(&doc, "Context")` returns the context value
- `set_section(&mut doc, "context", ...)` sets `doc.context`

### Open questions



### Amendment requests



## History

| When | From | To | By |
|------|------|----|----|
| 2026-04-01T22:09Z | — | new | claude-0401-2145-a8f3 |
| 2026-04-01T22:10Z | new | groomed | claude-0401-2145-a8f3 |
| 2026-04-01T22:10Z | groomed | in_design | claude-0401-2145-a8f3 |