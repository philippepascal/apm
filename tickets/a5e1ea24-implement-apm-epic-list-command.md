+++
id = "a5e1ea24"
title = "Implement apm epic list command"
state = "in_progress"
priority = 6
effort = 4
risk = 2
author = "claude-0401-2145-a8f3"
agent = "74751"
branch = "ticket/a5e1ea24-implement-apm-epic-list-command"
created_at = "2026-04-01T21:55:09.722953Z"
updated_at = "2026-04-02T05:54:55.515881Z"
+++

## Spec

### Problem

Once epic branches exist there is no way to see them or their status at a glance. Engineers and the supervisor need to know which epics are active, how many tickets are in each state, and whether an epic is done.

The full design is in `docs/epics.md` (§ Commands — `apm epic list`). Epic state is always derived — never stored — using config-driven rules based on `StateConfig` flags, not hard-coded state ID strings: no tickets → `empty`; any ticket whose state has `actionable` containing `"agent"` → `active`; all tickets terminal (`terminal = true`) → `done`; all tickets dep-satisfied (`satisfies_deps = true`) or terminal with at least one dep-satisfied → `complete`; otherwise → `active`.

The command lists all `epic/*` remote branches and for each shows: short ID, title (from slug), derived state, and per-state ticket counts (e.g. `2 in_progress, 1 ready, 3 implemented`).

### Acceptance criteria

- [x] `apm epic list` outputs one line per `epic/*` remote branch
- [x] Each line shows the 8-char ID, the humanized title (hyphens → spaces, title-cased), derived state, and non-zero per-state ticket counts
- [x] When no `epic/*` branches exist, the command exits 0 with no output
- [x] Derived state is `empty` when no tickets reference the epic ID
- [x] Derived state is `active` when any ticket's state config has `actionable` containing `"agent"`
- [x] Derived state is `done` when all tickets have `terminal = true` in their state config
- [x] Derived state is `complete` when all tickets are dep-satisfied (`satisfies_deps = true`) or terminal, and at least one is dep-satisfied
- [ ] Derived state falls back to `active` for any other mix of states
- [ ] Ticket counts omit states with a zero count (e.g. `2 in_progress, 3 implemented`, not `2 in_progress, 0 ready, 3 implemented`)
- [ ] The command respects the aggressive-fetch setting (same behaviour as `apm list`)

### Out of scope

- `apm epic new`, `apm epic show`, `apm epic close` commands
- Adding the `target_branch` or `depends_on` fields to `Frontmatter`
- `depends_on` scheduling / engine loop changes
- apm-server epic API routes
- apm-ui epic UI additions
- `apm new --epic` flag
- `apm work --epic` exclusive mode

### Approach

Six files change: one existing config struct gains a field, two existing modules gain functions, two new files are created, and `main.rs` is wired up.

**`apm-core/src/config.rs`** — add `satisfies_deps` field to `StateConfig`:
```rust
#[serde(default)]
pub satisfies_deps: bool,
```
States that mark dependency satisfaction (e.g. `implemented`) set `satisfies_deps = true` in `apm.toml`. Existing configs that omit the field default to `false`.

**`apm-core/src/ticket.rs`** — add `epic` optional field to `Frontmatter`:
```rust
#[serde(skip_serializing_if = "Option::is_none")]
pub epic: Option<String>,
```
Existing tickets that omit the field deserialize fine (`Option` defaults to `None`).

**`apm-core/src/git.rs`** — add `epic_branches() -> Result<Vec<String>>`:
Mirror `ticket_branches()`: collect local `epic/*` + remote `origin/epic/*` (strip prefix), deduplicate, return sorted.

**`apm-core/src/epic.rs`** — new module, export from `lib.rs` as `pub mod epic`:
```rust
pub fn derive_epic_state(states: &[&StateConfig]) -> &'static str
```
Rules in order:
1. Empty slice → `"empty"`
2. Any state has `actionable` containing `"agent"` → `"active"`
3. All states have `terminal = true` → `"done"`
4. All states have `satisfies_deps = true` or `terminal = true`, and at least one has `satisfies_deps = true` → `"complete"`
5. Otherwise → `"active"`

No state ID strings are compared anywhere. The function depends only on `StateConfig` boolean flags and the `actionable` vec.

**`apm/src/cmd/epic.rs`** — new file, `pub fn run_list(root: &Path) -> Result<()>`:
1. `Config::load(root)` — read aggressive-fetch flag, tickets dir, and workflow states
2. If aggressive, `git::fetch_all(root)` (warn on error, continue)
3. `git::epic_branches(root)` — list branch names, sort alphabetically
4. `ticket::load_all_from_git(root, &config.tickets.dir)` — all tickets
5. For each epic branch:
   - Strip `epic/` prefix; take first 8 chars as `id`, remainder after first `-` as slug
   - Humanize title: replace `-` with space, title-case each word
   - Filter tickets where `fm.epic.as_deref() == Some(id)`
   - For each matching ticket, look up its `StateConfig` via `config.workflow.states.iter().find(|s| s.id == fm.state)`; collect `&StateConfig` references (skip tickets whose state is unknown)
   - Call `epic::derive_epic_state(&state_configs)`
   - Count per state ID; build non-zero counts string (`"2 in_progress, 1 ready"`)
   - `println!("{id:<8} [{derived_state:<12}] {title:<40} {counts}")`
6. No output (and `Ok(())`) when no epics exist

**`apm/src/main.rs`** — add `Epic { #[command(subcommand)] cmd: EpicCommand }` to `Command` enum; add `enum EpicCommand { List }`; dispatch to `cmd::epic::run_list(&root)?`.

Output example:
```
ab12cd34 [active      ] User Authentication       2 in_progress, 1 ready, 3 implemented
ef567890 [empty       ] Billing Overhaul
```

### Tests

Unit tests inline in `apm-core/src/epic.rs` for `derive_epic_state`: empty slice, all terminal, all satisfies_deps+terminal, any agent-actionable, mixed states.

Integration test in `apm/tests/integration.rs`: temp git repo with two fake `epic/*` remote branches and ticket files (with `epic = "..."` frontmatter) on `ticket/*` branches; `apm.toml` defines workflow states with appropriate flags; assert `apm epic list` stdout matches expected lines.

### Open questions


### Amendment requests

- [x] Delete the duplicate "### Files changed" and "### State derivation note" and "### Tests" sections that remain at the bottom of the spec. They still contain the old `derive_epic_state(states: &[&str])` signature with hardcoded state names ("in_design", "in_progress", "accepted", "closed", "implemented"). The corrected Approach above those sections is authoritative; the stale duplicates must be removed entirely.

### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-04-01T21:55Z | — | new | claude-0401-2145-a8f3 |
| 2026-04-01T21:59Z | new | groomed | claude-0401-2145-a8f3 |
| 2026-04-02T00:47Z | groomed | in_design | philippepascal |
| 2026-04-02T00:52Z | in_design | specd | claude-0401-2345-spec1 |
| 2026-04-02T01:37Z | specd | ammend | philippepascal |
| 2026-04-02T01:40Z | ammend | in_design | philippepascal |
| 2026-04-02T01:44Z | in_design | specd | claude-0402-0140-spec2 |
| 2026-04-02T01:55Z | specd | ammend | philippepascal |
| 2026-04-02T01:56Z | ammend | in_design | philippepascal |
| 2026-04-02T01:57Z | in_design | specd | claude-0402-0156-3680 |
| 2026-04-02T02:03Z | specd | ammend | apm |
| 2026-04-02T02:11Z | ammend | in_design | philippepascal |
| 2026-04-02T02:13Z | in_design | specd | claude-0402-0212-spec4 |
| 2026-04-02T02:28Z | specd | ready | apm |
| 2026-04-02T05:54Z | ready | in_progress | philippepascal |