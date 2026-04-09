+++
id = "19c2ab13"
title = "Add --epic flag to apm new command"
state = "closed"
priority = 6
effort = 4
risk = 3
author = "claude-0401-2145-a8f3"
agent = "48176"
branch = "ticket/19c2ab13-add-epic-flag-to-apm-new-command"
created_at = "2026-04-01T21:55:26.992429Z"
updated_at = "2026-04-02T19:06:29.209537Z"
+++

## Spec

### Problem

Currently `apm new` always creates ticket branches from `main` (or the default branch) and writes no epic-related fields to frontmatter. For tickets that belong to an epic, the ticket branch must instead be created from the epic branch tip, and the frontmatter must carry `epic` and `target_branch` so that downstream commands (`apm start`, PR creation) know where to target. An optional `depends_on` field lets a ticket declare that it must not be dispatched until listed tickets are implemented.

The full design is in `docs/epics.md`. This ticket adds the `--epic <id>` flag (and `--depends-on`) to `apm new`. Without the flag, `apm new` behaviour is completely unchanged.

### Acceptance criteria

- [x] `apm new --epic <id> "Title"` exits non-zero with a clear error message when no `epic/<id>-*` branch exists on origin or locally
- [x] `apm new --epic <id> "Title"` creates a ticket whose frontmatter contains `epic = "<id>"`
- [x] `apm new --epic <id> "Title"` creates a ticket whose frontmatter contains `target_branch = "epic/<id>-<slug>"` (the full resolved branch name)
- [x] `apm new --epic <id> "Title"` creates the ticket branch from the tip of the epic branch, not from `main`
- [x] `apm new --epic <id> --depends-on <id1>,<id2> "Title"` creates a ticket whose frontmatter contains `depends_on = ["<id1>", "<id2>"]`
- [x] `apm new --depends-on <id1> "Title"` (no `--epic`) creates a ticket with `depends_on` set and no `epic` or `target_branch` fields
- [x] `apm new "Title"` (no epic flags) behaves exactly as before: branch from `main`, no `epic`/`target_branch`/`depends_on` fields in frontmatter
- [x] Existing tickets without the new fields continue to parse and round-trip without errors

### Out of scope

- `apm epic new` command (creating an epic branch) — separate ticket
- `apm epic list`, `apm epic show`, `apm epic close` commands
- `apm start` using `target_branch` to provision the worktree from the epic branch tip
- PR creation targeting `target_branch` instead of `main`
- `apm work --epic` exclusive scheduling mode
- `depends_on` dispatch blocking in the engine loop
- UI additions (epic column, lock icons, filter dropdowns)
- apm-server API changes (`/api/epics` routes, `CreateTicketRequest` additions)
- Moving a ticket into or out of an epic after creation

### Approach

**1. `apm-core/src/ticket.rs` — Frontmatter struct**

`epic`, `target_branch`, and `depends_on` are already present on `main` (landed via d877bd37). Skip this step.

**2. `apm-core/src/ticket.rs` — `create()` function**

Add parameters (or extend the options struct if one exists) for the three new fields. When `epic`/`target_branch` are `Some`, populate them in the `Frontmatter` being constructed. Pass `base_branch` through to the git layer.

**3. `apm-core/src/git.rs` — branch creation from base**

`commit_to_branch()` (and its inner `try_worktree_commit()`) currently branches from `HEAD`. Add an optional `base_branch: Option<&str>` parameter. When `Some(b)`:

- Resolve the base to a SHA via `git rev-parse origin/<b>` (fall back to local `<b>` if origin doesn't have it)
- Pass that SHA as the start point to `git worktree add -b <ticket-branch> <path> <sha>`

When `None`, existing behaviour (branch from `HEAD`) is preserved.

**4. `apm/src/main.rs` — CLI flags**

Add to the `New` subcommand variant:

```
--epic <ID>          Short epic ID (8 hex chars); resolves epic/<ID>-* branch
--depends-on <IDS>   Comma-separated ticket IDs (repeatable flag also acceptable)
```

**5. `apm/src/cmd/new.rs` — flag handling**

In `run()`:

1. If `--epic <id>` is given:
   - Run `git ls-remote origin 'refs/heads/epic/<id>-*'` to find the full branch name
   - If no match: print error (`"No epic branch found for id '<id>'"`) and exit non-zero
   - Set `target_branch = <full-branch-name>`, `epic = <id>`
2. Parse `--depends-on` (split on commas, strip whitespace)
3. Pass all three fields to `ticket::create()`

**6. Tests**

Unit (inline in `apm-core/src/ticket.rs`): Frontmatter with new fields round-trips through TOML serialize/deserialize; Frontmatter without new fields (legacy) parses without error. These tests already exist on main (d877bd37) — skip if present.

Integration (`apm/tests/integration.rs`): Create a temp git repo with an epic branch; `apm new --epic <id>` produces a ticket with correct frontmatter and a branch whose first parent is the epic branch tip; `apm new` without `--epic` still branches from `main` and has no epic fields; `apm new --epic <bad-id>` exits non-zero.

### 1. `apm-core/src/ticket.rs` — Frontmatter struct

Add three new optional fields with `#[serde(skip_serializing_if = "Option::is_none")]` so they are omitted from TOML when absent (backward-compatible):

```rust
pub epic: Option<String>,
pub target_branch: Option<String>,
pub depends_on: Option<Vec<String>>,
```

### 2. `apm-core/src/ticket.rs` — `create()` function

Add parameters (or extend the options struct if one exists) for the three new fields. When `epic`/`target_branch` are `Some`, populate them in the `Frontmatter` being constructed. Pass `base_branch` through to the git layer.

### 3. `apm-core/src/git.rs` — branch creation from base

`commit_to_branch()` (and its inner `try_worktree_commit()`) currently branches from `HEAD`. Add an optional `base_branch: Option<&str>` parameter. When `Some(b)`:

- Resolve the base to a SHA via `git rev-parse origin/<b>` (fall back to local `<b>` if origin doesn't have it)
- Pass that SHA as the start point to `git worktree add -b <ticket-branch> <path> <sha>`

When `None`, existing behaviour (branch from `HEAD`) is preserved.

### 4. `apm/src/main.rs` — CLI flags

Add to the `New` subcommand variant:

```
--epic <ID>          Short epic ID (8 hex chars); resolves epic/<ID>-* branch
--depends-on <IDS>   Comma-separated ticket IDs (repeatable flag also acceptable)
```

### 5. `apm/src/cmd/new.rs` — flag handling

In `run()`:

1. If `--epic <id>` is given:
   - Run `git ls-remote origin 'refs/heads/epic/<id>-*'` to find the full branch name
   - If no match: print error (`"No epic branch found for id '<id>'"`) and exit non-zero
   - Set `target_branch = <full-branch-name>`, `epic = <id>`
2. Parse `--depends-on` (split on commas, strip whitespace)
3. Pass all three fields to `ticket::create()`

### 6. Tests

**Unit (inline in apm-core/src/ticket.rs)**:
- Frontmatter with new fields round-trips through TOML serialize/deserialize
- Frontmatter without new fields (legacy) parses without error

**Integration (apm/tests/integration.rs)**:
- Create a temp git repo with an epic branch; `apm new --epic <id>` produces a ticket with correct frontmatter and a branch whose first parent is the epic branch tip
- `apm new` without `--epic` still branches from `main` and has no epic fields
- `apm new --epic <bad-id>` exits non-zero

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-04-01T21:55Z | — | new | claude-0401-2145-a8f3 |
| 2026-04-01T21:59Z | new | groomed | claude-0401-2145-a8f3 |
| 2026-04-02T00:49Z | groomed | in_design | philippepascal |
| 2026-04-02T00:52Z | in_design | specd | claude-0401-2200-sp01 |
| 2026-04-02T02:28Z | specd | ready | apm |
| 2026-04-02T06:16Z | ready | in_progress | philippepascal |
| 2026-04-02T06:26Z | in_progress | implemented | claude-0402-0620-b7c4 |
| 2026-04-02T19:06Z | implemented | closed | apm-sync |