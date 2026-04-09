+++
id = "5ae5f97c"
title = "Add --epic exclusive mode to apm work command"
state = "closed"
priority = 6
effort = 4
risk = 2
author = "claude-0401-2145-a8f3"
agent = "22294"
branch = "ticket/5ae5f97c-add-epic-exclusive-mode-to-apm-work-comm"
created_at = "2026-04-01T21:55:49.406819Z"
updated_at = "2026-04-02T19:06:52.228899Z"
+++

## Spec

### Problem

The `apm work` engine currently dispatches any actionable ticket regardless of epic membership. When a supervisor wants to focus a work session exclusively on one epic's tickets — to drive it to completion without interleaving unrelated work — there is no way to restrict the engine to that scope.

The desired behaviour is:

```
apm work --epic ab12cd34
```

Only tickets whose frontmatter contains `epic = "ab12cd34"` are eligible for dispatch. Free tickets (no `epic` field) and tickets from other epics are skipped entirely. Dependency ordering (`depends_on`) still applies within the filtered set.

A config shorthand is also required so persistent epic focus can be set without repeating the flag:

```toml
[work]
epic = "ab12cd34"   # implies exclusive mode every time apm work runs
```

The CLI flag takes precedence over the config value. This is the exclusive mode described in `docs/epics.md` § `apm work` — Exclusive mode. No other scheduling modes (balanced, --and-free, per-epic limits) are supported.

### Acceptance criteria

- [x] `apm work --epic ab12cd34` dispatches only tickets where `frontmatter.epic == "ab12cd34"`
- [x] `apm work --epic ab12cd34` does not dispatch free tickets (no `epic` field)
- [x] `apm work --epic ab12cd34` does not dispatch tickets from a different epic
- [x] `apm work --dry-run --epic ab12cd34` prints only epic-scoped candidates
- [x] When `[work] epic = "ab12cd34"` is set in `apm.toml` (or `.apm/config.toml`), `apm work` (with no flag) behaves identically to `apm work --epic ab12cd34`
- [x] `apm work --epic <id>` takes precedence over a `[work] epic` config value when both are present
- [x] When no epic-matching tickets are actionable, `apm work --epic <id>` exits with "No tickets to work." (non-daemon) or waits and polls (daemon)
- [x] `apm work` with no `--epic` flag and no `[work] epic` config behaves exactly as before (all actionable tickets eligible)

### Out of scope

- Server-side epic filtering (`POST /api/work/start` body `epic` field) — covered by a separate server ticket
- `apm start --next --epic <id>` — the `--epic` flag is for `apm work` only
- Balanced or mixed scheduling (dispatching both epic and free tickets with any weighting)
- Adding the `epic` field to tickets (`apm new --epic`, `apm epic` commands) — those are separate epic command tickets
- UI engine controls epic selector (apm-ui) — separate ticket
- `depends_on` scheduling within epics — that feature is independent and covered elsewhere

### Approach

Eight files change in order (each step compiles before the next).

#### 1. `apm-core/src/ticket.rs` — add `epic` to `Frontmatter`

Add one optional field:
```rust
#[serde(skip_serializing_if = "Option::is_none")]
pub epic: Option<String>,
```
No migration needed — TOML deserialization is additive; existing tickets get `None`.

#### 2. `apm-core/src/config.rs` — add `[work]` section

New struct and field on `Config`:
```rust
#[derive(Debug, Deserialize, Default)]
pub struct WorkConfig {
    #[serde(default)]
    pub epic: Option<String>,
}
// on Config:
#[serde(default)]
pub work: WorkConfig,
```

#### 3. `apm-core/src/start.rs` — add `epic_filter` to `spawn_next_worker`

New signature:
```rust
pub fn spawn_next_worker(
    root: &Path,
    no_aggressive: bool,
    skip_permissions: bool,
    epic_filter: Option<&str>,
) -> Result<Option<(String, std::process::Child, PathBuf)>>
```
After `ticket::load_all_from_git`, filter:
```rust
let tickets: Vec<ticket::Ticket> = match epic_filter {
    Some(id) => tickets.into_iter()
        .filter(|t| t.frontmatter.epic.as_deref() == Some(id))
        .collect(),
    None => tickets,
};
```
`run_next()` (for `apm start --next`) does NOT get `epic_filter` — out of scope.

#### 4. `apm/src/cmd/start.rs` — update thin wrapper

Add `epic_filter: Option<&str>` to the `spawn_next_worker` wrapper and forward it to `apm_core::start::spawn_next_worker`.

#### 5. `apm-core/src/work.rs` — thread `epic_filter` through `run_engine_loop`

New signature adds `epic_filter: Option<String>`. Pass `epic_filter.as_deref()` to every `spawn_next_worker` call in the loop.

#### 6. `apm-server/src/work.rs` — update call site with `None`

The server calls `apm_core::work::run_engine_loop`. Add `None` as the `epic_filter` argument. No behaviour change — server epic integration is out of scope.

#### 7. `apm/src/cmd/work.rs` — accept `epic`, resolve filter, apply in `run_dry`

`run` gains `epic: Option<String>`. Resolve early:
```rust
let epic_filter: Option<String> = epic.or_else(|| config.work.epic.clone());
```
Pass `epic_filter.as_deref()` to `spawn_next_worker`.

In `run_dry`, add filter clause:
```rust
&& epic_filter.as_deref()
    .map_or(true, |id| t.frontmatter.epic.as_deref() == Some(id))
```

#### 8. `apm/src/main.rs` — add `--epic` flag to `Work` command

```rust
/// Restrict dispatching to tickets in this epic (8-char ID)
#[arg(long, value_name = "EPIC_ID")]
epic: Option<String>,
```
Pass to `cmd::work::run`.

#### Tests

Unit (`apm-core/src/config.rs`):
- `[work] epic = "ab12cd34"` parses correctly
- absent `[work]` section defaults to `None`

Integration (`apm/tests/integration.rs`):
- Two tickets (one with `epic = "ab12cd34"`, one free); `apm work --epic ab12cd34 --dry-run` shows only the epic ticket
- One free ticket; `apm work --epic ab12cd34 --dry-run` shows zero candidates
- One epic ticket; `apm work --dry-run` (no flag) still shows it (no regression)

### 1. Add `epic` to `Frontmatter` — `apm-core/src/ticket.rs`

Add one optional field to the `Frontmatter` struct:

```rust
#[serde(skip_serializing_if = "Option::is_none")]
pub epic: Option<String>,
```

No migration needed — TOML deserialization is additive; existing tickets get `None`.

---

### 2. Add `[work]` section to config — `apm-core/src/config.rs`

Add a new config struct and field:

```rust
#[derive(Debug, Deserialize, Default)]
pub struct WorkConfig {
    #[serde(default)]
    pub epic: Option<String>,
}
```

Add to `Config`:
```rust
#[serde(default)]
pub work: WorkConfig,
```

---

### 3. Thread `epic_filter` into `spawn_next_worker` — `apm-core/src/start.rs`

Change the signature:
```rust
pub fn spawn_next_worker(
    root: &Path,
    no_aggressive: bool,
    skip_permissions: bool,
    epic_filter: Option<&str>,
) -> Result<Option<(String, std::process::Child, PathBuf)>>
```

After `let tickets = ticket::load_all_from_git(...)?`, add one filter step:
```rust
let tickets: Vec<ticket::Ticket> = match epic_filter {
    Some(epic_id) => tickets.into_iter()
        .filter(|t| t.frontmatter.epic.as_deref() == Some(epic_id))
        .collect(),
    None => tickets,
};
```

Note: `run_next()` (used by `apm start --next`) does NOT get an `epic_filter` — that command is out of scope.

---

### 4. Update thin wrapper — `apm/src/cmd/start.rs`

`spawn_next_worker` in this file is a thin pass-through. Add `epic_filter: Option<&str>` and forward it.

---

### 5. Thread `epic_filter` through the engine loop — `apm-core/src/work.rs`

Change `run_engine_loop` signature to accept `epic_filter: Option<String>` and pass `epic_filter.as_deref()` to every `spawn_next_worker` call inside the loop.

---

### 6. Update `apm-server/src/work.rs` call site

The server calls `apm_core::work::run_engine_loop`. Add `None` as the new `epic_filter` argument. No behaviour change for the server.

---

### 7. Update `cmd/work.rs` — `apm/src/cmd/work.rs`

Change `run` signature to accept `epic: Option<String>`.

Resolve the effective filter early:
```rust
let epic_filter: Option<String> = epic.or_else(|| config.work.epic.clone());
```

Pass `epic_filter.as_deref()` to `super::start::spawn_next_worker`.

In `run_dry`, apply the same filter after loading tickets (add an extra filter clause for the epic).

---

### 8. Add `--epic` flag to CLI — `apm/src/main.rs`

In the `Work` variant, add:
```rust
/// Restrict dispatching to tickets in this epic (8-char ID)
#[arg(long, value_name = "EPIC_ID")]
epic: Option<String>,
```

Update the dispatch arm to pass `epic` to `cmd::work::run`.

---

### 9. Tests

**Unit — `apm-core/src/config.rs`:**
- `WorkConfig` parses `[work] epic = "ab12cd34"` correctly
- `WorkConfig` defaults to `None` when section is absent

**Integration — `apm/tests/integration.rs`:**
- Two tickets (one with `epic = "ab12cd34"`, one free). `apm work --epic ab12cd34 --dry-run` shows only the epic ticket.
- One free ticket. `apm work --epic ab12cd34 --dry-run` shows no candidates.
- One epic ticket. `apm work --dry-run` (no flag) shows the ticket (no regression).

### Order of changes

1. `apm-core/src/ticket.rs` (add `epic` field)
2. `apm-core/src/config.rs` (add `WorkConfig`)
3. `apm-core/src/start.rs` (add `epic_filter` param to `spawn_next_worker`)
4. `apm/src/cmd/start.rs` (update wrapper)
5. `apm-core/src/work.rs` (update `run_engine_loop`)
6. `apm-server/src/work.rs` (pass `None`)
7. `apm/src/cmd/work.rs` (add `epic` param, resolve filter)
8. `apm/src/main.rs` (add `--epic` flag)
9. Tests

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-04-01T21:55Z | — | new | claude-0401-2145-a8f3 |
| 2026-04-01T22:00Z | new | groomed | claude-0401-2145-a8f3 |
| 2026-04-02T00:49Z | groomed | in_design | philippepascal |
| 2026-04-02T00:54Z | in_design | specd | claude-0402-0050-spec1 |
| 2026-04-02T02:28Z | specd | ready | apm |
| 2026-04-02T06:03Z | ready | in_progress | philippepascal |
| 2026-04-02T06:10Z | in_progress | implemented | claude-0402-0604-w5ae |
| 2026-04-02T19:06Z | implemented | closed | apm-sync |