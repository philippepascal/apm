# APM — Ticket Document Specification

> Defines the structure, sections, and formatting rules for APM ticket files.
> This is the canonical reference for what a ticket document contains and how
> each section is used.

---

## Overview

A ticket is a single markdown file with TOML frontmatter. It is the complete
record of a unit of work: what needs to be done, the questions asked along the
way, the agreed approach, and the state history.

The **state** signals whose turn it is. The **spec** contains the structured
content. Questions and amendment requests live inside the spec itself, in
defined subsections.

Each ticket has its own git branch from creation. The ticket file is written
to that branch throughout the ticket's lifecycle. Once the branch is merged
into `main`, the file becomes a permanent tracked file on `main` — it is no
longer ephemeral cache.

---

## File format

```
tickets/<id>-<slug>.md
```

- `id`: zero-padded 4-digit integer (`0001`, `0042`)
- `slug`: derived from title at creation — lowercase, hyphens, max 40 chars; never changes even if title changes

The file lives at this path on the ticket's branch until the branch is merged
into `main`, at which point it becomes a permanently tracked file on `main`.
The local `tickets/` directory therefore contains two kinds of files:

- **Tracked (merged tickets):** committed to `main` via PR merge; permanent;
  survive branch deletion; `apm sync` does not touch them.
- **Untracked (open tickets):** written by `apm sync` from the ticket branch;
  ephemeral cache; gitignored on `main`; pruned by `apm sync` when the branch
  disappears.

---

## Full document structure

```
+++
<TOML frontmatter>
+++

## Spec

### Problem
### Acceptance criteria
### Out of scope
### Open questions        ← optional; present when questions exist
### Amendment requests    ← optional; present when amendments requested
### Approach

## History
```

No other top-level sections. The spec contains everything written by humans.
The history is written only by APM.

---

## Frontmatter

TOML block delimited by `+++`. Written and maintained exclusively by APM via
`apm state`, `apm set`, `apm start`. Never edited manually.

```toml
+++
id          = 42
title       = "Add CSV export for portfolio data"
state       = "in_progress"
effort      = "medium"          # low | medium | high
risk        = "low"             # low | medium | high
priority    = 2                 # 0=none 1=urgent 2=high 3=medium 4=low
created_at  = "2026-03-25T10:00:00Z"
updated_at  = "2026-03-25T16:00:00Z"
author      = "philippe"        # set once at creation; never changes
supervisor  = "philippe"        # responsible engineer; can be reassigned
agent       = "claude-0325-a3f9"  # current worker; null until in_progress
branch      = "ticket/42-add-csv-export-for-portfolio-data"
repos       = ["org/ticker"]

[[prs]]
number      = 7
url         = "https://github.com/org/ticker/pull/7"
type        = "closes"          # closes | refs
state       = "open"            # open | merged | closed
review      = "approved"        # "" | review_requested | changes_requested | approved
+++
```

### Frontmatter field reference

| Field | Required | Set by | Notes |
|-------|----------|--------|-------|
| `id` | yes | APM on create | From `apm/meta` NEXT_ID; never changes |
| `title` | yes | creator | Can be updated; slug does not change |
| `state` | yes | APM | Must match a state id in `apm.toml` |
| `effort` | no | anyone | `low` / `medium` / `high` |
| `risk` | no | anyone | `low` / `medium` / `high` |
| `priority` | no | anyone | Integer 0–4 |
| `created_at` | yes | APM on create | RFC 3339; never changes |
| `updated_at` | yes | APM | Updated on every frontmatter write |
| `author` | yes | APM on create | Identity of creator; never changes |
| `supervisor` | no | creator or `apm set` | Engineer responsible; can be reassigned |
| `agent` | no | `apm start` / `apm take` | Current worker; cleared on rollback |
| `branch` | no | `apm start` | Set when implementation begins; cleared on rollback |
| `repos` | no | creator | Code repos this ticket touches |
| `prs` | no | APM via provider | Array of PR records; see below |

### PR record fields

| Field | Values | Notes |
|-------|--------|-------|
| `number` | integer | PR number in the provider |
| `url` | string | Full URL |
| `type` | `closes` \| `refs` | Closing PRs drive `implemented → accepted`; refs do not |
| `state` | `open` \| `merged` \| `closed` | Updated by `apm sync` |
| `review` | `""` \| `review_requested` \| `changes_requested` \| `approved` | Updated by `apm sync` via provider API |

---

## `## Spec` section

The core human-written content. Written by the agent, refined through the
question and amendment cycle with the supervisor.

All spec content lives on the ticket's branch. APM's `apm spec <id>` command
opens the file in `$EDITOR` on the correct branch. Direct file editing works
but bypasses APM's branch routing.

### Required subsections

All four must be present and non-empty before the ticket can move to `specd`.

---

#### `### Problem`

What is broken, missing, or needed — and why it matters. Written in prose.
One or two paragraphs. Should be understandable by someone unfamiliar with
the codebase.

```markdown
### Problem
Users cannot download their portfolio history as CSV. They must manually copy
values from the table view. The ticker app has all the data; it just needs an
export endpoint and a download trigger in the UI.
```

---

#### `### Acceptance criteria`

A checklist of independently testable outcomes. Each item is a checkbox.
The agent checks items off as they are verified during implementation.

```markdown
### Acceptance criteria
- [ ] GET /portfolio/export returns 200 with Content-Type: text/csv
- [ ] CSV includes headers: date, ticker, quantity, price, unrealized_gain_pct
- [ ] Date range filter works via ?from=YYYY-MM-DD&to=YYYY-MM-DD
- [ ] Empty portfolio returns valid CSV with headers only
- [ ] Filename in Content-Disposition is portfolio_YYYY-MM-DD.csv
```

**Rules:**
- Each criterion must be independently verifiable — no compound "and" criteria
- Do not check a box until the implementation is verified against it
- Do not remove or reword criteria once the supervisor has approved the spec; add new ones instead
- All boxes must be checked before the ticket can move to `accepted`

APM precondition `spec_all_criteria_checked` verifies this before `implemented → accepted`.

---

#### `### Out of scope`

An explicit list of what this ticket does not cover. Prevents scope creep and
reduces amendment cycles. Written as a flat list.

```markdown
### Out of scope
- PDF export format
- Exporting data from multiple portfolios in one file
```

If nothing is explicitly out of scope, write "None identified." Do not omit
the section.

---

#### `### Approach`

How the implementation will work. Written after open questions are resolved.
Updated when amendments are addressed. Should be specific enough that a
different agent could implement from it.

```markdown
### Approach
Add a `GET /portfolio/export` route in `src/routes/portfolio.rs`. Query both
the `transactions` and `positions` tables. Stream CSV using the `csv` crate
to avoid buffering the full result. Use `Content-Disposition: attachment` with
a date-stamped filename. Date range filter is optional; default to all-time.
```

---

### Optional subsections

---

#### `### Open questions`

Questions from the agent that require supervisor input before the spec or
implementation can proceed. Present only when there are open or resolved
questions. The section remains in the document after questions are answered —
it is a permanent record of decisions made.

**Format:**

```markdown
### Open questions

**Q (claude-0325-a3f9, 2026-03-25):** Should the CSV include unrealized gains
from open positions, or only realized transactions? The data model has both in
separate tables.

**A (philippe, 2026-03-25):** Include both. Add an `unrealized_gain_pct` column
sourced from the `positions` table. Date range filter applies to transactions;
open positions are always included regardless of date range.
```

**Rules:**
- Agent writes the question, then `apm state N question` (or `apm ask N "..."`)
- Supervisor writes the answer directly in this section, then changes state back
- Unanswered questions (no `**A**` line following a `**Q**` line) are detected by `apm verify`
- Do not delete answered questions — they are the decision record
- The Approach section should be updated to reflect decisions from answered questions

**Workflow:**
```
agent writes question in ### Open questions
→ apm state N question        (signals: supervisor has the ball)

supervisor writes answer in ### Open questions
→ apm state N <prior-state>   (signals: agent has the ball again)
```

Both the question text and the state change commit to the ticket branch.

---

#### `### Amendment requests`

Changes to the spec requested by the supervisor during spec review. The
supervisor writes items here, moves the ticket to `ammend`. The agent
addresses each item, checks it off, and moves back to `specd`.

**Format:**

```markdown
### Amendment requests

- [x] Clarify whether the date range filter is inclusive or exclusive on both
      ends. (philippe, 2026-03-26)
- [ ] Add an acceptance criterion for the case where `from` > `to` — should
      return 400, not an empty CSV. (philippe, 2026-03-26)
```

**Rules:**
- Supervisor writes items here, then `apm state N ammend`
- Each item is a checkbox the agent checks off as it is addressed in the spec
- The agent updates the Approach section (and other sections as needed) to reflect each addressed amendment
- Agent moves back to `specd` only when all boxes are checked (`spec_all_amendments_addressed` precondition)
- Do not remove items once checked — they are the amendment history
- New amendment rounds append new items below existing ones

**Workflow:**
```
supervisor writes items in ### Amendment requests
→ apm state N ammend          (signals: agent has the ball)

agent addresses items, checks boxes, updates spec
→ apm state N specd           (signals: supervisor has the ball again)
```

All edits commit to the `ticket/<id>-<slug>` branch (pre-`in_progress`).

---

## `## History`

Append-only log of state transitions. Written exclusively by APM. Never
edited manually.

```markdown
## History

| When | From | To | By |
|------|------|----|----|
| 2026-03-25T10:00Z | — | new | philippe |
| 2026-03-25T11:15Z | new | question | claude-0325-a3f9 |
| 2026-03-25T14:30Z | question | new | philippe |
| 2026-03-25T15:00Z | new | specd | claude-0325-a3f9 |
| 2026-03-25T15:45Z | specd | ammend | philippe |
| 2026-03-26T09:00Z | ammend | specd | claude-0325-a3f9 |
| 2026-03-26T09:30Z | specd | ready | philippe |
| 2026-03-26T10:00Z | ready | in_progress | claude-0325-a3f9 |
```

**Rules:**
- APM appends a row on every state transition
- The `By` column is the `APM_AGENT_NAME` value at the time of the transition
- The first row always has `—` in the `From` column
- Rows are never deleted or modified
- Committed to the ticket's current branch along with each state transition

---

## Branch and write routing

All ticket writes route to the ticket's current branch. APM handles this automatically.

| Phase | Where the file lives | Who writes |
|-------|----------------------|------------|
| `new` through `implemented` | `ticket/<id>-<slug>` branch | agent, supervisor, APM |
| `accepted` | `main` (arrived via PR merge) | APM (post-merge state commit) |
| `closed` | `main` | APM (`apm state N closed` commits to `main`) |

**Key rules:**
- Supervisor does not push to the feature branch after `in_progress` begins
- Supervisor feedback during implementation goes through PR review comments
- `apm spec <id>` always opens the file on the correct branch regardless of what is checked out locally
- The local `tickets/` directory is a cache for open tickets; merged ticket files are tracked by git and must not be deleted manually
- Deleting a ticket branch after the PR is merged is safe — the file is already on `main`

---

## Lifecycle: when each section is written

| Section | Written | By |
|---------|---------|-----|
| Frontmatter (initial) | `apm new` | APM |
| `### Problem` | `new` state | agent (or creator) |
| `### Acceptance criteria` | `new` state | agent |
| `### Out of scope` | `new` state | agent |
| `### Open questions` (Q) | any state, when blocked | agent |
| `### Open questions` (A) | `question` state | supervisor |
| `### Approach` | after questions resolved, before `specd` | agent |
| `### Amendment requests` | `specd` or `ammend` state | supervisor |
| `## History` | every state transition | APM |
| Frontmatter (state updates) | every state transition | APM |
| Acceptance criteria checkboxes | `in_progress` state | agent |

---

## Preconditions reference

| Precondition | What APM checks |
|-------------|-----------------|
| `spec_not_empty` | `### Problem`, `### Acceptance criteria`, `### Out of scope`, `### Approach` all exist and are non-empty |
| `spec_has_acceptance_criteria` | At least one `- [ ]` or `- [x]` checkbox in `### Acceptance criteria` |
| `spec_all_criteria_checked` | No unchecked `- [ ]` boxes in `### Acceptance criteria` |
| `spec_all_amendments_addressed` | No unchecked `- [ ]` boxes in `### Amendment requests` |
| `spec_no_open_questions` | Every `**Q ...**` line is followed by an `**A ...**` line |
| `pr_exists` | `prs` array is non-empty with at least one `closes`-type record |
| `pr_all_closing_merged` | All `closes`-type PR records have `state = "merged"` |

---

## Complete example

```markdown
+++
id          = 42
title       = "Add CSV export for portfolio data"
state       = "in_progress"
effort      = "medium"
risk        = "low"
priority    = 2
created_at  = "2026-03-25T10:00:00Z"
updated_at  = "2026-03-26T10:00:00Z"
author      = "philippe"
supervisor  = "philippe"
agent       = "claude-0325-a3f9"
branch      = "ticket/42-add-csv-export-for-portfolio-data"
repos       = ["org/ticker"]
+++

## Spec

### Problem
Users cannot download their portfolio history as CSV. They must manually copy
values from the table view. The ticker app has all the data; it just needs an
export endpoint and a download trigger in the UI.

### Acceptance criteria
- [ ] GET /portfolio/export returns 200 with Content-Type: text/csv
- [ ] CSV includes headers: date, ticker, quantity, price, unrealized_gain_pct
- [ ] Date range filter works via ?from=YYYY-MM-DD&to=YYYY-MM-DD
- [ ] Empty portfolio returns valid CSV with headers only
- [ ] Filename in Content-Disposition is portfolio_YYYY-MM-DD.csv
- [ ] Returns 400 if from > to

### Out of scope
- PDF export format
- Exporting data from multiple portfolios in one file
- System-level scheduling of exports

### Open questions

**Q (claude-0325-a3f9, 2026-03-25):** Should the CSV include unrealized gains
from open positions, or only realized transactions? The data model has both in
separate tables.

**A (philippe, 2026-03-25):** Include both. Add an `unrealized_gain_pct` column
sourced from the `positions` table. Date range filter applies to transactions;
open positions are always included.

### Amendment requests

- [x] Clarify whether date range filter is inclusive on both ends. (philippe, 2026-03-26)
- [x] Add criterion for from > to returning 400. (philippe, 2026-03-26)

### Approach
Add `GET /portfolio/export` in `src/routes/portfolio.rs`. Query `transactions`
joined with `positions`. Stream CSV using the `csv` crate. Use
`Content-Disposition: attachment; filename=portfolio_YYYY-MM-DD.csv`. Date
range is optional (default all-time), inclusive on both ends. Return 400 if
`from > to`.

## History

| When | From | To | By |
|------|------|----|----|
| 2026-03-25T10:00Z | — | new | philippe |
| 2026-03-25T11:15Z | new | question | claude-0325-a3f9 |
| 2026-03-25T14:30Z | question | new | philippe |
| 2026-03-25T15:00Z | new | specd | claude-0325-a3f9 |
| 2026-03-25T15:45Z | specd | ammend | philippe |
| 2026-03-26T09:00Z | ammend | specd | claude-0325-a3f9 |
| 2026-03-26T09:30Z | specd | ready | philippe |
| 2026-03-26T10:00Z | ready | in_progress | claude-0325-a3f9 |
```
