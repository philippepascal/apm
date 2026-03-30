+++
id = "ebf77230"
title = "refactor: move validation logic from validate.rs and verify.rs into apm-core"
state = "ready"
priority = 0
effort = 3
risk = 2
author = "claude-0330-0245-main"
agent = "38718"
branch = "ticket/ebf77230-refactor-move-validation-logic-from-vali"
created_at = "2026-03-30T14:27:38.346647Z"
updated_at = "2026-03-30T16:58:39.809658Z"
+++

## Spec

### Problem

`validate.rs` (257 lines) and `verify.rs` (152 lines) contain validation logic
that belongs in `apm-core`:

**validate.rs** — config integrity checks:
- State ID reference validation (transitions reference valid states)
- Transition precondition and side-effect validation
- Instructions file existence checks
- Provider type validation for PR/Merge completion strategies
- Non-terminal dead-end detection

**verify.rs** — ticket consistency checks:
- Ticket state vs config state validation
- Filename/ID consistency
- Branch requirements by state
- Branch merge status checks
- Agent assignment validation
- Spec section presence checks
- Auto-fix for merged branches (state → accepted)

Both operate purely on data — config structs and ticket structs. `apm-serve`
will want to surface validation errors and consistency warnings in the UI
without shelling out.

Target: `apm_core::validate` and `apm_core::verify` modules. CLI wrappers
format and print results.

### Acceptance criteria

- [ ] `apm_core::validate::validate_config(config: &Config, root: &Path) -> Vec<String>` exists in `apm-core` and produces identical results to the current implementation in `apm/src/cmd/validate.rs`
- [ ] `apm_core::verify::verify_tickets(config: &Config, tickets: &[Ticket], merged: &HashSet<String>) -> Vec<String>` exists in `apm-core` and returns one issue string per consistency problem found
- [ ] `KNOWN_PRECONDITIONS` and `KNOWN_SIDE_EFFECTS` constants are defined in `apm-core::validate` (not in the CLI crate)
- [ ] All existing unit tests for `validate_config` pass after relocating them to `apm-core`
- [ ] `apm/src/cmd/validate.rs` calls `apm_core::validate::validate_config` and contains no duplicated validation logic
- [ ] `apm/src/cmd/verify.rs` calls `apm_core::verify::verify_tickets` for issue collection; `apply_fixes` and all output formatting stay in the CLI crate
- [ ] `cargo test --workspace` passes with no regressions

### Out of scope

- Creating an `apm-serve` binary or any HTTP/gRPC API surface — this ticket only moves the logic, not wires it into a server
- Moving `apply_fixes` to `apm-core` — it performs git I/O and calls CLI-internal helpers (`append_history`), so it stays in `apm/src/cmd/verify.rs`
- Adding new validation or verification rules beyond what already exists
- Changing the string format of any error/issue messages

### Approach

**1. Add `apm-core/src/validate.rs`**

Copy `validate_config` verbatim from `apm/src/cmd/validate.rs` into `apm-core/src/validate.rs`.
Move the `KNOWN_PRECONDITIONS` and `KNOWN_SIDE_EFFECTS` constants with it.
Move the `#[cfg(test)]` block too — the tests are pure unit tests with no CLI dependency and belong in `apm-core`.
Export the module from `apm-core/src/lib.rs`: `pub mod validate;`

**2. Add `apm-core/src/verify.rs`**

Extract the issue-collection loop from `apm/src/cmd/verify.rs::run` into a new function:

```rust
pub fn verify_tickets(
    config: &Config,
    tickets: &[Ticket],
    merged: &HashSet<String>,
) -> Vec<String>
```

This function collects the same issues the current `run` function prints, including the `doc.validate()` sub-checks. It takes a pre-computed `merged` set (caller is responsible for fetching it via `git::merged_into_main`).

The `in_progress_states` constant (`["in_progress", "implemented", "accepted"]`) moves into this function.

Export the module from `apm-core/src/lib.rs`: `pub mod verify;`

**3. Update `apm/src/cmd/validate.rs`**

- Delete the `validate_config` function body and the two constants.
- Add `use apm_core::validate::validate_config;` (re-export or direct call).
- Keep `run`, `apply_branch_fixes`, and all output-formatting logic.
- Keep tests that test CLI behaviour (JSON output format, `run` integration); pure logic tests moved to `apm-core`.

**4. Update `apm/src/cmd/verify.rs`**

- Delete the inline issue-collection loop from `run`.
- Call `apm_core::verify::verify_tickets(&config, &tickets, &merged_set)` to get `issues`.
- Keep `apply_fixes`, the completion-strategy reporting loop, and all `println!` calls.

**Order of steps**

1. Add `apm-core/src/validate.rs` (with tests) → compile-check `apm-core`
2. Update `apm/src/cmd/validate.rs` to delegate → compile-check `apm`
3. Add `apm-core/src/verify.rs` → compile-check `apm-core`
4. Update `apm/src/cmd/verify.rs` to delegate → compile-check `apm`
5. `cargo test --workspace`

**Known constraints**

- `verify_tickets` must not import anything from `apm/src` — it lives in `apm-core`.
- `HashSet<String>` (owned strings) is used for `merged` rather than `HashSet<&str>` to avoid lifetime entanglement with the caller's `Vec<String>` return from `git::merged_into_main`.
- The `doc.validate()` call inside the loop requires `Ticket::document()` — already part of `apm-core::ticket`; no new dependency needed.

### Open questions



### Amendment requests



### Code review



## History

| When | From | To | By |
|------|------|----|----|
| 2026-03-30T14:27Z | — | new | claude-0330-0245-main |
| 2026-03-30T14:29Z | new | in_design | claude-0330-0245-main |
| 2026-03-30T14:42Z | claude-0330-0245-main | philippepascal | handoff |
| 2026-03-30T16:27Z | in_design | new | philippepascal |
| 2026-03-30T16:33Z | new | in_design | philippepascal |
| 2026-03-30T16:36Z | in_design | specd | claude-0330-1645-b2e7 |
| 2026-03-30T16:58Z | specd | ready | philippepascal |
