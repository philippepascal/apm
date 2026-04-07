+++
id = "24069bd8"
title = "Extract shared config-and-ticket loading helper in CLI crate"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
branch = "ticket/24069bd8-extract-shared-config-and-ticket-loading"
created_at = "2026-04-07T22:30:46.572883Z"
updated_at = "2026-04-07T22:53:21.721662Z"
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

How the implementation will work.

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-04-07T22:30Z | — | new | philippepascal |
| 2026-04-07T22:44Z | new | groomed | apm |
| 2026-04-07T22:53Z | groomed | in_design | philippepascal |