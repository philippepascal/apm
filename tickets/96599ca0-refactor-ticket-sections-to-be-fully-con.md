+++
id = "96599ca0"
title = "Refactor ticket sections to be fully config-driven, removing hardcoded TicketDocument fields"
state = "specd"
priority = 8
effort = 6
risk = 4
author = "claude-0401-2145-a8f3"
agent = "44317"
branch = "ticket/96599ca0-refactor-ticket-sections-to-be-fully-con"
created_at = "2026-04-01T22:27:39.127351Z"
updated_at = "2026-04-01T22:40:56.017978Z"
+++

## Spec

### Problem

`TicketDocument` in `apm-core/src/ticket.rs` (~line 503) hardcodes the ticket body as six typed Rust fields (`problem`, `acceptance_criteria`, `out_of_scope`, `approach`, `open_questions`, `amendment_requests`). `spec.rs` has matching hardcoded arms in `get_section`, `set_section`, and `is_doc_field`. Section order in serialization is also hardcoded.

The config already defines sections properly via `[[ticket.sections]]` in `.apm/config.toml` (name, type, required, placeholder), but this config is only used for skeleton generation and CLI validation — not at the model layer.

The consequence: adding any new section (e.g. a delegator-facing Context field) requires Rust code changes in `ticket.rs` and `spec.rs` instead of a config entry. Worse, sections not in `TicketDocument` get silently dropped on the next round-trip through `serialize`.

The fix is to replace `TicketDocument`'s typed fields with a config-driven ordered map. The server `CreateTicketRequest` in `apm-server/src/main.rs` (~line 91) also hardcodes the four main section fields — breaking that API is acceptable; the only client is `apm-ui` which must be fixed in the same PR.

### Acceptance criteria

- [ ] Adding a new entry to `[[ticket.sections]]` in `.apm/config.toml` makes that section appear in newly created ticket skeletons without any Rust code changes
- [ ] `apm spec <id> --section <name>` works for any section defined in config, not only the six currently hardcoded ones
- [ ] A ticket containing a section whose name is not in config is preserved unchanged on a parse → serialize round-trip (no silent drops)
- [ ] The "Code review" section (present in config but absent from `TicketDocument`) survives a parse-serialize round-trip on an existing ticket file
- [ ] `TicketDocument` no longer declares individual typed Rust fields (`problem`, `acceptance_criteria`, etc.) — sections are stored in an ordered map
- [ ] `get_section` and `set_section` in `spec.rs` contain no hardcoded section-name match arms
- [ ] `is_doc_field` in `spec.rs` is removed; all callers route sections through `set_section` / `get_section` without branching on section name
- [ ] `validate()` accepts a `&[TicketSection]` parameter and enforces `required = true` sections from config — no hardcoded field names
- [ ] `CreateTicketRequest` in `apm-server/src/main.rs` no longer has individual named section fields; it accepts a generic sections map
- [ ] `apm-ui` `NewTicketModal` sends form data using the new generic sections map shape
- [ ] `cargo test --workspace` passes with no new failures after the refactor

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

Note on `Vec<ChecklistItem>`: The current typed fields expose `acceptance_criteria: Vec<ChecklistItem>` and `amendment_requests: Option<Vec<ChecklistItem>>`. Any call sites that access these directly (search for `.acceptance_criteria` and `.amendment_requests` across the workspace) must be updated to parse the raw string on demand using `ChecklistItem::parse_list(&doc.sections["Acceptance criteria"])` (or equivalent helper). Do not add a new abstraction layer — just inline the parse where needed.

**Step 3 — Update `TicketDocument::parse()` (`ticket.rs`)**

- Scan the `## Spec` body for `### <name>` headings.
- Collect body text between consecutive headings into `self.sections` in file order.
- Stop when `## History` (or EOF) is reached; route that content to `raw_history` as before.
- Sections not present in config are still inserted into the map (preserves unknown sections).

**Step 4 — Update `TicketDocument::serialize()` (`ticket.rs`)**

- Iterate over `self.sections` (IndexMap preserves insertion order).
- Emit `### <name>\n\n<body>\n\n` for each entry.
- Append `raw_history` at the end if non-empty.
- Remove all hardcoded if/else chains for individual section names.

Because new tickets are created from config-ordered skeletons, their parse order equals config order. Existing tickets retain whatever order they had on disk.

**Step 5 — Refactor `spec.rs`**

`get_section(doc, name)`: replace the match with a case-insensitive lookup over `doc.sections` keys, returning the value or an empty string.

`set_section(doc, name, value)`: replace the match with a case-insensitive key lookup; update the existing entry if found, otherwise insert with the supplied casing.

`is_doc_field(name, config)`: change signature to accept `&Config` and return true if `config.ticket.sections` contains a section whose name matches case-insensitively. All call sites in `ticket.rs` already have config available; update them to pass it. Alternatively, since the only reason to distinguish "doc field" vs "raw body section" was the struct's fixed field set — now that the map accepts any key — consider removing `is_doc_field` entirely and always routing through `set_section`. Prefer removal if it simplifies call sites.

Remove all hardcoded match arms in `get_section` and `set_section`.

**Step 6 — Update ticket creation / section-setting logic (`ticket.rs`)**

The fallback hardcoded template (used when `config.ticket.sections` is empty) can remain for tests that run without a config, but any typed-field references in that path must be removed.

The section-setting loop that calls `is_doc_field` / `set_section` / `set_section_body` should be simplified: if the section name is found in the document map (or in config), use `set_section`; otherwise fall back to `set_section_body` for raw-body injection.

**Step 7 — Update `apm-server/src/main.rs`**

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

**Step 8 — Update `apm-ui/src/components/NewTicketModal.tsx`**

Change `CreateTicketData`:

```ts
interface CreateTicketData {
  title: string
  sections?: Record<string, string>
}
```

In the submit handler, build the sections map from the four textarea values, using the same human-readable labels as keys ("Problem", "Acceptance criteria", "Out of scope", "Approach"). The form's visual labels and textareas are unchanged — only the JSON payload shape changes.

**Step 9 — Update tests**

- Update `document_round_trip` and related unit tests in `ticket.rs` to access `doc.sections["Problem"]` etc. instead of `doc.problem`.
- Add a test: parse a ticket body containing an unrecognised section (e.g. `### Foo`), serialize, and assert the section is present in the output.
- Add a test: parse a ticket body containing `### Code review`, serialize, assert it survives.
- Run `cargo test --workspace` and fix any remaining compilation errors from typed-field accesses found across the workspace.

### Open questions



### Amendment requests



## History

| When | From | To | By |
|------|------|----|----|
| 2026-04-01T22:27Z | — | new | claude-0401-2145-a8f3 |
| 2026-04-01T22:28Z | new | groomed | claude-0401-2145-a8f3 |
| 2026-04-01T22:35Z | groomed | in_design | philippepascal |
| 2026-04-01T22:40Z | in_design | specd | claude-0401-2230-spec1 |