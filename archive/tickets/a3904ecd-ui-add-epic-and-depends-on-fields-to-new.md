+++
id = "a3904ecd"
title = "UI: add epic and depends_on fields to new ticket modal"
state = "closed"
priority = 2
effort = 4
risk = 2
author = "claude-0401-2145-a8f3"
agent = "82968"
branch = "ticket/a3904ecd-ui-add-epic-and-depends-on-fields-to-new"
created_at = "2026-04-01T21:56:06.583740Z"
updated_at = "2026-04-02T19:07:32.007545Z"
+++

## Spec

### Problem

The new ticket modal offers only a title and four spec-section text areas. There is no way to associate a ticket with an epic or declare dependencies from the UI — users must resort to the CLI. This blocks teams that prefer managing epic-linked work through the web interface.

The full design is in `docs/epics.md` (§ apm-ui changes — New ticket modal). Two optional fields are added below the title input:
- **Epic** — dropdown populated from `GET /api/epics`; selecting one includes the short epic ID in the create payload.
- **Depends on** — free-text input for space- or comma-separated ticket IDs, stored as a `depends_on` array.

Neither `GET /api/epics` nor the `epic` / `depends_on` frontmatter fields exist yet. This ticket covers the UI modal changes plus the minimum server and core changes required to make those fields functional.

### Acceptance criteria

- [x] The new ticket modal renders an "Epic" dropdown below the title input
- [x] The Epic dropdown is populated by `GET /api/epics`; an empty response renders as a dropdown containing only a blank "(none)" option
- [x] Selecting an epic from the dropdown includes `epic: "<id>"` in the `POST /api/tickets` request body
- [x] Leaving the Epic dropdown on "(none)" omits the `epic` field from the request body
- [x] The new ticket modal renders a "Depends on" text input below the Epic dropdown
- [x] Entering ticket IDs (space- or comma-separated) in "Depends on" sends them as a `depends_on` string array in the request body
- [x] Leaving "Depends on" blank omits the `depends_on` field from the request body
- [x] Submitting with both Epic and Depends on left empty creates a ticket identical to current behaviour
- [x] The Epic and Depends on fields are reset to empty when the modal closes
- [x] `GET /api/epics` returns HTTP 200 with a JSON array of `{ id, title, branch }` objects, one per `epic/*` remote branch
- [x] `GET /api/epics` returns `[]` when no `epic/*` branches exist
- [x] `POST /api/tickets` with `epic` set writes `epic = "<id>"` in the created ticket's TOML frontmatter
- [x] `POST /api/tickets` with `depends_on` set writes `depends_on = ["...", ...]` in the created ticket's TOML frontmatter

### Out of scope

- Ticket detail panel showing epic / depends_on values (separate ticket)
- Ticket cards showing a lock icon for unresolved depends_on (separate ticket)
- Queue panel epic column and epic filter dropdown (separate ticket)
- Supervisor board epic filter (separate ticket)
- Engine controls epic selector (separate ticket)
- `apm epic new`, `apm epic list`, `apm epic show`, `apm epic close` CLI commands (separate ticket)
- Setting `target_branch` when epic is chosen — tickets remain branched from main
- Validating that a submitted epic ID corresponds to an existing branch
- Validating that depends_on IDs correspond to existing tickets
- The `apm new --epic` CLI flag

### Approach

Changes span three layers: core, server, UI. Apply in this order.

1. apm-core/src/ticket.rs

Add three optional fields to Frontmatter (after focus_section), each with
serde(skip_serializing_if = Option::is_none):
  pub epic: Option<String>
  pub target_branch: Option<String>
  pub depends_on: Option<Vec<String>>

Add epic: Option<String> and depends_on: Option<Vec<String>> to create() params.
Initialize both to None in the Frontmatter literal, then overwrite with passed values.

2. apm/src/cmd/new.rs

Pass None, None for the new epic and depends_on params in ticket::create(). No change visible to users.

3. apm-server/src/main.rs

Extend CreateTicketRequest with: pub epic: Option<String>, pub depends_on: Option<Vec<String>>.
Pass both through to apm_core::ticket::create() in create_ticket handler.

Add EpicSummary struct (derive Serialize) with fields id: String, title: String, branch: String.

Add list_epics handler:
  - If state.git_root() is None, return Json(vec![]).
  - Otherwise run git branch -r, filter lines containing epic/, strip remote prefix.
  - Parse epic/<8-char-id>-<slug>: id = first 8 chars after epic/;
    title = remaining slug with hyphens replaced by spaces, each word title-cased.
  - Return Json(Vec<EpicSummary>).

Register .route("/api/epics", get(list_epics)) in both router builders.

4. apm-ui/src/components/NewTicketModal.tsx

Add state: epicId (string) and dependsOn (string). Reset both in the useEffect([open]) cleanup.
Fetch GET /api/epics via useQuery with initialData: [].

Render below the title field:
  - Epic <select> with a leading (none) option and one option per epic (value = id).
  - Depends on <input type=text> placeholder: e.g. ab12cd34 cd56ef78.

Extend CreateTicketData: epic?: string, depends_on?: string[].
In handleSubmit: if epicId set, assign data.epic = epicId;
if dependsOn.trim() non-empty, split on whitespace/commas, filter empty, assign data.depends_on.

Tests

- apm-core/src/ticket.rs inline test: round-trip Frontmatter with epic and depends_on
  through TOML serialize/deserialize; assert values survive.
- apm-server/src/main.rs inline tests:
    GET /api/epics returns 200 with [];
    POST /api/tickets with epic and depends_on parses cleanly (returns 501 not 400/422).

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-04-01T21:56Z | — | new | claude-0401-2145-a8f3 |
| 2026-04-01T22:00Z | new | groomed | claude-0401-2145-a8f3 |
| 2026-04-02T00:52Z | groomed | in_design | philippepascal |
| 2026-04-02T00:57Z | in_design | specd | claude-0402-0100-s9x2 |
| 2026-04-02T02:29Z | specd | ready | apm |
| 2026-04-02T06:47Z | ready | in_progress | philippepascal |
| 2026-04-02T06:52Z | in_progress | implemented | claude-0401-2200-w7k3 |
| 2026-04-02T19:07Z | implemented | closed | apm-sync |