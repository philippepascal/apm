+++
id = "7502e379"
title = "add a force flag to apm assign"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/7502e379-add-a-force-flag-to-apm-assign"
created_at = "2026-04-08T23:57:24.004823Z"
updated_at = "2026-04-09T00:10:24.252918Z"
+++

## Spec

### Problem

The `apm assign` command currently only allows the current ticket owner to reassign ownership. Any attempt by a non-owner to run `apm assign <id> <user>` fails with "only the current owner (<owner>) can reassign this ticket". There is no escape hatch for administrators or collaborators who need to take over or hand off a ticket when the current owner is unavailable.

A `--force` flag would let any collaborator override the ownership check, while a confirmation prompt prevents accidental overrides by requiring explicit acknowledgement of the current owner before proceeding.

### Acceptance criteria

- [ ] `apm assign --force <id> <user>` succeeds when the current user is not the ticket owner
- [ ] When `--force` is used and the ticket has an existing owner, a prompt shows "Ticket <id> is currently owned by <owner>. Reassign to <user>? [y/N]" before proceeding
- [ ] Entering `y` or `Y` at the prompt completes the assignment
- [ ] Entering anything other than `y`/`Y` (including empty input) aborts with message "aborted" and leaves the ticket unchanged
- [ ] `--force` on an unowned ticket proceeds without showing a confirmation prompt
- [ ] `--force` does not bypass the terminal-state guard — `apm assign --force <id> <user>` on a closed ticket still errors with "cannot change owner of a closed ticket"
- [ ] `--force` still validates the target username against the configured collaborators list
- [ ] Without `--force`, the existing behaviour is unchanged: a non-owner gets the error "only the current owner (<owner>) can reassign this ticket"

### Out of scope

- A separate `--yes` / `-y` flag to skip the confirmation prompt non-interactively (not requested)
- Bypassing the terminal-state check (closed tickets remain immutable)
- Changes to any command other than `apm assign`
- Audit logging of forced reassignments

### Approach

### `apm/src/cmd/assign.rs`

Add a `force: bool` parameter to `run()`. Replace the current unconditional `ticket::check_owner(root, t)?` call with a branch:

```rust
if force {
    // Terminal-state guard still applies
    let cfg = Config::load(root)?;
    let is_terminal = cfg.workflow.states.iter()
        .find(|s| s.id == t.frontmatter.state)
        .map(|s| s.terminal)
        .unwrap_or(false);
    if is_terminal {
        bail!("cannot change owner of a closed ticket");
    }
    // Confirm only when there is a current owner to displace
    if let Some(current_owner) = &t.frontmatter.owner.clone() {
        print!("Ticket {id} is currently owned by {current_owner}. Reassign to {username}? [y/N] ");
        io::stdout().flush()?;
        let mut line = String::new();
        io::stdin().lock().read_line(&mut line)?;
        if !line.trim().eq_ignore_ascii_case("y") {
            println!("aborted");
            return Ok(());
        }
    }
} else {
    ticket::check_owner(root, t)?;
}
```

Add imports `use std::io::{self, Write, BufRead};` (check whether they are already present).

For testability, extract the inner logic into a private `run_inner(root, id_arg, username, no_aggressive, force, confirm_override: Option<bool>)` where `None` uses the interactive stdin prompt and `Some(b)` short-circuits to `b`. The public `run()` calls `run_inner(..., None)`. Integration tests call `run_inner(..., Some(true))` or `Some(false)`.

### `apm/src/main.rs`

Add `--force` to the `assign` subcommand arg definition (a boolean flag, no value). Pass it into `assign::run()`.

### `apm/tests/integration.rs`

Add three tests (all calling `run_inner` directly to avoid stdin):

1. `assign_force_succeeds_when_not_owner` — create ticket, assign to `alice`, then call force-assign to `bob` with `confirm_override: Some(true)`; assert owner becomes `bob`.
2. `assign_force_aborts_on_deny` — same setup, call with `confirm_override: Some(false)`; assert owner remains `alice` and call returns `Ok(())`.
3. `assign_force_skips_prompt_when_no_owner` — create ticket (no owner), call force-assign with `confirm_override: Some(true)`; assert owner is set. (Ensures unowned path doesn't prompt.)

No changes to `apm-core/src/ticket.rs` — `check_owner` is left intact; the bypass lives entirely in the CLI layer.

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-04-08T23:57Z | — | new | philippepascal |
| 2026-04-08T23:57Z | new | groomed | apm |
| 2026-04-09T00:10Z | groomed | in_design | philippepascal |