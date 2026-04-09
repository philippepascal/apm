+++
id = 55
title = "apm spec: write and check spec sections without direct file editing"
state = "closed"
priority = 4
effort = 5
risk = 3
author = "claude-0329-1200-a1b2"
agent = "claude-0329-main"
branch = "ticket/0055-apm-spec-write-and-check-spec-sections-w"
created_at = "2026-03-29T19:11:46.489066Z"
updated_at = "2026-03-30T02:02:46.501095Z"
+++

## Spec

### Problem

Agents currently have no CLI interface to read or write individual sections of
a ticket's spec. The only way to update a section is to provision a worktree,
find the ticket file, parse the markdown, edit it in place, and commit manually.
This is verbose, brittle, and requires the agent to understand the internal
markdown layout. It also means that checking whether required sections are
filled requires the same round-trip.

A dedicated `apm spec` command fixes this by exposing section-level reads and
writes as first-class CLI operations, and provides a `--check` mode that
validates required sections programmatically.

### Acceptance criteria

- [x] `apm spec 42` pretty-prints all `### ...` sections of ticket #42 with
      the section name as a header and its content below.
- [x] `apm spec 42 --section Problem` prints only the body of the Problem
      section and exits 0.
- [x] `apm spec 42 --section Problem --set "New problem text"` replaces the
      Problem section body with the given text, commits to the ticket branch
      (via `git::commit_to_branch`), and prints a confirmation line.
- [x] `apm spec 42 --section Problem --set -` reads replacement text from
      stdin (useful for piping multi-line content without shell quoting issues).
- [x] `apm spec 42 --check` exits 1 and prints the failing section names when
      any required section is empty or Acceptance criteria has no items.
- [x] `apm spec 42 --check` exits 0 and prints a success message when all
      required sections are non-empty and Acceptance criteria has at least
      one item.
- [x] `apm spec 42 --section NonExistent` exits non-zero and prints an error
      naming the unknown section.
- [x] `apm spec 999` (non-existent ticket) exits non-zero and prints
      "ticket #999 not found".
- [x] `--section` without `--set` is read-only and makes no commit.
- [x] `--set` without `--section` is reported as a usage error.

### Out of scope

- Interactive editor (that is `apm review`)
- Section reordering or renaming
- Creating new section types at runtime
- Modifying individual checklist items within a section
- Supporting section names beyond the known TicketDocument fields (Problem,
  Acceptance criteria, Out of scope, Approach, Open questions)

### Approach

**New file:** `apm/src/cmd/spec.rs`

**Clap subcommand** added to the `Command` enum in `apm/src/main.rs`:

```
Spec {
    id: u32,
    #[arg(long)]
    section: Option<String>,
    #[arg(long)]
    set: Option<String>,
    #[arg(long)]
    check: bool,
}
```

**Reading** follows the same pattern as `show.rs`: resolve the branch from the
ticket ID, call `git::read_from_branch`, parse with `Ticket::parse`, then call
`t.document()` to get a `TicketDocument`. No worktree or filesystem access is
needed for reads.

**Section lookup** matches against the known field names of `TicketDocument`:
`"Problem"`, `"Acceptance criteria"`, `"Out of scope"`, `"Approach"`,
`"Open questions"`. Any other name is an error that also prints the list of
valid names.

**Writing** (`--set`): update the appropriate `TicketDocument` field with the
new text, call `doc.serialize()` to regenerate the body, replace `ticket.body`
with it, then call `ticket.serialize()` to get the full file content, and
finally call `git::commit_to_branch`. Commit message:
`ticket(<id>): set section <name>`.

**Stdin support** for `--set -`: drain stdin with
`std::io::read_to_string(std::io::stdin())`.

**`--check` mode**: call `doc.validate()` (already exists on `TicketDocument`).
If the returned `Vec` is empty, print "all required sections present" and exit 0.
If non-empty, print each `ValidationError` via its `Display` impl, then call
`std::process::exit(1)`.

No changes to `apm-core` are required. `TicketDocument` already parses,
serializes, and validates all required sections.

**Tests**: integration tests in `apm/tests/integration.rs` covering each
acceptance criterion using temp git repos (following the existing pattern).

## History

| When | From | To | By |
|------|------|----|----|
| 2026-03-29T19:11Z | — | new | claude-0329-1200-a1b2 |
| 2026-03-29T20:36Z | new | in_design | claude-spec-55 |
| 2026-03-29T20:38Z | in_design | specd | claude-spec-55 |
| 2026-03-29T20:49Z | specd | ready | claude-0329-main |
| 2026-03-29T20:49Z | ready | in_progress | claude-0329-main |
| 2026-03-29T20:53Z | in_progress | implemented | claude-0329-main |
| 2026-03-29T22:47Z | implemented | accepted | apm |
| 2026-03-30T02:02Z | accepted | closed | apm-sync |