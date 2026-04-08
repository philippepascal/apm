+++
id = "ebae68e2"
title = "apm-ui: open question and amendment request badges on ticket cards"
state = "closed"
priority = 25
effort = 2
risk = 1
author = "apm"
agent = "60571"
branch = "ticket/ebae68e2-apm-ui-open-question-and-amendment-reque"
created_at = "2026-03-31T06:13:20.438546Z"
updated_at = "2026-04-01T07:12:53.653878Z"
+++

## Spec

### Problem

Ticket summary cards in the SupervisorView swimlanes show id, title, agent, effort, and risk, but give no signal about whether a ticket is waiting on supervisor input. Specifically: a ticket in *question* state may have written questions in `### Open questions` that need reading, and a ticket in *ammend* state has unchecked checkboxes in `### Amendment requests` that the spec-writer must address. Without glanceable badges, a supervisor must open every detail panel to know whether action is required.

The desired behaviour is: when a ticket has non-empty content in its `### Open questions` section, its card shows a small "?" badge; when a ticket has one or more unchecked items (`- [ ]`) in its `### Amendment requests` section, its card shows a small "A" badge. These badges let supervisors triage at a glance without opening the detail panel.

### Acceptance criteria

- [x] A ticket card shows a question badge when the ticket `### Open questions` section contains non-whitespace content
- [x] A ticket card shows an amendment badge when the ticket `### Amendment requests` section contains at least one unchecked checkbox (`- [ ]`)
- [x] A ticket card with no open questions and no pending amendments shows neither badge
- [x] Both badges can appear simultaneously on the same card
- [x] The question and amendment badges are visually distinct from the effort and risk badges already on the card
- [x] The `GET /api/tickets` response includes `has_open_questions` and `has_pending_amendments` boolean fields derived from the ticket body
- [x] The badges update when TanStack Query refetches (no manual reload needed)

### Out of scope

- Search and filter controls on the swimlanes (covered by ticket 4ce2a53e)
- Editing open questions or amendment requests from the card (the detail/editor panels handle that)
- Resolving or marking off questions/amendments from the card
- Any new API endpoint beyond adding fields to the existing `GET /api/tickets` list response

### Approach

**Backend — `apm-server`**

The `GET /api/tickets` handler already calls `ticket::load_all_from_git()` from `apm-core` and serialises each ticket. Two new boolean fields need to be added to the JSON serialisation:

1. **`has_open_questions: bool`** — true when the `### Open questions` section of the ticket body contains at least one non-whitespace character after extracting the section text.
2. **`has_pending_amendments: bool`** — true when the `### Amendment requests` section contains at least one occurrence of the literal string `- [ ]` (an unchecked markdown checkbox).

Section extraction logic (reusable utility in `apm-core` or inline in the handler):
- Split the body on `### ` headings.
- Find the chunk that starts with the target heading name.
- Strip the heading line itself; what remains is the section body.
- For `has_open_questions`: trim the section body and check `!is_empty()`.
- For `has_pending_amendments`: check `section_body.contains("- [ ]")`.

If a section is absent from the body, treat it as empty/false.

Add these fields to the `TicketSummary` (or equivalent) struct that is returned by the list endpoint. The detail endpoint (`GET /api/tickets/:id`) may also include them for consistency but is not required by the acceptance criteria.

**Frontend — `apm-ui/src/components/supervisor/TicketCard.tsx`**

The `Ticket` TypeScript type (or interface) gains two optional boolean fields: `has_open_questions` and `has_pending_amendments`.

In the card layout, after the existing effort and risk badges, conditionally render:
- A shadcn `Badge` with variant `outline` (or a distinct colour, e.g. amber) and text `?` when `ticket.has_open_questions` is true.
- A shadcn `Badge` with variant `outline` (or a distinct colour, e.g. violet) and text `A` when `ticket.has_pending_amendments` is true.

Use `title` attributes on the badges (`"Has open questions"` / `"Has pending amendments"`) so the meaning is visible on hover.

No new files are required. Changes are confined to:
- `apm-core/src/ticket.rs` (or the module that holds the ticket struct) — add computed fields or extraction helper
- `apm-server/src/routes/tickets.rs` (or equivalent) — populate the new fields when serialising
- `apm-ui/src/components/supervisor/TicketCard.tsx` — render the badges
- `apm-ui/src/types/ticket.ts` (or wherever the TS type lives) — extend the type

### Open questions



### Amendment requests



## History

| When | From | To | By |
|------|------|----|----|
| 2026-03-31T06:13Z | — | new | apm |
| 2026-03-31T07:27Z | new | in_design | philippepascal |
| 2026-03-31T07:29Z | in_design | specd | claude-0331-0800-b7e2 |
| 2026-03-31T19:44Z | specd | ready | apm |
| 2026-04-01T06:39Z | ready | in_progress | philippepascal |
| 2026-04-01T06:45Z | in_progress | implemented | claude-0401-0639-ca90 |
| 2026-04-01T07:02Z | implemented | accepted | apm-sync |
| 2026-04-01T07:12Z | accepted | closed | apm-sync |