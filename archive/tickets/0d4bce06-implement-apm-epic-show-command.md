+++
id = "0d4bce06"
title = "Implement apm epic show command"
state = "closed"
priority = 6
effort = 4
risk = 3
author = "claude-0401-2145-a8f3"
agent = "74475"
branch = "ticket/0d4bce06-implement-apm-epic-show-command"
created_at = "2026-04-01T21:55:14.006927Z"
updated_at = "2026-04-02T19:06:17.992791Z"
+++

## Spec

### Problem

Engineers and supervisors can see aggregate ticket counts via `apm epic list` (not yet implemented), but there is no way to drill into a specific epic to inspect individual ticket status, assignees, and dependency relationships. Without `apm epic show`, diagnosing blocked epics, tracking down the assigned agent for a specific ticket, or checking whether `depends_on` prerequisites have been met requires manual branch browsing.

The full design for this command is in `docs/epics.md` (§ Commands — `apm epic show`). The command accepts a short epic ID (or an unambiguous prefix) and prints: title, branch name, derived state, and a table of associated tickets with columns for ID, title, current state, assigned agent, and `depends_on` entries.

Two related pieces of infrastructure must land with this ticket because `apm epic show` depends on them and neither exists yet:
1. The `Frontmatter` struct does not have `epic`, `target_branch`, or `depends_on` fields; without the `epic` field there is no way to filter tickets by epic.
2. There is no CLI `epic` subcommand; the new `Epic { Show { ... } }` command variant and its dispatch must be added to `apm/src/main.rs`.

### Acceptance criteria

- [x] `apm epic show <id>` prints a header block with the epic title, branch name, and derived state
- [x] `apm epic show <id>` prints a table of associated tickets, one row per ticket, showing: short ID, title, current state, assigned agent (or — if none), and `depends_on` entries (or — if none)
- [x] Tickets with no `epic` frontmatter field set to the epic's ID are not shown in the table
- [x] A 4-or-more character prefix that uniquely identifies one epic branch is accepted and resolves correctly
- [x] An ambiguous prefix (matches more than one epic branch) exits non-zero and prints a list of the matching branch names
- [x] An ID or prefix that matches no epic branch exits non-zero with a clear error message
- [x] Derived state is computed from config flags only — no state name strings are hardcoded: no tickets → `empty`; any ticket whose state has neither `satisfies_deps = true` nor `terminal = true` → `in_progress`; all tickets in states with `satisfies_deps = true` or `terminal = true`, but not all terminal → `implemented`; all tickets in states with `terminal = true` → `done`
- [x] `apm epic show` with no argument prints usage and exits non-zero
- [x] Adding `epic`, `target_branch`, and `depends_on` optional fields to `Frontmatter` does not break serialisation of any existing ticket that lacks those fields (they are omitted from output when `None`)
- [x] Adding `satisfies_deps` optional boolean field to `StateConfig` does not break deserialisation of any existing `workflow.toml` that lacks it (defaults to `false`)

### Out of scope

- `apm epic list` — a separate ticket; this ticket only implements `show`
- `apm epic new` and `apm epic close` — separate tickets
- `depends_on` scheduling engine changes (blocking dispatch until deps are `implemented`) — separate ticket
- Setting `epic`, `target_branch`, or `depends_on` automatically when creating a ticket with `apm new --epic` — separate ticket
- Server API routes for epics (`GET /api/epics`, `GET /api/epics/:id`) — separate ticket
- UI additions (epic column, filter dropdown, lock icon on cards) — separate ticket
- `apm work --epic` exclusive-mode scheduling — separate ticket

### Approach

Step 1: Add `satisfies_deps` to `StateConfig` in `apm-core/src/config.rs`

Add one field after `actionable`:

  #[serde(default)]
  pub satisfies_deps: bool,

This field defaults to `false`; existing workflow.toml files that omit it continue to deserialise cleanly.

Step 2: Mark `satisfies_deps = true` on "implemented" in `.apm/workflow.toml`

Add `satisfies_deps = true` to the `[[workflow.states]]` block whose `id = "implemented"`. Also update the default workflow template in `apm-core/src/init.rs` to match.

Step 3: Extend Frontmatter in `apm-core/src/ticket.rs`

Add three optional fields after `focus_section`:

  pub epic: Option<String>,            // #[serde(skip_serializing_if = "Option::is_none")]
  pub target_branch: Option<String>,   // #[serde(skip_serializing_if = "Option::is_none")]
  pub depends_on: Option<Vec<String>>, // #[serde(skip_serializing_if = "Option::is_none")]

These are entirely additive; tickets lacking the fields deserialise to `None`.

Step 4: Add `epic_branches` to `apm-core/src/git.rs`

Mirror the existing `ticket_branches` function, listing local `epic/*` then remote `origin/epic/*` branches (stripping `origin/`), deduplicating via `HashSet`.

Step 5: New module `apm-core/src/epic.rs`

Expose via `pub mod epic` in `apm-core/src/lib.rs`.

Provide:

  pub struct EpicRef {
      pub id: String,    // 8-char hex from branch name
      pub title: String, // slug → title-cased
      pub branch: String,
  }

  pub fn parse_epic_branch(branch: &str) -> Option<EpicRef>
  - Strips "epic/" prefix, splits on first '-', title-cases the remainder.
  - Returns None if the branch does not match epic/<8-hex>-<slug>.

  pub fn resolve_epic(branches: &[String], arg: &str) -> Result<EpicRef>
  - Matches branches whose ID starts with arg.
  - 1 match → Ok; 0 matches → error "no epic matching '...'";
    2+ matches → error "ambiguous prefix '...', matches: ...".

  pub fn derive_epic_state(tickets: &[&Ticket], states: &[StateConfig]) -> &'static str
  - Builds a lookup map: state_id → &StateConfig.
  - No tickets → "empty".
  - Any ticket whose state maps to a config where both `satisfies_deps` and `terminal`
    are false → "in_progress".
  - All tickets whose state maps to `satisfies_deps = true` or `terminal = true`,
    but at least one is not terminal → "implemented".
  - All tickets whose state maps to `terminal = true` (or state not found in config) → "done".
  - Otherwise → "in_progress".
  - Uses only `StateConfig` fields (`satisfies_deps`, `terminal`); no state ID string
    comparisons.

Step 6: New CLI file `apm/src/cmd/epic.rs`

  pub fn run_show(root: &Path, id: &str, no_aggressive: bool) -> Result<()>

  1. If `!no_aggressive`: fetch origin (same helper as list.rs).
  2. `load_all_from_git` for tickets; `Config::load` for config.
  3. `epic_branches(root)` → resolve via `resolve_epic`.
  4. Filter tickets where `frontmatter.epic.as_deref() == Some(&epic.id)`.
  5. `derive_epic_state` on filtered tickets, passing `&config.workflow.states`.
  6. Print header:
       Epic:   <title>
       Branch: <branch>
       State:  <derived_state>

     Then blank line and table:
       ID        State          Agent              Title                             Depends on
       --------  -------------  -----------------  --------------------------------  ----------
       ab12cd34  in_progress    alice              Implement login                   -
       cd56ef78  ready          -                  Add OAuth                         ab12cd34

     If no tickets: print "(no tickets)".

Step 7: Wire up in `apm/src/main.rs`

Add `Epic { subcommand: EpicSubcommand }` variant to `Command` enum.
Add `EpicSubcommand` enum with `Show { id: String, no_aggressive: bool }`.
Dispatch:
  Command::Epic { subcommand } => match subcommand {
      EpicSubcommand::Show { id, no_aggressive } =>
          cmd::epic::run_show(&root, &id, no_aggressive),
  }

Step 8: Tests

Integration tests in `apm/tests/integration.rs`:
- Temp repo with `epic/ab12cd34-user-auth` branch + two tickets with `epic = "ab12cd34"` in frontmatter.
- `apm epic show ab12cd34` output contains title, branch, ticket IDs.
- `apm epic show ab12` (prefix) resolves correctly.
- `apm epic show zzzzzzz` exits non-zero.
- `derive_epic_state` returns "in_progress" when a ticket's state lacks both `satisfies_deps` and `terminal`; "implemented" when all have `satisfies_deps`; "done" when all have `terminal`.

Unit tests in `apm-core/src/epic.rs`:
- `parse_epic_branch`: valid and invalid branch names.
- `derive_epic_state`: each condition using synthetic `StateConfig` values.

### Open questions


### Amendment requests

- [x] Same as a5e1ea24: `derive_epic_state` in the Approach must not hardcode state names. Remove all hardcoded "in_design", "in_progress", "accepted", "closed", "implemented" from AC and Approach. Use `actionable`, `satisfies_deps`, and `terminal` state config flags to determine epic aggregate state. The function signature should accept `&[StateConfig]` (or pass the full config) rather than comparing state ID strings.
- [x] Update the Derived state AC items to describe the rules in terms of those config flags, not specific state names.

### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-04-01T21:55Z | — | new | claude-0401-2145-a8f3 |
| 2026-04-01T21:59Z | new | groomed | claude-0401-2145-a8f3 |
| 2026-04-02T00:47Z | groomed | in_design | philippepascal |
| 2026-04-02T00:52Z | in_design | specd | claude-0402-0050-s7w2 |
| 2026-04-02T01:37Z | specd | ammend | philippepascal |
| 2026-04-02T01:40Z | ammend | in_design | philippepascal |
| 2026-04-02T01:43Z | in_design | specd | claude-0402-0200-x9k1 |
| 2026-04-02T02:28Z | specd | ready | apm |
| 2026-04-02T06:10Z | ready | in_progress | philippepascal |
| 2026-04-02T06:16Z | in_progress | implemented | claude-0402-0615-b7r4 |
| 2026-04-02T19:06Z | implemented | closed | apm-sync |