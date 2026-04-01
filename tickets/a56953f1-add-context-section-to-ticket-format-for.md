+++
id = "a56953f1"
title = "Make ticket body sections fully config-driven"
state = "specd"
priority = 0
effort = 2
risk = 1
author = "claude-0401-2145-a8f3"
branch = "ticket/a56953f1-add-context-section-to-ticket-format-for"
created_at = "2026-04-01T22:09:53.033510Z"
updated_at = "2026-04-01T22:21:43.547233Z"
+++

## Spec

### Problem

The ticket body section model has two sources of truth. `[[ticket.sections]]` in `config.toml` drives skeleton generation and `apm spec` validation, but `TicketDocument` hardcodes six typed fields (`problem`, `acceptance_criteria`, `out_of_scope`, `approach`, `open_questions`, `amendment_requests`) with a fixed serialization order.

Any section not in this struct — including custom sections like a delegator-facing `Context` — either bypasses `TicketDocument` entirely via raw string manipulation, or gets silently dropped the next time any doc-field section is updated (because `TicketDocument::serialize` only outputs its six hardcoded fields).

Adding a new section today requires code changes in `ticket.rs`, `spec.rs`, and `apm-server` instead of a single config entry. The config already has everything needed (`name`, `type`, `required`, `placeholder`); it just isn't used at the model layer.

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
| 2026-04-01T22:12Z | in_design | specd | claude-0401-2145-a8f3 |