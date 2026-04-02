+++
id = "a5e1ea24"
title = "Implement apm epic list command"
state = "in_design"
priority = 6
effort = 0
risk = 0
author = "claude-0401-2145-a8f3"
agent = "87256"
branch = "ticket/a5e1ea24-implement-apm-epic-list-command"
created_at = "2026-04-01T21:55:09.722953Z"
updated_at = "2026-04-02T00:47:06.221425Z"
+++

## Spec

### Problem

Once epic branches exist there is no way to see them or their status at a glance. Engineers and the supervisor need to know which epics are active, how many tickets are in each state, and whether an epic is done.

The full design is in `docs/epics.md` (§ Commands — `apm epic list`). Epic state is always derived — never stored — using these rules: no tickets → `empty`; any ticket `in_design` or `in_progress` → `in_progress`; all `implemented` or later → `implemented`; all `accepted`/`closed` → `done`; otherwise → `in_progress`.

The command lists all `epic/*` remote branches and for each shows: short ID, title (from slug), derived state, and per-state ticket counts (e.g. `2 in_progress, 1 ready, 3 implemented`).

### Acceptance criteria

- [ ] `apm epic list` outputs one line per `epic/*` remote branch
- [ ] Each line shows the 8-char ID, the humanized title (hyphens → spaces, title-cased), derived state, and non-zero per-state ticket counts
- [ ] When no `epic/*` branches exist, the command exits 0 with no output
- [ ] Derived state is `empty` when no tickets reference the epic ID
- [ ] Derived state is `in_progress` when at least one ticket is in state `in_design` or `in_progress`
- [ ] Derived state is `implemented` when all tickets are in state `implemented` or a later non-terminal state (but not all accepted/closed)
- [ ] Derived state is `done` when all tickets are in state `accepted` or `closed`
- [ ] Derived state falls back to `in_progress` for any other mix of states
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


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-04-01T21:55Z | — | new | claude-0401-2145-a8f3 |
| 2026-04-01T21:59Z | new | groomed | claude-0401-2145-a8f3 |
| 2026-04-02T00:47Z | groomed | in_design | philippepascal |