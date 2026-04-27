+++
id = "056b1ee1"
title = "Require epic quiescence in apm epic close"
state = "in_design"
priority = 0
effort = 2
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/056b1ee1-require-epic-quiescence-in-apm-epic-clos"
created_at = "2026-04-27T20:29:06.958516Z"
updated_at = "2026-04-27T21:36:34.893454Z"
epic = "5ea30227"
target_branch = "epic/5ea30227-strategy-and-dependency-hardening"
depends_on = ["2973e208"]
+++

## Spec

### Problem

`apm epic close` currently gates on a state check: it refuses if any epic ticket is not in a `satisfies_deps: true` or `terminal` state. This check is too narrow — it does not account for live worker processes and does not use the shared quiescence definition established by ticket 2973e208.

The spec at `docs/strategy-and-dependencies.md` (§ 'Refresh and close: epic must be quiescent') requires the epic to be fully quiescent before the close PR is opened: no ticket may be in an active, non-terminal state, and no ticket may have a live worker process. Ticket 2973e208 adds `epic_is_quiescent()` in `apm-core/src/epic.rs` as the canonical helper for this check, used by both `apm refresh-epic` and `apm epic close`.

This ticket wires that helper into `run_close`, replacing the existing bespoke gate logic with a single call to `epic_is_quiescent()`.

### Acceptance criteria

- [ ] `apm epic close <id>` refuses when any epic ticket is in a non-terminal, non-`worker_end` state
- [ ] `apm epic close <id>` refuses when any epic ticket has a live worker process
- [ ] The refusal message begins with `"cannot close epic: the following tickets are not quiescent:"` followed by one line per blocker
- [ ] Each blocker line names the ticket ID, title, and either its state or `"live worker"`
- [ ] `apm epic close <id>` succeeds (pushes branch and opens PR) when all epic tickets are in terminal or `worker_end` states and no live workers exist
- [ ] An epic with zero tickets passes the quiescence check and proceeds to PR creation

### Out of scope

- Adding `epic_is_quiescent()` itself — that is ticket 2973e208
- The `apm refresh-epic` command — ticket 2973e208
- Auto-killing workers or auto-transitioning tickets before close
- Any changes to `apm validate`, dependency rules, or strategy enforcement
- Changing the PR creation logic or the `gh_pr_create_or_update` call
- Removing or changing the per-epic `max_workers` override (ticket 6e3f9e91)
- Changing the default completion strategy (ticket 941e57fa)

### Approach

Single file changes: **`apm/src/cmd/epic.rs`**, function `run_close`.

**Remove** the existing gate check (the `not_ready` vec, the loop over `epic_tickets`, and the `anyhow::bail!` for not-ready tickets — currently lines 85–105). Also remove the `tickets` and `epic_tickets` bindings that were only used by that check (steps 3–4 in the current code).

**Replace** with the following, inserted after `epic_id` is resolved (after step 2) and before the PR-title derivation (step 5):

```rust
let worktrees = apm_core::worktree::list_ticket_worktrees(root)?;
let blockers = apm_core::epic::epic_is_quiescent(root, epic_id, &config, &worktrees)?;
if !blockers.is_empty() {
    anyhow::bail!(
        "cannot close epic: the following tickets are not quiescent:\n{}",
        blockers.join("\n")
    );
}
```

Steps 5–6 (derive title, push branch, call `gh_pr_create_or_update`) are unchanged.

**Constraints:**
- `epic_is_quiescent` is defined in ticket 2973e208. This ticket must not be implemented until that dependency is merged; the function signature is `pub fn epic_is_quiescent(root: &Path, epic_id: &str, config: &Config, worktrees: &[(PathBuf, String)]) -> Result<Vec<String>>`.
- The error message prefix must match `"cannot close epic: the following tickets are not quiescent:"` exactly, to stay consistent with the analogous message in `run_refresh_epic` (`"cannot refresh epic: the following tickets are not quiescent:"`).
- No new tests are required in `apm/` — `epic_is_quiescent` carries its own unit tests (ticket 2973e208). An integration-level smoke test verifying the bail path is welcome but not required.

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-04-27T20:29Z | — | new | philippepascal |
| 2026-04-27T20:44Z | new | groomed | philippepascal |
| 2026-04-27T21:32Z | groomed | in_design | philippepascal |