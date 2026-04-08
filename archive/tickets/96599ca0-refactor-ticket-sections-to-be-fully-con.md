+++
id = "96599ca0"
title = "Refactor ticket sections to be fully config-driven, removing hardcoded TicketDocument fields"
state = "closed"
priority = 8
effort = 6
risk = 4
author = "claude-0401-2145-a8f3"
agent = "43280"
branch = "ticket/96599ca0-refactor-ticket-sections-to-be-fully-con"
created_at = "2026-04-01T22:27:39.127351Z"
updated_at = "2026-04-02T00:04:58.068067Z"
+++

## Spec

### Problem

`TicketDocument` in `apm-core/src/ticket.rs` (~line 503) hardcodes the ticket body as six typed Rust fields (`problem`, `acceptance_criteria`, `out_of_scope`, `approach`, `open_questions`, `amendment_requests`). `spec.rs` has matching hardcoded arms in `get_section`, `set_section`, and `is_doc_field`. Section order in serialization is also hardcoded.

The config already defines sections properly via `[[ticket.sections]]` in `.apm/config.toml` (name, type, required, placeholder), but this config is only used for skeleton generation and CLI validation — not at the model layer.

The consequence: adding any new section (e.g. a delegator-facing Context field) requires Rust code changes in `ticket.rs` and `spec.rs` instead of a config entry. Worse, sections not in `TicketDocument` get silently dropped on the next round-trip through `serialize`.

The fix is to replace `TicketDocument`'s typed fields with a config-driven ordered map. The server `CreateTicketRequest` in `apm-server/src/main.rs` (~line 91) also hardcodes the four main section fields — breaking that API is acceptable; the only client is `apm-ui` which must be fixed in the same PR.

### Acceptance criteria

- [x] Adding a new entry to `[[ticket.sections]]` in `.apm/config.toml` makes that section appear in newly created ticket skeletons without any Rust code changes
- [x] `apm spec <id> --section <name>` works for any section defined in config, not only the six currently hardcoded ones
- [x] A ticket containing a section whose name is not in config is preserved unchanged on a parse → serialize round-trip (no silent drops)
- [x] The "Code review" section (present in config but absent from `TicketDocument`) survives a parse-serialize round-trip on an existing ticket file
- [x] `TicketDocument` no longer declares individual typed Rust fields (`problem`, `acceptance_criteria`, etc.) — sections are stored in an ordered map
- [x] `get_section` and `set_section` in `spec.rs` contain no hardcoded section-name match arms
- [x] `is_doc_field` in `spec.rs` is removed; all callers route sections through `set_section` / `get_section` without branching on section name
- [x] `validate()` accepts a `&[TicketSection]` parameter and enforces `required = true` sections from config — no hardcoded field names
- [x] `CreateTicketRequest` in `apm-server/src/main.rs` no longer has individual named section fields; it accepts a generic sections map
- [x] `apm-ui` `NewTicketModal` sends form data using the new generic sections map shape
- [x] `cargo test --workspace` passes with no new failures after the refactor

### Out of scope

- Making the `apm-ui` form dynamically fetch section definitions from a config API endpoint (form fields remain hardcoded in the UI, only the payload shape changes)
- Adding new sections to the default `.apm/config.toml` (this ticket only makes new sections work once added; it does not add any)
- Changing the `SectionType` enum values or config parsing logic in `config.rs`
- Migrating existing ticket files on disk — old files continue to round-trip correctly
- Changes to `apm verify` beyond what is required to compile (if it accesses typed fields it will be updated to use the map, but no behaviour changes)

### Approach

**Step 1 — Add `indexmap` dependency**

In `apm-core/Cargo.toml`, add `indexmap = "2"`. `IndexMap` gives O(1) lookup and preserves insertion order, which means parse order == serialize order without requiring config at serialize time.

**Step 2 — Replace `TicketDocument` fields with an ordered map (`apm-core/src/ticket.rs`)**

Change the struct from six typed fields to:

```rust
pub struct TicketDocument {
    pub sections: IndexMap<String, String>,  // canonical name → raw body (no header)
    raw_history: String,
}
```

Section names use the canonical casing from config (e.g. "Problem", "Acceptance criteria"). Values are the raw markdown body of each section, stripped of the leading `### <name>` header line.

Remove `unchecked_criteria()` and `unchecked_amendments()` — they are replaced by a single helper:

```rust
pub fn unchecked_tasks(section_name: &str) -> Vec<usize>
```

which parses the raw string from `self.sections.get(section_name)` on demand and returns indices of unchecked `- [ ]` items. In `state.rs`, replace:
- `doc.unchecked_criteria()` → `doc.unchecked_tasks("Acceptance criteria")`
- `doc.unchecked_amendments()` → `doc.unchecked_tasks("Amendment requests")`

Search for all remaining typed-field accesses (`.problem`, `.acceptance_criteria`, `.out_of_scope`, `.approach`, `.open_questions`, `.amendment_requests`) across the workspace and replace with `doc.sections.get("…").map(String::as_str).unwrap_or("")` or the equivalent.

**Step 3 — Update `TicketDocument::parse()` (`ticket.rs`)**

- Scan the `## Spec` body for `### <name>` headings.
- Collect body text between consecutive headings into `self.sections` in file order.
- Stop when `## History` (or EOF) is reached; route that content to `raw_history` as before.
- Sections not present in config are still inserted into the map (preserves unknown sections on round-trip).

**Step 4 — Update `TicketDocument::serialize()` (`ticket.rs`)**

- Iterate over `self.sections` (IndexMap preserves insertion order).
- Emit `### <name>\n\n<body>\n\n` for each entry.
- Append `raw_history` at the end if non-empty.
- Remove all hardcoded if/else chains for individual section names.

**Step 5 — Update `TicketDocument::validate()` (`ticket.rs`)**

Change signature to:

```rust
pub fn validate(&self, config_sections: &[TicketSection]) -> Vec<ValidationError>
```

Implementation: for each config section with `required = true`, check that `self.sections` contains it and the value is non-empty. For `SectionType::Tasks` sections, also check that `unchecked_tasks(name)` returns no items if all criteria must be checked before transitioning.

All callers (`state.rs`, `verify.rs`, `cmd/spec.rs`) already hold a `Config`; thread `&config.ticket.sections` through each call site.

**Step 6 — Refactor `spec.rs`**

`get_section(doc, name)`: replace the match with a case-insensitive lookup over `doc.sections` keys, returning the value or an empty string.

`set_section(doc, name, value)`: replace the match with a case-insensitive key lookup; update the existing entry if found, otherwise insert with the supplied casing.

Remove `is_doc_field` entirely. The sole reason it existed was to route between the typed-field path and `set_section_body` — that distinction disappears now that the map accepts any key. Delete `set_section_body` and `get_section_body` from `spec.rs` once they have no callers. Update all call sites in `cmd/spec.rs` and `ticket.rs` to go through `set_section` / `get_section` unconditionally.

**Step 7 — Remove the hardcoded skeleton fallback (`ticket.rs`)**

Delete the fallback template string used when `config.ticket.sections` is empty. Tests that relied on this path must be updated to supply a minimal `TicketConfig` with the standard sections — a shared test helper `fn minimal_ticket_config() -> TicketConfig` is the cleanest approach. This ensures no hidden code path bypasses config-driven behaviour.

**Step 8 — Update `apm-server/src/main.rs`**

Replace `CreateTicketRequest`:

```rust
// Before
struct CreateTicketRequest {
    title: Option<String>,
    problem: Option<String>,
    acceptance_criteria: Option<String>,
    out_of_scope: Option<String>,
    approach: Option<String>,
}

// After
struct CreateTicketRequest {
    title: Option<String>,
    sections: Option<HashMap<String, String>>,
}
```

Update the `create_ticket` handler to build `section_sets` by iterating over `req.sections` instead of the four named fields. Preserve the filter-empty-values behaviour.

**Step 9 — Update `apm-ui/src/components/NewTicketModal.tsx`**

Change `CreateTicketData`:

```ts
interface CreateTicketData {
  title: string
  sections?: Record<string, string>
}
```

In the submit handler, build the sections map from the four textarea values, using the same human-readable labels as keys ("Problem", "Acceptance criteria", "Out of scope", "Approach"). The form's visual labels and textareas are unchanged — only the JSON payload shape changes.

**Step 10 — Update tests**

- Rewrite `TicketDocument` unit tests to access `doc.sections["Problem"]` etc.
- Add: parse a ticket body with an unrecognised section (`### Foo`), serialize, assert it is present in output.
- Add: parse a ticket body with `### Code review`, serialize, assert it survives.
- Add: `validate()` with a config that marks a section required returns an error when that section is empty.
- Run `cargo test --workspace` and fix any remaining compilation errors from typed-field accesses.

### Open questions



### Amendment requests



## History

| When | From | To | By |
|------|------|----|----|
| 2026-04-01T22:27Z | — | new | claude-0401-2145-a8f3 |
| 2026-04-01T22:28Z | new | groomed | claude-0401-2145-a8f3 |
| 2026-04-01T22:35Z | groomed | in_design | philippepascal |
| 2026-04-01T22:40Z | in_design | specd | claude-0401-2230-spec1 |
| 2026-04-01T22:51Z | specd | ready | apm |
| 2026-04-01T23:08Z | ready | in_progress | philippepascal |
| 2026-04-01T23:31Z | in_progress | implemented | claude-0401-0000-w96a |
| 2026-04-02T00:04Z | implemented | closed | apm-sync |