+++
id = "a5e1ea24"
title = "Implement apm epic list command"
state = "in_design"
priority = 6
effort = 3
risk = 2
author = "claude-0401-2145-a8f3"
agent = "97180"
branch = "ticket/a5e1ea24-implement-apm-epic-list-command"
created_at = "2026-04-01T21:55:09.722953Z"
updated_at = "2026-04-02T01:40:30.684744Z"
+++

## Spec

### Problem

Once epic branches exist there is no way to see them or their status at a glance. Engineers and the supervisor need to know which epics are active, how many tickets are in each state, and whether an epic is done.

The full design is in `docs/epics.md` (§ Commands — `apm epic list`). Epic state is always derived — never stored — using config-driven rules based on `StateConfig` flags, not hard-coded state ID strings: no tickets → `empty`; any ticket whose state has `actionable` containing `"agent"` → `active`; all tickets terminal (`terminal = true`) → `done`; all tickets dep-satisfied (`satisfies_deps = true`) or terminal with at least one dep-satisfied → `complete`; otherwise → `active`.

The command lists all `epic/*` remote branches and for each shows: short ID, title (from slug), derived state, and per-state ticket counts (e.g. `2 in_progress, 1 ready, 3 implemented`).

### Acceptance criteria

- [ ] `apm epic list` outputs one line per `epic/*` remote branch
- [ ] Each line shows the 8-char ID, the humanized title (hyphens → spaces, title-cased), derived state, and non-zero per-state ticket counts
- [ ] When no `epic/*` branches exist, the command exits 0 with no output
- [ ] Derived state is `empty` when no tickets reference the epic ID
- [ ] Derived state is `active` when any ticket's state config has `actionable` containing `"agent"`
- [ ] Derived state is `done` when all tickets have `terminal = true` in their state config
- [ ] Derived state is `complete` when all tickets are dep-satisfied (`satisfies_deps = true`) or terminal, and at least one is dep-satisfied
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

### Files changed

**1. `apm-core/src/ticket.rs` — add `epic` field to `Frontmatter`**

Add one optional field to the existing struct:

```rust
#[serde(skip_serializing_if = "Option::is_none")]
pub epic: Option<String>,
```

This is the only Frontmatter change in scope. The field is already expected by `docs/epics.md`; adding it makes it visible to downstream consumers without breaking existing tickets that omit it.

**2. `apm-core/src/git.rs` — add `epic_branches()`**

Mirror the existing `ticket_branches()` function:
- Collect local branches matching `epic/*`
- Collect remote branches matching `origin/epic/*`, stripping the `origin/` prefix
- Deduplicate (local wins)
- Return `Vec<String>`

**3. `apm-core/src/epic.rs` — new module with `derive_epic_state()`**

Add a public pure function `derive_epic_state(states: &[&str]) -> &'static str` encoding the rules from `docs/epics.md`:
1. Empty slice → `"empty"`
2. Any state is `"in_design"` or `"in_progress"` → `"in_progress"`
3. All states are `"accepted"` or `"closed"` → `"done"`
4. All states are `"implemented"`, `"accepted"`, or `"closed"` → `"implemented"`
5. Otherwise → `"in_progress"`

Re-export from `apm-core/src/lib.rs` as `pub mod epic`.

**4. `apm/src/cmd/epic.rs` — new file**

`pub fn run_list(root: &Path) -> Result<()>`:
1. `Config::load(root)` for aggressive-fetch flag and tickets dir
2. If aggressive, `git::fetch_all(root)` (warn on error, don't abort)
3. `git::epic_branches(root)` — list of `epic/<id>-<slug>` names, sorted
4. `ticket::load_all_from_git(root, &config.tickets.dir)` — all tickets
5. For each epic branch:
   a. Strip `epic/` prefix; split on first `-` to get `id` (8 chars) and slug remainder
   b. Humanize title: replace `-` with space, title-case each word
   c. Filter tickets where `fm.epic.as_deref() == Some(id)`
   d. Collect state strings; call `epic::derive_epic_state()`
   e. Count tickets per state; format non-zero counts as `"2 in_progress, 1 ready"`
   f. Print: `{id:<8} [{derived_state:<12}] {title:<40} {counts}`

**5. `apm/src/main.rs` — wire up the subcommand**

Add `Epic { #[command(subcommand)] cmd: EpicCommand }` to the top-level `Command` enum.
Add `enum EpicCommand { List }`.
Match arm: `Command::Epic { cmd: EpicCommand::List } => cmd::epic::run_list(&root)?`

### Output example

```
ab12cd34 [in_progress ] User Authentication       2 in_progress, 1 ready, 3 implemented
ef567890 [empty       ] Billing Overhaul
```

### State derivation note

The function hard-codes the state name strings from the spec. This avoids a `&Config` parameter. If the workflow config ever renames these states the derivation would need updating — acceptable given the spec names them explicitly.

### Tests

Unit tests inline in `apm-core/src/epic.rs` for `derive_epic_state`: empty, all closed, all implemented, any in_progress, any in_design, mixed.

Integration test in `apm/tests/integration.rs`: create a temp git repo with two fake `epic/*` remote branches and ticket files (with `epic = "..."` frontmatter) on `ticket/*` branches; assert `apm epic list` stdout matches expected lines.

### Open questions


### Amendment requests

- [x] `derive_epic_state` must not reference ticket state names. Remove hardcoded "in_design", "in_progress", "accepted", "closed", "implemented" from AC and Approach. Replace with config-driven rules: a ticket is "actively worked" when its state config has `actionable` containing "agent"; a ticket is "dep-satisfied" when its state has `satisfies_deps = true`; a ticket is "done" when its state has `terminal = true`. Update the AC to describe the four epic states using these flags: empty (no tickets), active (any agent-actionable ticket), complete (all tickets satisfies_deps or terminal, at least one satisfies_deps), done (all tickets terminal).
- [ ] Update the Approach's `derive_epic_state` implementation to accept `&[StateConfig]` and use the flags above instead of comparing state ID strings.

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