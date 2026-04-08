+++
id = "5ec960eb"
title = "refactor: move spec section logic from spec.rs into apm-core"
state = "closed"
priority = 0
effort = 3
risk = 2
author = "claude-0330-0245-main"
agent = "83352"
branch = "ticket/5ec960eb-refactor-move-spec-section-logic-from-sp"
created_at = "2026-03-30T14:27:31.109323Z"
updated_at = "2026-03-30T18:08:30.948576Z"
+++

## Spec

### Problem

spec.rs contains 394 lines of spec-document manipulation logic that belongs in apm-core:

- Section format enforcement based on SectionType (tasks/checkboxes, QA format, free text)
- Setting named sections on TicketDocument fields (Problem, Acceptance criteria, Out of scope, Approach, Open questions, Amendment requests)
- Acceptance criteria checkbox parsing and toggling (the `mark` subcommand)
- Raw body section get/set for custom sections not mapped to TicketDocument fields
- Section printing helpers shared by multiple code paths

None of this is CLI-specific — it all operates on ticket document structure.
apm-serve will need to read and write spec sections from the browser (e.g.
checking off acceptance criteria). Without this refactor it must shell out to
`apm spec` or duplicate all the parsing logic.

Target: a new `apm_core::spec` module exposing `get_section()`, `set_section()`,
`apply_section_type()`, `mark_item()`, `get_section_body()`, and `set_section_body()`.
CLI `spec.rs` becomes a thin wrapper of ~50 lines.

### Acceptance criteria

- [x] `apm_core::spec::get_section` returns the problem text when called with "Problem"
- [x] `apm_core::spec::get_section` returns the checklist serialized as markdown when called with "Acceptance criteria"
- [x] `apm_core::spec::get_section` returns None when the section name is unknown
- [x] `apm_core::spec::set_section` sets doc.problem when called with "problem" (case-insensitive)
- [x] `apm_core::spec::set_section` parses checklist lines into doc.acceptance_criteria when called with "acceptance criteria"
- [x] `apm_core::spec::set_section` parses checklist lines into doc.amendment_requests when called with "amendment requests"
- [x] `apm_core::spec::apply_section_type` with Tasks wraps a bare line in `- [ ] ` prefix
- [x] `apm_core::spec::apply_section_type` with Tasks leaves a pre-formatted `- [ ] ` line unchanged
- [x] `apm_core::spec::apply_section_type` with Qa prefixes a bare line with `**Q:** `
- [x] `apm_core::spec::apply_section_type` with Free returns the value unchanged
- [x] `apm_core::spec::mark_item` replaces the matching unchecked item with a checked one
- [x] `apm_core::spec::mark_item` returns an error when no unchecked item matches the text
- [x] `apm_core::spec::mark_item` returns an error when multiple unchecked items match (ambiguous)
- [x] `apm/src/cmd/spec.rs` is 50 lines or fewer after the refactor
- [x] All existing `apm spec` integration tests pass without behavior change

### Out of scope

- Adding new `apm spec` subcommands or CLI options
- Creating an `apm-serve` crate or HTTP API layer
- Changing `TicketDocument` fields or the document serialization format
- Moving section-name validation logic (KNOWN_SECTIONS / config check) — this stays in spec.rs as CLI argument validation

### Approach

1. Create `apm-core/src/spec.rs` with the following public functions extracted from `apm/src/cmd/spec.rs`:
   - `get_section(doc: &TicketDocument, name: &str) -> Option<String>` — returns a section's content as a String (problem, checklist, etc.); replaces `print_section` but returns instead of printing
   - `set_section(doc: &mut TicketDocument, name: &str, value: String)` — sets a named doc field case-insensitively; merges `set_section` (line 370, exact match) and `set_section_doc` (line 188, lowercase) into one unified function
   - `apply_section_type(type_: &SectionType, value: String) -> String` — formats content per SectionType; moved verbatim
   - `mark_item(content: &str, section: &str, item_text: &str) -> Result<String>` — checks an item by text match in raw doc content; moved verbatim
   - `get_section_body(body: &str, name: &str) -> Option<String>` — extracts a custom (non-doc-field) section from raw body text; extracted from `print_section_body`
   - `set_section_body(body: &mut String, name: &str, value: &str)` — updates a custom section in raw body text; moved verbatim

2. Register the module: add `pub mod spec;` to `apm-core/src/lib.rs`.

3. Rewrite `apm/src/cmd/spec.rs` as a thin CLI wrapper (~50 lines):
   - Keep arg validation (`--set requires --section`, `--mark requires --section`)
   - Keep config loading, branch resolution, git read/write
   - Keep section-name validation against KNOWN_SECTIONS or config (this is CLI-level input validation)
   - Delegate all document manipulation to `apm_core::spec::*`
   - Keep print-to-stdout calls (these are CLI responsibilities)

4. Add unit tests in `apm-core/src/spec.rs` covering each public function (happy path + error cases), matching the acceptance criteria above.

5. Run `cargo test --workspace` — all tests must pass.

**Key gotcha:** two `set_section` variants currently exist in spec.rs with subtly different matching (exact vs. case-insensitive). The merged `apm_core::spec::set_section` must use case-insensitive matching to be consistent and to handle the "Amendment requests" field that was only handled by the case-insensitive path.

### Open questions



### Amendment requests



## History

| When | From | To | By |
|------|------|----|----|
| 2026-03-30T14:27Z | — | new | claude-0330-0245-main |
| 2026-03-30T16:31Z | new | in_design | philippepascal |
| 2026-03-30T16:35Z | in_design | specd | claude-0330-1645-spec5 |
| 2026-03-30T16:57Z | specd | ready | philippepascal |
| 2026-03-30T17:20Z | ready | in_progress | philippepascal |
| 2026-03-30T17:29Z | in_progress | implemented | claude-0330-1730-w5ec9 |
| 2026-03-30T18:04Z | implemented | accepted | philippepascal |
| 2026-03-30T18:08Z | accepted | closed | apm-sync |