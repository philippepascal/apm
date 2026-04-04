+++
id = "553ec190"
title = "apm list: add --mine and --author flags for filtering by collaborator"
state = "closed"
priority = 0
effort = 2
risk = 1
author = "apm"
branch = "ticket/553ec190-apm-list-add-mine-and-author-flags-for-f"
created_at = "2026-04-02T20:54:04.874772Z"
updated_at = "2026-04-04T06:20:21.985906Z"
epic = "8db73240"
target_branch = "epic/8db73240-user-mgmt"
depends_on = ["610be42e"]
+++

## Spec

### Problem

There is no way to filter `apm list` output by ticket author. A developer working on a shared project has to scan all tickets to find their own. `apm list --mine` and `apm list --author <username>` are the intended daily-driver filters. See `initial_specs/DESIGN-users.md` point 7.

### Acceptance criteria

- [x] `apm list --mine` shows only tickets where `author` matches the current user identity resolved via `identity::resolve_current_user`
- [x] `apm list --mine` when `.apm/local.toml` is absent (identity resolves to `"apm"`) shows only tickets where `author == "apm"`
- [x] `apm list --author alice` shows only tickets where `author == "alice"`
- [x] `apm list --author alice` with no matching tickets prints no output and exits 0
- [x] `--mine` and `--author` are mutually exclusive: passing both produces an error and non-zero exit code
- [x] `apm list --mine --state ready` shows only tickets matching both the author and state filters (AND logic)
- [x] `apm list --author <username> --state <state>` combines with all other existing filters (AND logic)
- [x] All existing `apm list` filters continue to work unchanged after this addition

### Out of scope

- `apm epic list --mine` — epic author filtering (separate ticket per DESIGN-users.md point 7)
- UI/server author filter changes (`/api/me` endpoint, board author dropdown) — point 8
- Git host plugin identity resolution — point 4
- `apm init` prompting for username and writing `.apm/local.toml`
- Validating `--author` value against the collaborators list
- `apm list --unassigned` semantics change — covered by the dependency ticket 610be42e

### Approach

This ticket sits on top of 610be42e which adds `apm_core::identity::resolve_current_user`. All four changes are mechanical.

**1. `apm-core/src/ticket.rs` — extend `list_filtered`**

Add one parameter: `author_filter: Option<&str>`.

Inside the filter closure add a predicate:
  `let author_ok = author_filter.map_or(true, |a| fm.author.as_deref() == Some(a));`
Include `author_ok` in the final boolean conjunction.
Update every direct `list_filtered` call in unit tests to pass `None` as the new last argument.

**2. `apm/src/cmd/list.rs` — wire up the new parameter**

Change the `run` signature to accept `mine: bool` and `author: Option<String>`.
Resolve the effective author filter before calling `list_filtered`:
  if mine is true  -> call identity::resolve_current_user(root), use result as author_filter
  if author is set -> use it as author_filter
  else             -> None
Pass `author_filter.as_deref()` to `list_filtered`.

**3. `apm/src/main.rs` — add CLI flags**

Inside the `List` variant add:
- `#[arg(long)] mine: bool` — show only tickets authored by the current user
- `#[arg(long, value_name = "USERNAME", conflicts_with = "mine")] author: Option<String>`

`conflicts_with = "mine"` gives Clap-level mutual exclusion with an automatic error message.
Update the `Command::List { ... }` match arm to forward the two new fields to `cmd::list::run`.
Add examples to the long-about string: `apm list --mine` and `apm list --author alice`.

**4. Tests**

Unit tests in `apm-core/src/ticket.rs`:
- `list_filtered_by_author`: two tickets with different author values; assert only the matching one is returned.
- `list_filtered_author_none`: author_filter = None returns all tickets regardless of author.

If an integration test for `apm list` exists in `apm/tests/integration.rs`, extend it to cover `--mine` by writing a `.apm/local.toml` with a test username and asserting only matching tickets appear.

**Order of steps**
1. Update `list_filtered` + unit tests in ticket.rs
2. Update `cmd/list.rs` run signature + author resolution
3. Update `main.rs` CLI struct + match arm
4. `cargo test --workspace`

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-04-02T20:54Z | — | new | apm |
| 2026-04-02T23:22Z | new | groomed | apm |
| 2026-04-02T23:39Z | groomed | in_design | philippepascal |
| 2026-04-02T23:42Z | in_design | specd | claude-0402-2340-s7w2 |
| 2026-04-04T00:29Z | specd | ready | apm |
| 2026-04-04T02:22Z | ready | in_progress | philippepascal |
| 2026-04-04T02:30Z | in_progress | implemented | claude-0403-1422-w7k9 |
| 2026-04-04T06:20Z | implemented | closed | apm-sync |
