+++
id = "bbd5d271"
title = "Validate owner against collaborators in config-based mode"
state = "in_progress"
priority = 0
effort = 3
risk = 2
author = "philippepascal"
branch = "ticket/bbd5d271-validate-owner-against-collaborators-in-"
created_at = "2026-04-08T15:09:59.601187Z"
updated_at = "2026-04-08T22:15:55.971974Z"
epic = "18dab82d"
target_branch = "epic/18dab82d-ticket-ownership-model"
depends_on = ["b0708201"]
+++

## Spec

### Problem

In config-based mode (no git_host provider), there is no validation when changing a ticket's owner. A typo in a username goes undetected. The `project.collaborators` list exists in config.toml but is never checked at runtime. Owner changes should validate the new owner against this list.

### Acceptance criteria

- [x] When `git_host.provider` is not set, `apm assign` validates the new owner against `project.collaborators`
- [x] If the new owner is not in the collaborators list, command fails with "unknown user '<name>'; valid collaborators: <list>"
- [x] If collaborators list is empty, validation is skipped (no restriction)
- [x] `apm set <id> owner <user>` has the same validation
- [x] Tests cover: valid collaborator accepted, unknown user rejected, empty collaborators list skips validation

### Out of scope

GitHub-based validation (separate ticket c738d9cc). Adding users to the collaborators list (manual config edit).

### Approach

Add a `validate_owner(config: &Config, username: &str) -> Result<()>` function in `apm-core/src/validate.rs` (the existing validation module). Logic:

- If `config.git_host.provider` is `Some(_)` (e.g. GitHub mode), return `Ok(())` immediately — this ticket only covers config-based mode.
- If `config.project.collaborators` is empty, return `Ok(())` — no restriction.
- If `username == "-"` (owner clear), return `Ok(())` — clearing is always allowed.
- Otherwise, if `username` is not in `config.project.collaborators`, return an error: `"unknown user '{username}'; valid collaborators: {list}"` where `{list}` is the collaborators joined by `", "`.

Wire the validation into both command handlers, after loading config and resolving the username, before calling `ticket::set_field()`:

**`apm/src/cmd/assign.rs`** — after `let config = Config::load(root)?;`, add:

    apm_core::validate::validate_owner(&config, username)?;

**`apm/src/cmd/set.rs`** — detect `field.as_str() == "owner"` and add the same call before `ticket::set_field()`. The config is available via `ctx.config`.

Export: `validate_owner` is added to the existing `validate` module already re-exported from `apm-core/src/lib.rs`.

**Tests** — add a `#[cfg(test)]` block in `apm-core/src/validate.rs` covering:
- `valid_collaborator_accepted`: collaborators `["alice", "bob"]`, call with `"alice"` → `Ok(())`.
- `unknown_user_rejected`: same config, call with `"charlie"` → `Err` whose message contains `"unknown user 'charlie'"` and `"alice, bob"`.
- `empty_collaborators_skips_validation`: `collaborators = []`, any username → `Ok(())`.
- `clear_owner_always_allowed`: collaborators `["alice"]`, username `"-"` → `Ok(())`.
- `github_mode_skips_validation`: `git_host.provider = Some("github")`, collaborators `["alice"]`, username `"charlie"` → `Ok(())`.

No changes to `ticket::set_field()` or the state machine. The `docs/ownership-spec.md` referenced in the original draft does not exist; ignore it.

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-04-08T15:09Z | — | new | philippepascal |
| 2026-04-08T15:33Z | new | groomed | apm |
| 2026-04-08T16:02Z | groomed | in_design | philippepascal |
| 2026-04-08T16:05Z | in_design | specd | claude-0408-1602-1f60 |
| 2026-04-08T21:47Z | specd | ready | apm |
| 2026-04-08T22:15Z | ready | in_progress | philippepascal |
