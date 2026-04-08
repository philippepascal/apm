+++
id = "ba4e8499"
title = "Add epic and depends_on fields to CreateTicketRequest and ticket API responses"
state = "closed"
priority = 8
effort = 4
risk = 2
author = "claude-0401-2145-a8f3"
agent = "46476"
branch = "ticket/ba4e8499-add-epic-and-depends-on-fields-to-create"
created_at = "2026-04-01T21:55:57.801343Z"
updated_at = "2026-04-02T19:07:40.307498Z"
+++

## Spec

### Problem

The `CreateTicketRequest` struct in `apm-server/src/main.rs` accepts only `title` and `sections`. It has no `epic` or `depends_on` fields, so the UI cannot create epic-linked or dependency-declared tickets via the API.

The `Frontmatter` struct in `apm-core/src/ticket.rs` also has no `epic`, `target_branch`, or `depends_on` fields. Because `TicketResponse` and `TicketDetailResponse` both serialize frontmatter via `#[serde(flatten)]`, adding these fields to `Frontmatter` is sufficient to make them appear in all existing ticket API read responses — no struct changes are needed in `apm-server`.

The `ticket::create` function must also be extended to accept and persist these three optional fields so the server (and the CLI in a future ticket) can populate them at creation time.

### Acceptance criteria

- [x] `GET /api/tickets` response includes `epic`, `target_branch`, and `depends_on` keys for a ticket that has those frontmatter fields set
- [x] `GET /api/tickets/:id` response includes `epic`, `target_branch`, and `depends_on` keys for a ticket that has those frontmatter fields set
- [x] `GET /api/tickets` response omits `epic`, `target_branch`, and `depends_on` for a ticket that does not have those fields (keys absent, not null)
- [x] `POST /api/tickets` with `{"title": "T", "depends_on": ["ab12cd34"]}` creates a ticket whose frontmatter contains `depends_on = ["ab12cd34"]`
- [x] `POST /api/tickets` with `{"title": "T", "epic": "ab12cd34"}` where branch `epic/ab12cd34-some-slug` exists creates a ticket with `epic = "ab12cd34"` and `target_branch = "epic/ab12cd34-some-slug"` in frontmatter
- [x] `POST /api/tickets` with `{"title": "T", "epic": "ab12cd34"}` where no matching epic branch exists returns HTTP 400
- [x] `POST /api/tickets` response body includes `epic`, `target_branch`, and `depends_on` when those fields were set
- [x] Existing `POST /api/tickets` calls with no `epic` or `depends_on` fields continue to work unchanged

### Out of scope

- Epic CRUD routes (`GET /api/epics`, `POST /api/epics`, `GET /api/epics/:id`) — separate ticket
- `apm epic new` / `apm epic list` / `apm epic show` CLI commands — separate ticket
- `apm new --epic` CLI flag — separate ticket
- `depends_on` scheduling in the engine loop — separate ticket
- UI changes (epic dropdown in new-ticket modal, lock icons on cards, epic filter) — separate ticket
- `POST /api/work/start` epic filter field — separate ticket
- `apm start` worktree provisioning from `target_branch` — separate ticket

### Approach

Step 1 - apm-core/src/ticket.rs - extend Frontmatter

Add three optional fields to Frontmatter with skip_serializing_if Option::is_none:
  pub epic: Option<String>
  pub target_branch: Option<String>
  pub depends_on: Option<Vec<String>>

Because TicketResponse and TicketDetailResponse both use #[serde(flatten)] frontmatter: Frontmatter,
these fields appear automatically in all existing read-path API responses.
No server struct changes needed for reads.

Step 2 - apm-core/src/ticket.rs - extend ticket::create

Add three new optional parameters at the end of the signature:
  epic: Option<String>
  target_branch: Option<String>
  depends_on: Option<Vec<String>>

Set them in the Frontmatter literal constructed inside create.

Update all existing call sites to pass None, None, None for the new params:
- apm-server/src/main.rs (handler at line ~533, test helper at line ~1142)
- apm-core/tests/ticket_create.rs (four call sites)
- apm/src/cmd/new.rs (one call site)

Step 3 - apm-server/src/main.rs - extend CreateTicketRequest

Add fields:
  epic: Option<String>
  depends_on: Option<Vec<String>>

Step 4 - apm-server/src/main.rs - update create_ticket handler

Before the spawn_blocking closure, if req.epic is Some(short_id):
1. Scan local branches with git branch --list epic/short_id-* and remote branches
   with git branch -r --list origin/epic/short_id-* (stripping origin/ prefix),
   using apm_core::git::run or equivalent.
2. If no branch is found, return HTTP 400 with error message immediately.
3. Pass resolved epic (short ID), target_branch (full branch name), and depends_on
   values through to ticket::create.

Keep the branch resolution in a small helper:
  fn find_epic_branch(root: &Path, short_id: &str) -> Option<String>

Step 5 - Tests

Add to the inline tests in apm-server/src/main.rs:

- create_ticket_with_depends_on_persists_to_git: git_setup, post with title and
  depends_on array, read back ticket branch content, assert depends_on in frontmatter TOML.
- create_ticket_with_unknown_epic_returns_400: git_setup, post with epic ID that
  has no matching branch, assert HTTP 400.
- create_ticket_with_epic_resolves_target_branch: git_setup, create branch
  epic/ab12cd34-foo locally (empty commit), post with title and epic ab12cd34,
  assert response JSON contains epic and target_branch fields with correct values.

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-04-01T21:55Z | — | new | claude-0401-2145-a8f3 |
| 2026-04-01T22:00Z | new | groomed | claude-0401-2145-a8f3 |
| 2026-04-02T00:43Z | groomed | in_design | philippepascal |
| 2026-04-02T00:47Z | in_design | specd | claude-0402-0050-spec1 |
| 2026-04-02T02:28Z | specd | ready | apm |
| 2026-04-02T05:46Z | ready | in_progress | philippepascal |
| 2026-04-02T05:50Z | in_progress | implemented | claude-0401-2145-impl1 |
| 2026-04-02T19:07Z | implemented | closed | apm-sync |