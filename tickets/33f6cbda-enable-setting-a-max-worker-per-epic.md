+++
id = "33f6cbda"
title = "enable setting a max worker per epic"
state = "in_design"
priority = 0
effort = 4
risk = 3
author = "philippepascal"
branch = "ticket/33f6cbda-enable-setting-a-max-worker-per-epic"
created_at = "2026-04-07T19:08:03.080608Z"
updated_at = "2026-04-07T19:23:37.472087Z"
+++

## Spec

### Problem

APM supports a global `[agents] max_concurrent` setting that caps the total number of simultaneously running workers. When using `apm work`, this global cap applies uniformly across all epics. There is currently no way to say "this epic should run at most N workers at the same time", which matters when:

- One epic is I/O-bound or API-rate-limited and flooding it with workers makes things worse.
- A project has multiple epics with different parallelism needs (e.g. a fast-moving feature epic vs. a careful refactor epic).
- A supervisor wants to throttle an in-flight epic without pausing all work.

The desired behaviour is: users can assign a `max_workers` ceiling to a specific epic, and `apm work` will not spawn more than that many concurrent workers for tickets belonging to that epic, regardless of the global `max_concurrent` limit.

### Acceptance criteria

- [ ] `apm epic set-max-workers <epic-id> <N>` writes `max_workers = N` into the `[epics."<epic-id>"]` table in `.apm/config.toml`
- [ ] `apm epic set-max-workers <epic-id> 0` (or `--unset`) removes the `max_workers` field from that table, restoring uncapped behaviour
- [ ] `apm epic show <epic-id>` prints the current `max_workers` limit when one is set
- [ ] `apm work` (without `--epic`) respects each epic's `max_workers` limit: it does not spawn a new worker for a ticket whose epic already has `max_workers` active workers
- [ ] `apm work --epic <id>` also respects the `max_workers` limit for that epic
- [ ] When a running worker finishes and a slot opens up, `apm work` spawns the next eligible ticket in that epic (normal pick-next behaviour resumes)
- [ ] Tickets with no epic, or whose epic has no `max_workers` set, are unaffected — they are still bounded only by `[agents] max_concurrent`
- [ ] Setting `max_workers` greater than `[agents] max_concurrent` is allowed but has no additional effect (the global cap still binds)
- [ ] `apm epic set-max-workers` with a non-existent epic ID prints an error and exits non-zero
- [ ] `apm epic set-max-workers` with a value ≤ 0 (other than the unset sentinel) prints an error and exits non-zero

### Out of scope

- Setting a global default `max_workers` that applies to all epics without an explicit override
- Per-ticket concurrency limits (only epic-level granularity is in scope)
- Dynamically adjusting `max_workers` while `apm work` is already running (takes effect on next loop iteration only; no hot-reload)
- Surfacing per-epic worker counts in `apm epic list`
- Any UI (apm-ui) changes
- Migrating or deprecating the existing `[agents] max_concurrent` global setting

### Approach

Store per-epic limits in `.apm/config.toml` using TOML table-per-epic syntax alongside existing `[agents]` and `[workers]` sections:

```toml
[epics."8db73240"]
max_workers = 2

[epics."a1b2c3d4"]
max_workers = 1
```

No new file is needed. Epic IDs use the 8-char prefix only (same as `ticket.frontmatter.epic`).

#### 1. `apm-core/src/config.rs`

Add `EpicConfig` struct and wire it into `Config`:

```rust
#[derive(Debug, Deserialize, Default)]
pub struct EpicConfig {
    pub max_workers: Option<usize>,
}

// In Config:
#[serde(default)]
pub epics: HashMap<String, EpicConfig>,

// Helper on Config:
pub fn epic_max_workers(&self, epic_id: &str) -> Option<usize> {
    self.epics.get(epic_id).and_then(|e| e.max_workers)
}
```

#### 2. `apm/src/cmd/epic.rs` — new `set-max-workers` subcommand

- Add `SetMaxWorkers { epic_id: String, max_workers: Option<usize> }` variant to `EpicCommand` (`None` = unset/remove).
- Validate: epic must exist via `epic_branches(root)`; `max_workers` must be ≥ 1 if provided.
- Load `.apm/config.toml` via `toml_edit` (preserves comments and key order), insert/update or remove `epics."<id>".max_workers`, write back.

#### 3. `apm/src/cmd/epic.rs` — update `show`

After printing the ticket list, print `Max workers: N` if `config.epic_max_workers(id)` returns `Some(n)`.

#### 4. `apm-core/src/work.rs` and `apm/src/cmd/work.rs`

Add `epic_id: Option<String>` to the `Worker` struct; populate it at spawn time from `ticket.frontmatter.epic`.

Before each spawn attempt, compute `blocked_epics`: the set of epic IDs where `epic_worker_count(&workers, id) >= config.epic_max_workers(id)`. Pass `blocked_epics: &[String]` to `spawn_next_worker`, which pre-filters tickets belonging to those epics before calling `pick_next`. This keeps `pick_next` unchanged.

```rust
fn epic_worker_count(workers: &[Worker], epic_id: &str) -> usize {
    workers.iter().filter(|w| w.epic_id.as_deref() == Some(epic_id)).count()
}
```

#### 5. `apm/src/main.rs`

Add `set-max-workers` to the `epic` subcommand tree:

```
apm epic set-max-workers <epic-id> <N>
apm epic set-max-workers <epic-id> --unset
```

#### Order of changes

1. `config.rs` — add `EpicConfig`, `HashMap<String, EpicConfig>`, helper (no behaviour change)
2. `epic.rs` + `main.rs` — add `set-max-workers` subcommand and `show` update
3. `start.rs` / `work.rs` — add `epic_id` to `Worker`; add `blocked_epics` filtering to the spawn loop

#### Constraints

- Use `toml_edit` (not `toml` serde round-trip) when writing `config.toml` to preserve comments.
- Fully backward-compatible: no `[epics.*]` tables → identical behaviour to today.
- `max_workers > max_concurrent` is valid but the global cap still binds first.

### Storage: `.apm/config.toml`

Add a TOML table-per-epic in the existing config file. TOML supports dotted keys for this:

```toml
[epics."8db73240"]
max_workers = 2

[epics."a1b2c3d4"]
max_workers = 1
```

No new file is needed. The `epics` key lives alongside `[agents]`, `[workers]`, etc.

### 1. `apm-core/src/config.rs`

- Add `EpicConfig` struct:
  ```rust
  #[derive(Debug, Deserialize, Default)]
  pub struct EpicConfig {
      pub max_workers: Option<usize>,
  }
  ```
- Add field to `Config`:
  ```rust
  #[serde(default)]
  pub epics: HashMap<String, EpicConfig>,
  ```
- Add helper:
  ```rust
  impl Config {
      pub fn epic_max_workers(&self, epic_id: &str) -> Option<usize> {
          self.epics.get(epic_id).and_then(|e| e.max_workers)
      }
  }
  ```

### 2. `apm/src/cmd/epic.rs` — new `set-max-workers` subcommand

- Add `SetMaxWorkers { epic_id: String, max_workers: Option<usize> }` variant to the `EpicCommand` enum (a `None` value means unset/remove).
- Validate: epic must exist (`epic_branches(root)` must contain a branch whose ID prefix matches); `max_workers` must be ≥ 1 if provided.
- Load `.apm/config.toml` as raw text using `toml_edit` (already a dependency or add it).
- Insert or update `epics."<id>".max_workers = N`, or remove the key if unsetting.
- Write the file back.

### 3. `apm/src/cmd/epic.rs` — update `show`

- After printing ticket list, if `config.epic_max_workers(id)` returns `Some(n)`, print `Max workers: n`.

### 4. `apm/src/cmd/work.rs` and `apm-core/src/work.rs`

Currently the worker loop tracks a flat `Vec<Worker>`. Extend the spawn-check:

```rust
// Before spawning, count active workers in the target epic.
fn epic_worker_count(workers: &[Worker], epic_id: &str) -> usize {
    workers.iter().filter(|w| w.epic_id.as_deref() == Some(epic_id)).count()
}

// Spawn guard:
let can_spawn_for_epic = |epic_id: Option<&str>| -> bool {
    match epic_id {
        None => true,
        Some(id) => match config.epic_max_workers(id) {
            None => true,
            Some(limit) => epic_worker_count(&workers, id) < limit,
        },
    }
};
```

The `Worker` struct needs an `epic_id: Option<String>` field populated at spawn time from the ticket's frontmatter.

`spawn_next_worker` in `apm-core/src/start.rs` already receives the full ticket; add `epic_id` to the returned/created `Worker` record.

The pick-next call also needs to pass a per-epic exclusion list: if an epic is already at its limit, skip all tickets in that epic before calling `pick_next`. Alternatively, `spawn_next_worker` can accept a `blocked_epics: &[&str]` parameter to pre-filter tickets.

**Recommended approach** (simpler, no change to `pick_next`):
- Before spawning, compute `blocked_epics`: the set of epic IDs currently at their `max_workers` limit.
- Pass `blocked_epics` to `spawn_next_worker`; it filters out those tickets before running `pick_next`.
- Signature change: `spawn_next_worker(root, no_aggressive, skip_permissions, epic_filter, blocked_epics: &[String])`.

### 5. `apm/src/main.rs`

Add `set-max-workers` to the `epic` subcommand tree:

```
apm epic set-max-workers <epic-id> <N>
apm epic set-max-workers <epic-id> --unset
```

### Order of changes

1. `config.rs` — add `EpicConfig` + `HashMap<String, EpicConfig>` + helper (no behaviour change yet)
2. `epic.rs` — add `set-max-workers` + `show` update + wire up in `main.rs`
3. `start.rs` — add `epic_id` to `Worker` struct; populate from ticket frontmatter at spawn
4. `work.rs` — add `blocked_epics` computation and pass it through to `spawn_next_worker`

### Constraints

- `toml_edit` must be used (not `toml` serde round-trip) to preserve comments and ordering in `config.toml`.
- The feature must be fully backward-compatible: configs without any `[epics.*]` tables behave identically to today.
- Epic IDs in the config key use the 8-char prefix only (same as `ticket.frontmatter.epic`).

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-04-07T19:08Z | — | new | philippepascal |
| 2026-04-07T19:08Z | new | groomed | apm |
| 2026-04-07T19:19Z | groomed | in_design | philippepascal |