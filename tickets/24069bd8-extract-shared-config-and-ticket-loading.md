+++
id = "24069bd8"
title = "Extract shared config-and-ticket loading helper in CLI crate"
state = "in_design"
priority = 0
effort = 3
risk = 2
author = "philippepascal"
branch = "ticket/24069bd8-extract-shared-config-and-ticket-loading"
created_at = "2026-04-07T22:30:46.572883Z"
updated_at = "2026-04-07T22:56:20.027719Z"
epic = "ac0fb648"
target_branch = "epic/ac0fb648-code-separation-and-reuse-cleanup"
+++

## Spec

### Problem

Approximately 12 command handlers in `apm/src/cmd/` open with the same boilerplate sequence:

```rust
let config = Config::load(root)?;
let aggressive = config.sync.aggressive && !no_aggressive;
if aggressive {
    if let Err(e) = git::fetch_all(root) {
        eprintln!("warning: fetch failed: {e:#}");
    }
}
let tickets = ticket::load_all_from_git(root, &config.tickets.dir)?;
```

The commands that use the full standard pattern are: `list.rs`, `verify.rs`, `validate.rs`, `review.rs`, `set.rs`, and the three sub-functions in `epic.rs`. Commands `new.rs` and `clean.rs` share the `Config::load` step but diverge immediately after (no fetch, no ticket load). Commands `show.rs` and `spec.rs` use a per-branch fetch variant and do not load all tickets, making them a different shape that does not benefit from the same helper.

Each copy drifts slightly — some omit the `!no_aggressive` guard, some load tickets conditionally — so future changes to the loading sequence (e.g. adding a validation step or changing fetch behaviour) must be hunted down and applied individually. The boilerplate also obscures the real command logic: 5–8 lines of identical setup must be read past before the interesting code begins.

The desired state is a single `CmdContext` type and a small set of constructor functions living in `apm/src/ctx.rs`, so that each command handler expresses its setup intent in one line and its unique logic without noise.

### Acceptance criteria

- [ ] A `CmdContext` struct exists in `apm/src/ctx.rs` with public fields `config: Config`, `tickets: Vec<Ticket>`, and `aggressive: bool`
- [ ] `CmdContext::load(root: &Path, no_aggressive: bool) -> Result<CmdContext>` loads config, performs `git::fetch_all` when `config.sync.aggressive && !no_aggressive` (printing a warning on failure, not returning an error), then loads all tickets
- [ ] `CmdContext::load_config_only(root: &Path) -> Result<Config>` loads and returns the config without performing any fetch or ticket load
- [ ] `list.rs` uses `CmdContext::load` and removes its inline boilerplate
- [ ] `verify.rs` uses `CmdContext::load` and removes its inline boilerplate
- [ ] `validate.rs` uses `CmdContext::load` (or `load_config_only` for the `--config-only` branch) and removes its inline boilerplate
- [ ] `review.rs` uses `CmdContext::load` and removes its inline boilerplate
- [ ] `set.rs` uses `CmdContext::load` and removes its inline boilerplate
- [ ] `epic.rs` sub-functions (`run_list`, `run_show`, `run_close`) each use whichever helper matches their pattern and remove their inline boilerplate
- [ ] `new.rs` uses `CmdContext::load_config_only` and removes its inline `Config::load` call
- [ ] `clean.rs` uses `CmdContext::load_config_only` and removes its inline `Config::load` call
- [ ] All existing `apm` integration tests and unit tests pass without modification
- [ ] `cargo clippy` reports no new warnings in the `apm` crate

### Out of scope

- Refactoring `show.rs` or `spec.rs` — they use a per-branch fetch (`git::fetch_branch`) and read a single ticket directly; a different helper shape would be needed
- Refactoring `sync.rs` — it has an `--offline` flag that inverts the fetch guard and delegates ticket loading internally to `sync::detect`; its shape does not match the common pattern
- Refactoring `state.rs` or `start.rs` — they delegate entirely to `apm-core` and contain no CLI-layer loading boilerplate
- Extracting a push-after-modification helper — commands that push after writing a ticket are addressed in a separate ticket
- Moving any logic into `apm-core` — this ticket only adds a thin helper within the `apm` (CLI) crate
- Changing the behaviour of any command — this is a pure mechanical refactor; observable behaviour must remain identical

### Approach

**1. Create `apm/src/ctx.rs`**

Add a new module with two public items:

```rust
use std::path::Path;
use anyhow::Result;
use apm_core::{config::Config, ticket::Ticket, git};

pub struct CmdContext {
    pub config: Config,
    pub tickets: Vec<Ticket>,
    pub aggressive: bool,
}

impl CmdContext {
    /// Load config, optionally fetch all remotes, then load all tickets.
    /// A fetch failure is a warning, not a hard error.
    pub fn load(root: &Path, no_aggressive: bool) -> Result<Self> {
        let config = Config::load(root)?;
        let aggressive = config.sync.aggressive && !no_aggressive;
        if aggressive {
            if let Err(e) = git::fetch_all(root) {
                eprintln!("warning: fetch failed: {e:#}");
            }
        }
        let tickets = apm_core::ticket::load_all_from_git(root, &config.tickets.dir)?;
        Ok(Self { config, tickets, aggressive })
    }

    /// Load config only — no fetch, no ticket load.
    pub fn load_config_only(root: &Path) -> Result<Config> {
        Config::load(root)
    }
}
```

**2. Register the module in `apm/src/main.rs` (or `lib.rs`)**

Add `mod ctx;` alongside the existing `mod cmd;` declarations. No public re-export is needed; callers refer to `crate::ctx::CmdContext`.

**3. Update command handlers — full pattern (use `CmdContext::load`)**

For each of `list.rs`, `verify.rs`, `validate.rs`, `review.rs`, `set.rs`:
- Replace the 4–8 line boilerplate block with `let ctx = CmdContext::load(root, no_aggressive)?;`
- Replace subsequent references:
  - `config` → `ctx.config`
  - `tickets` → `ctx.tickets`
  - `aggressive` → `ctx.aggressive`
- For `validate.rs` where tickets are only loaded when `!config_only`: call `CmdContext::load_config_only(root)?` for the config-only path, and `CmdContext::load(root, no_aggressive)?` otherwise.

**4. Update `epic.rs` sub-functions**

- `run_list`: calls `git::fetch_all` when `config.sync.aggressive` is true (no `no_aggressive` arg); replace with `CmdContext::load(root, false)?` — `no_aggressive: false` preserves the existing behaviour since the flag doesn't exist in this subcommand.
- `run_show`: replace with `CmdContext::load(root, no_aggressive)?`.
- `run_close`: config-only load followed by manual ticket load; replace the `Config::load` line with `CmdContext::load_config_only(root)?` for config, keep the existing ticket load since it filters tickets before all-ticket loading.

**5. Update `new.rs` and `clean.rs`**

Both call `Config::load(root)?` as their only boilerplate. Replace with `let config = CmdContext::load_config_only(root)?;`. No other changes needed.

**6. Verify**

Run `cargo test -p apm` and `cargo clippy -p apm -- -D warnings` to confirm no regressions. No test changes should be necessary.

**Constraints**

- Do not change any function signatures visible outside the `apm` crate.
- Preserve the exact warning message text (`"warning: fetch failed: {e:#}"`) so existing test fixtures that match stderr output keep passing.
- `CmdContext::load_config_only` returns `Config` directly (not `CmdContext`) to avoid allocating an empty `Vec<Ticket>` in callers that never need it.

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-04-07T22:30Z | — | new | philippepascal |
| 2026-04-07T22:44Z | new | groomed | apm |
| 2026-04-07T22:53Z | groomed | in_design | philippepascal |