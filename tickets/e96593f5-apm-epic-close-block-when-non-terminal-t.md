+++
id = "e96593f5"
title = "apm epic close: block when non-terminal tickets exist; add --close-all to cascade"
state = "specd"
priority = 0
effort = 3
risk = 2
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/e96593f5-apm-epic-close-block-when-non-terminal-t"
created_at = "2026-05-31T03:26:36.317944Z"
updated_at = "2026-06-01T07:06:50.258313Z"
+++

## Spec

### Problem

`apm epic close` runs a quiescence check (no live workers, no tickets currently in
an active coding state) but does not verify that all tickets in the epic have
reached a terminal state. A ticket in `specd`, `new`, `groomed`, `blocked`
(pre-implementation), `in_design` (no live worker), or `question` passes
quiescence fine, yet the epic closes around it: the epic branch is deleted or a
PR is opened, and those tickets are silently orphaned — still carrying the epic's
ID in their `epic` frontmatter field but with no managed path forward.

The fix is a second guard, separate from quiescence, that enforces a fully closed
epic before the branch is touched. Without `--close-all` the command bails and
tells the supervisor which tickets still need attention. With `--close-all` it
cascades a force-close over safe tickets, but refuses to silently swallow tickets
in `blocked` or `question`, which represent open questions that would lose their
context if closed without review.

### Acceptance criteria

- [ ] `apm epic close <id>` succeeds unchanged when every ticket in the epic is in `closed` state.
- [ ] `apm epic close <id>` exits non-zero and prints a table of non-terminal tickets (id, state, title) when at least one ticket is non-terminal.
- [ ] The non-terminal bail message ends with `Re-run with --close-all to cascade close, or close them manually first.`
- [ ] `apm epic close <id> --close-all` exits non-zero and prints a table of offending tickets before closing anything when at least one ticket is in `blocked` or `question`.
- [ ] `apm epic close <id> --close-all` closes each non-terminal ticket and then closes the epic when all non-terminal tickets are in states other than `blocked`/`question`.
- [ ] `apm epic close <id> --close-all` prints `closing ticket #<id> ... done` for each ticket it closes.
- [ ] `apm epic close <id> --close-all` with a mix of `blocked` and closable tickets bails before modifying any ticket or the epic.
- [ ] The existing quiescence check (live workers, active coding states) still runs before the new non-terminal check in both paths.

### Out of scope

- `apm refresh-epic` cascade into ticket branches (separate ticket).
- The `refresh-epic --merge` push bug (separate ticket).
- Renaming or refactoring existing epic subcommands beyond adding `--close-all`.
- Changes to `apm sync`'s ready-to-close detection.
- Closing tickets that have live workers; that remains the quiescence check's responsibility.
- Determining whether "safe" states for cascade-close should be configurable; the set (`blocked`, `question`) is hard-coded for now.

### Approach

#### Step 1 — `apm-core/src/epic.rs`: add `EpicTicketInfo` and `non_terminal_epic_tickets()`

Add a plain struct (no derives needed beyond Debug):

```rust
pub struct EpicTicketInfo {
    pub id: String,
    pub state: String,
    pub title: String,
}
```

Add a public function:

```rust
pub fn non_terminal_epic_tickets(
    root: &Path,
    epic_id: &str,
    config: &crate::config::Config,
) -> Result<Vec<EpicTicketInfo>>
```

Implementation: call `crate::ticket::load_all_from_git`, filter to tickets whose
`epic` frontmatter equals `epic_id`, then filter to those whose state is NOT in
`config.terminal_state_ids()`. Return sorted by `id`. This is the single helper
that both guard paths use; no state-set knowledge leaks into the CLI layer.

Add three inline unit tests in the existing `#[cfg(test)]` block, using the same
`setup_repo()` and TOML constants already present:
- `non_terminal_epic_tickets_all_closed_returns_empty`
- `non_terminal_epic_tickets_mixed_returns_non_terminal`
- `non_terminal_epic_tickets_ignores_other_epics`

#### Step 2 — `apm/src/main.rs`: add `--close-all` to `EpicCommand::Close`

Change the `Close` variant from:
```rust
Close { id: String }
```
to:
```rust
Close {
    /// Epic ID (4–8 char hex prefix)
    id: String,
    /// Cascade-close all non-terminal tickets before closing the epic
    #[arg(long)]
    close_all: bool,
}
```

Update the `EpicCommand::Close` dispatch arm (the match arm that calls `cmd::epic::run_close`) to
destructure `close_all` and pass it:
```rust
Command::Epic { command: EpicCommand::Close { id, close_all } }
    => cmd::epic::run_close(&root, &id, close_all),
```

#### Step 3 — `apm/src/cmd/epic.rs`: update `run_close()`

Change signature to `pub fn run_close(root: &Path, id_arg: &str, close_all: bool) -> Result<()>`.

After the existing quiescence bail (the closing `}` of the `if !blockers.is_empty()` block), insert
the new guard immediately before step 4 (PR title derivation / `branch_to_title` call):

```rust
// Non-terminal check.
let non_terminal = apm_core::epic::non_terminal_epic_tickets(root, epic_id, &config)?;
if !non_terminal.is_empty() {
    if !close_all {
        let rows: String = non_terminal.iter()
            .map(|t| format!("  {:<8}  {:<13}  {}", t.id, t.state, t.title))
            .collect::<Vec<_>>()
            .join("\n");
        anyhow::bail!(
            "epic has {} non-terminal ticket(s):\n{}\nRe-run with --close-all to cascade close, or close them manually first.",
            non_terminal.len(), rows
        );
    }
    // --close-all: fail-fast on blocked/question first.
    let unsafe_tickets: Vec<_> = non_terminal.iter()
        .filter(|t| t.state == "blocked" || t.state == "question")
        .collect();
    if !unsafe_tickets.is_empty() {
        let rows: String = unsafe_tickets.iter()
            .map(|t| format!("  {:<8}  {:<13}  {}", t.id, t.state, t.title))
            .collect::<Vec<_>>()
            .join("\n");
        anyhow::bail!(
            "cannot cascade close: the following tickets require manual resolution:\n{}\nResolve them manually, then retry.",
            rows
        );
    }
    // Safe to cascade — fail-fast error handling.
    // If any close fails: bail immediately, leave already-closed tickets closed,
    // do not touch the epic. Re-running --close-all is safe because
    // non_terminal_epic_tickets() only returns tickets that are still non-terminal,
    // so successfully closed tickets are skipped on retry.
    let agent = apm_core::config::resolve_caller_name();
    for t in &non_terminal {
        print!("closing ticket #{} ... ", t.id);
        apm_core::ticket::close(root, &config, &t.id, None, &agent, false)
            .with_context(|| format!("failed to close ticket #{}", t.id))?;
        println!("done");
    }
}
```

**Error handling**: fail-fast is chosen over continue-on-error or roll-back.
Rationale: it is the simplest option and is effectively idempotent — tickets already
closed in a previous partial run will not appear in `non_terminal_epic_tickets()` on
retry, so the supervisor can fix the failing ticket and re-run without any cleanup.
If `apm_core::ticket::close` errors, the error is surfaced with the failing ticket's
ID via `.with_context(...)` and the epic is left open. No roll-back of earlier closes
is performed.

The rest of `run_close` (PR title derivation, already-merged check, push, PR) is unchanged.

#### Step 4 — Integration tests

Add tests to `apm/tests/integration.rs` (or a new `apm/tests/epic_close.rs`):

- `epic_close_no_flag_bails_on_non_terminal_ticket`: create a ticket in `specd`
  (no impl history, so quiescence passes), call `run_close(root, epic_id, false)`,
  assert `Err` whose message contains "non-terminal".

- `epic_close_all_bails_on_blocked_ticket`: create a ticket in `blocked` (no impl
  history), call `run_close(root, epic_id, true)`, assert `Err` message contains
  "blocked".

- `epic_close_all_bails_on_mixed_blocked_and_safe`: create one ticket in `specd`
  and one in `blocked`, call `run_close(root, epic_id, true)`, assert `Err` and
  that neither ticket was closed.

- `epic_close_all_closes_safe_tickets`: create a ticket in `specd`, call
  `run_close(root, epic_id, true)` — note: this will bail later when it tries
  to push/open a PR; catch the git error and verify the ticket's branch state
  was updated to `closed` before that point. Alternatively, stub the git push
  by using a bare remote in the temp repo.

For the last test, the simplest approach is to verify the ticket branch contains
`state = "closed"` immediately after the cascade (read it from git) before the
push step fails; or refactor just enough to make the cascade separately testable.
The first three tests (bail paths) are straightforward since they exit before any
git push.

### Open questions


### Amendment requests

- [x] Line number drift: the spec cites apm/src/cmd/epic.rs::run_close at lines 73-132, but the actual function spans lines 73-133 (off by one). Update the spec reference, or describe the location by symbol (the run_close function in apm/src/cmd/epic.rs) so future edits do not invalidate the spec.
- [x] Cascade close error handling is ambiguous in the pseudocode. The current shape propagates errors immediately, which means a mid-cascade failure leaves N tickets closed, M tickets orphaned, and the epic still open. Choose one explicit behaviour and document it: (a) fail-fast — bail on first error, do not close the epic, leave already-closed tickets closed; (b) continue-on-error — attempt to close all tickets, then bail with a list of failures, do not close the epic; (c) roll-back — undo any closes that succeeded if a later one fails. State which option is chosen, the rationale, and how progress is reported to the supervisor.

### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-05-31T03:26Z | — | new | philippepascal |
| 2026-06-01T02:52Z | new | groomed | philippepascal |
| 2026-06-01T02:57Z | groomed | in_design | philippepascal |
| 2026-06-01T03:01Z | in_design | specd | claude |
| 2026-06-01T03:06Z | specd | ammend | philippepascal |
| 2026-06-01T07:04Z | ammend | in_design | philippepascal |
| 2026-06-01T07:06Z | in_design | specd | claude |
