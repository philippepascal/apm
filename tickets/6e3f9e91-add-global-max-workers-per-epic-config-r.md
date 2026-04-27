+++
id = "6e3f9e91"
title = "Add global max_workers_per_epic config; remove per-epic override"
state = "in_design"
priority = 0
effort = 5
risk = 2
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/6e3f9e91-add-global-max-workers-per-epic-config-r"
created_at = "2026-04-27T20:28:07.069581Z"
updated_at = "2026-04-27T20:55:50.048261Z"
epic = "5ea30227"
target_branch = "epic/5ea30227-strategy-and-dependency-hardening"
+++

## Spec

### Problem

Per-epic concurrency is currently controlled via a per-epic override: `apm epic set <id> max_workers <N>` writes a `max_workers` entry to `.apm/epics.toml`, and both the engine loop (`apm work`) and `apm start --next` read it via `Config::blocked_epics()` to cap concurrent workers per epic. Epics **without** an explicit entry are completely uncapped — any number of workers can be dispatched into the same epic simultaneously.\n\nThe design spec at `docs/strategy-and-dependencies.md` (§ 'Epic concurrency') replaces this model: each epic gets at most one active worker by default, controlled by a single global `max_workers_per_epic` setting (default `1`). Users gain parallelism by creating more epics, not by raising a per-epic cap. This makes epics the atomic parallelism unit and eliminates within-epic merge races.\n\nThe per-epic override mechanism must be removed entirely: `apm epic set <id> max_workers` should become an error, `.apm/epics.toml` should stop being read, and the global limit must be enforced uniformly for every epic — including the `run_next` path (`apm start --next --spawn`), which currently applies no epic concurrency limit at all.

### Acceptance criteria

- [ ] `apm epic set <id> max_workers <N>` exits non-zero and prints an error naming `owner` as the only valid field
- [ ] `apm epic set <id> owner <username>` continues to work correctly after the removal
- [ ] `.apm/config.toml` / `apm.toml` accepts `[agents] max_workers_per_epic = <N>` and parses it as a usize
- [ ] When `max_workers_per_epic` is absent from config, it defaults to `1`
- [ ] The engine loop (`apm work`) does not dispatch a second ticket from epic E while a worker for epic E is already active, given `max_workers_per_epic = 1` (the default)
- [ ] The engine loop (`apm work`) dispatches a second ticket from epic E when `max_workers_per_epic = 2` and only one worker is active in that epic
- [ ] `apm start --next` skips all tickets in epic E if epic E already has a ticket in an agent-active non-startable state and `max_workers_per_epic = 1`
- [ ] `apm epic show <id>` no longer prints a `Max workers:` line
- [ ] `.apm/epics.toml`, if present, is no longer read and its `max_workers` entries are silently ignored
- [ ] All removed integration tests (`epic_set_max_workers_*`, `epic_set_zero_value_exits_nonzero`, `epic_set_preserves_existing_config_content`) are replaced by a test asserting `apm epic set <id> max_workers <N>` now errors

### Out of scope

- Extending `apm validate` with dependency/strategy rule checks (ticket e845127e)
- Hash-trip on config/workflow change triggering re-validation (ticket b10d957a)
- The `apm refresh-epic` command (ticket 2973e208)
- Epic quiescence checks in `apm epic close` (ticket 056b1ee1)
- Migrating or flagging existing `.apm/epics.toml` files in user repos; silently ignoring them is sufficient
- Changing `max_concurrent` behaviour (the cross-epic global worker cap under `[agents]`)
- Any changes to `apm validate` that are not strictly required to make removed symbols compile-clean

### Approach

Seven files change. Order matters: update `config.rs` first (removes `EpicConfig`, adds global field, rewrites `blocked_epics`), then the callers in `start.rs`, `cmd/epic.rs`, `cmd/work.rs`, then tests and docs.

#### `apm-core/src/config.rs`

Add `max_workers_per_epic: usize` to `AgentsConfig` with a serde default of `1`:
```rust
#[serde(default = "default_max_workers_per_epic")]
pub max_workers_per_epic: usize,
```
Add `fn default_max_workers_per_epic() -> usize { 1 }` alongside `default_max_concurrent`. Update `AgentsConfig::default()` to set `max_workers_per_epic: 1`.

Remove `EpicConfig` struct (lines 5–8) and `epics: HashMap<String, EpicConfig>` from `Config` (line 181).

Remove the `.apm/epics.toml` loading block from `Config::load()` (lines 603–615).

Remove `epic_max_workers()` (lines 511–513). Its only external caller is `apm/src/cmd/epic.rs:160` — remove that call site too.

Rewrite `blocked_epics()` to apply the global limit uniformly to every epic (not just those with an explicit entry):
```rust
pub fn blocked_epics(&self, active_epic_ids: &[Option<String>]) -> Vec<String> {
    let limit = self.agents.max_workers_per_epic;
    let mut counts: std::collections::HashMap<&str, usize> = std::collections::HashMap::new();
    for eid in active_epic_ids.iter().filter_map(|e| e.as_deref()) {
        *counts.entry(eid).or_insert(0) += 1;
    }
    counts.into_iter()
        .filter(|(_, count)| *count >= limit)
        .map(|(eid, _)| eid.to_string())
        .collect()
}
```

Unit tests: remove `epic_config_parses_from_epics_toml`, `epic_config_absent_defaults_empty`, `epic_config_no_max_workers_returns_none`. Add `agents_max_workers_per_epic_defaults_to_one` (parse minimal config, assert value is 1), `blocked_epics_global_limit_one` (limit=1, one active worker in epic A → epic A blocked), `blocked_epics_global_limit_two` (limit=2, one active worker → not blocked).

#### `apm-core/src/start.rs` — `run_next()` (line 335)

After loading `tickets` and computing `actionable`/`startable`, add epic-concurrency filtering before the `pick_next` call:

```rust
let active_epic_ids: Vec<Option<String>> = tickets.iter()
    .filter(|t| {
        let s = t.frontmatter.state.as_str();
        actionable.contains(&s) && !startable.contains(&s)
    })
    .map(|t| t.frontmatter.epic.clone())
    .collect();
let blocked = config.blocked_epics(&active_epic_ids);
let tickets: Vec<_> = tickets.into_iter()
    .filter(|t| match t.frontmatter.epic.as_deref() {
        Some(eid) => !blocked.iter().any(|b| b == eid),
        None => true,
    })
    .collect();
```

Tickets with no epic pass through unfiltered. Tickets in agent-actionable non-startable states are those currently being worked on by an agent — the correct proxy for "active worker" in this stateless context.

#### `apm/src/cmd/epic.rs` — `run_set()` (line 201)

Change the field guard (line 202–204): remove `"max_workers"` from the accepted set and update the error message to `"valid fields: owner"`.

Remove the `if field == "owner"` wrapper — `owner` is now the only path. Promote the owner logic to the top of the function body (no logic change, just removes the conditional shell).

Delete the `max_workers` handling block entirely (lines 234–269): the `epics_path` variable, the `toml_edit` read/parse, the `if value == "-"` / else branch, and `std::fs::write`.

#### `apm/src/cmd/epic.rs` — `run_show()` (around line 160)

Remove the three lines that call `epic_max_workers` and print `Max workers:`:
```rust
if let Some(limit) = ctx.config.epic_max_workers(epic_id) {
    println!("Max workers: {limit}");
}
```

#### `apm/src/main.rs` — CLI help text

Update the `field` argument description in the `epic set` sub-command from `"Field to update: max_workers or owner"` to `"Field to update: owner"`.

#### `apm/tests/integration.rs`

Remove: `epic_set_max_workers_writes_config`, `epic_set_max_workers_updates_existing_value`, `epic_set_max_workers_clear_removes_field`, `epic_set_max_workers_invalid_value_is_error`, `epic_set_zero_value_exits_nonzero`, `epic_set_preserves_existing_config_content`.

Update `epic_set_nonexistent_epic_exits_nonzero`: change subprocess args from `["epic", "set", "deadbeef", "max_workers", "2"]` to `["epic", "set", "deadbeef", "owner", "alice"]`.

Add `epic_set_max_workers_is_now_unknown_field`: assert that `apm::cmd::epic::run_set(p, &epic_id, "max_workers", "2")` returns `Err`.

Add `agents_max_workers_per_epic_defaults_to_one`: load config from a repo with no `max_workers_per_epic` key and assert `config.agents.max_workers_per_epic == 1`.

#### `docs/commands.md` — `apm epic set` section (lines ~808–848)

Remove the `max_workers` synopsis lines, the "Set `max_workers` to cap…" paragraph, the `max_workers` row from the options table, and the `.apm/epics.toml` row from the file-internals table.

### `apm-core/src/config.rs`

1. Add `max_workers_per_epic: usize` to `AgentsConfig` with a serde default of `1`:
   ```rust
   #[serde(default = "default_max_workers_per_epic")]
   pub max_workers_per_epic: usize,
   ```
   Add `fn default_max_workers_per_epic() -> usize { 1 }` alongside the existing `default_max_concurrent`. Update `AgentsConfig::default()` to set `max_workers_per_epic: 1`.

2. Remove `EpicConfig` struct (lines 5–8) and the `epics: HashMap<String, EpicConfig>` field from `Config` (line 181).

3. Remove the `.apm/epics.toml` loading block from `Config::load()` (lines 603–615).

4. Remove `epic_max_workers()` (lines 511–513). Its only external caller is `apm/src/cmd/epic.rs:160`; remove that call site too (see below).

5. Rewrite `blocked_epics()` to apply the global limit uniformly to every epic — not just those with an explicit entry:
   ```rust
   pub fn blocked_epics(&self, active_epic_ids: &[Option<String>]) -> Vec<String> {
       let limit = self.agents.max_workers_per_epic;
       let mut counts: std::collections::HashMap<&str, usize> = std::collections::HashMap::new();
       for eid in active_epic_ids.iter().filter_map(|e| e.as_deref()) {
           *counts.entry(eid).or_insert(0) += 1;
       }
       counts.into_iter()
           .filter(|(_, count)| *count >= limit)
           .map(|(eid, _)| eid.to_string())
           .collect()
   }
   ```

6. Unit tests in `config.rs`:
   - Remove `epic_config_parses_from_epics_toml`, `epic_config_absent_defaults_empty`, `epic_config_no_max_workers_returns_none`.
   - Add `agents_max_workers_per_epic_defaults_to_one`: parse a minimal config (no `max_workers_per_epic` key) and assert `config.agents.max_workers_per_epic == 1`.
   - Add `blocked_epics_global_limit_one`: with limit=1 and one active worker in epic A, assert epic A is in the returned blocked list.
   - Add `blocked_epics_global_limit_two`: with limit=2 and one active worker in epic A, assert epic A is NOT blocked.

---

### `apm-core/src/start.rs` — `run_next()` (line 335)

After loading `tickets` (line 347) and computing `actionable`/`startable`, add epic-concurrency filtering before the `pick_next` call (line 351):

```rust
// Compute active epic IDs from tickets currently being worked on by an agent
// (agent-actionable but no longer startable = already in progress).
let active_epic_ids: Vec<Option<String>> = tickets.iter()
    .filter(|t| {
        let s = t.frontmatter.state.as_str();
        actionable.contains(&s) && !startable.contains(&s)
    })
    .map(|t| t.frontmatter.epic.clone())
    .collect();
let blocked = config.blocked_epics(&active_epic_ids);
let tickets: Vec<_> = tickets.into_iter()
    .filter(|t| match t.frontmatter.epic.as_deref() {
        Some(eid) => !blocked.iter().any(|b| b == eid),
        None => true,
    })
    .collect();
```

Then pass `&tickets` to `pick_next` as before. Tickets with no epic are never filtered.

---

### `apm/src/cmd/epic.rs` — `run_set()` (line 201)

1. Change the field guard (line 202–204):
   - **Before:** `if field != "max_workers" && field != "owner" { bail!("... valid fields: max_workers, owner") }`
   - **After:** `if field != "owner" { bail!("unknown field {field:?}; valid fields: owner") }`

2. Remove the `if field == "owner"` wrapper — owner is now the only path. Promote the owner logic to the top level of the function body (no change in logic, just removes the conditional wrapper and the early return).

3. Delete the `max_workers` handling block entirely (lines 234–269: `let apm_dir`, `let epics_path`, the `if value == "-"` / else branch, and `std::fs::write`).

---

### `apm/src/cmd/epic.rs` — `run_show()` (around line 160)

Remove the three lines that call `epic_max_workers` and print `Max workers:`:
```rust
// Delete:
if let Some(limit) = ctx.config.epic_max_workers(epic_id) {
    println!("Max workers: {limit}");
}
```

---

### `apm/src/main.rs` — CLI help text

Update the `field` argument description in the `epic set` sub-command from `"Field to update: max_workers or owner"` to `"Field to update: owner"`.

---

### `apm/tests/integration.rs`

**Remove** these tests (all test `epic set max_workers` behaviour that is being deleted):
- `epic_set_max_workers_writes_config`
- `epic_set_max_workers_updates_existing_value`
- `epic_set_max_workers_clear_removes_field`
- `epic_set_max_workers_invalid_value_is_error`
- `epic_set_zero_value_exits_nonzero`
- `epic_set_preserves_existing_config_content`

**Update** `epic_set_nonexistent_epic_exits_nonzero`: change the subprocess args from `["epic", "set", "deadbeef", "max_workers", "2"]` to `["epic", "set", "deadbeef", "owner", "alice"]` so it still tests non-zero exit for a missing epic using the surviving `owner` field.

**Add**:
- `epic_set_max_workers_is_now_unknown_field`: call `apm::cmd::epic::run_set(p, &epic_id, "max_workers", "2")` and assert it returns `Err`.
- `agents_max_workers_per_epic_defaults_to_one`: load config from a repo with no `max_workers_per_epic` key and assert `config.agents.max_workers_per_epic == 1`.

---

### `docs/commands.md` — `apm epic set` section (lines ~808–848)

- Remove the two `apm epic set <id> max_workers …` lines from the synopsis.
- Delete the "Set `max_workers` to cap…" paragraph from the description.
- Remove the `max_workers` row from the options table.
- Remove the `.apm/epics.toml` row from the file-internals table.

---

### `.apm/config.toml` (project config)

No change required — `max_workers_per_epic` defaults to `1` when absent. Optionally add `max_workers_per_epic = 1` under `[agents]` for explicitness, but this is cosmetic.

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-04-27T20:28Z | — | new | philippepascal |
| 2026-04-27T20:43Z | new | groomed | philippepascal |
| 2026-04-27T20:47Z | groomed | in_design | philippepascal |