+++
id = 32
title = "apm-ticket-format-in-core"
state = "closed"
priority = 0
effort = 5
risk = 3
author = "apm"
agent = "claude-0327-1757-391b"
branch = "ticket/0032-apm-ticket-format-in-core"
created_at = "2026-03-27T20:49:59.747092Z"
updated_at = "2026-03-30T02:02:46.501095Z"
+++

## Spec

### Problem

The ticket document body is stored and manipulated as a raw `String` in `apm-core`. The spec body sections (`### Problem`, `### Acceptance criteria`, etc.) are matched with ad-hoc `str::contains` checks. This means malformed tickets — missing required sections, empty sections, improperly formatted checklists — are never detected at parse time. Acceptance criteria checkbox state (`- [ ]` / `- [x]`) cannot be read or toggled programmatically. `apm verify` only checks that `## Spec` and `## History` substrings exist, not whether required sections are present or non-empty. The preconditions declared in `TICKET-SPEC.md` (`spec_not_empty`, `spec_has_acceptance_criteria`, `spec_all_criteria_checked`, etc.) are defined but not enforced in code at state-transition time.

### Acceptance criteria

- [ ] A `TicketDocument` struct in `apm-core` (in `ticket.rs` or a new `document.rs`) owns the parsed spec body as typed fields: `problem`, `acceptance_criteria`, `out_of_scope`, `approach`, and optional `open_questions` and `amendment_requests`
- [ ] `TicketDocument::parse(body: &str)` extracts each section; returns an error if any required section is absent
- [ ] Round-trip: `parse(doc.serialize())` reproduces the original body (modulo trailing whitespace normalization)
- [ ] Acceptance criteria items are exposed as `Vec<ChecklistItem>` where each item has `checked: bool` and `text: String`
- [ ] A `toggle_criterion(index: usize, checked: bool)` method re-serializes the document with the checkbox updated
- [ ] A `validate()` method returns `Vec<ValidationError>` covering: missing required section, empty required section, no acceptance criteria items, unanswered open questions
- [ ] `apm verify` calls `validate()` on each ticket body and reports per-ticket errors
- [ ] `apm state <id> specd` rejects the transition if required sections are absent/empty or open questions are unanswered
- [ ] `apm state <id> implemented` rejects if any acceptance criteria checkbox is unchecked
- [ ] `apm state <id> specd` (returning from `ammend`) rejects if any amendment request checkbox is unchecked
- [ ] All new logic is covered by unit tests in `apm-core/src/`

### Out of scope

- Markdown rendering or rich display formatting
- An interactive editor for ticket files
- Migration tooling for existing ticket files (backward compatibility required, not migration)
- Enforcement of criterion wording style
- PR-based preconditions (`pr_exists`, `pr_all_closing_merged`)

### Approach

Add a `TicketDocument` struct (in `apm-core/src/ticket.rs` or a new `apm-core/src/document.rs`). The struct holds the body split into named section strings, extracted by scanning for `### <Name>` headings. A `ChecklistItem { checked: bool, text: String }` type and helper methods for reading/toggling checklists live on the struct.

The `Ticket` struct gains a `document()` method that parses `self.body` into a `TicketDocument`. `validate()` returns `Vec<ValidationError>` so callers can report all issues at once.

In `apm/src/cmd/state.rs`, before writing the new state, call `t.document()?.validate()` and filter errors relevant to the transition. In `apm/src/cmd/verify.rs`, call `validate()` on every non-terminal ticket and report all errors.

Optional sections (`### Open questions`, `### Amendment requests`) are `Option<String>` fields — tickets without them parse cleanly.

## History

| When | From | To | By |
|------|------|----|----|
| 2026-03-27T20:49Z | — | new | apm |
| 2026-03-28T01:02Z | new | specd | claude-0327-1757-391b |
| 2026-03-28T01:04Z | specd | ready | apm |
| 2026-03-28T01:35Z | ready | in_progress | claude-0327-1757-391b |
| 2026-03-28T01:52Z | in_progress | implemented | claude-0327-1852-b516 |
| 2026-03-28T07:31Z | implemented | accepted | apm sync |
| 2026-03-30T02:02Z | accepted | closed | apm-sync |