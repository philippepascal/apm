+++
id = "071886fc"
title = "Workflow corrections: remove bad transitions, restructure ammend path"
state = "implemented"
priority = 0
effort = 3
risk = 1
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/071886fc-workflow-corrections-remove-bad-transiti"
created_at = "2026-05-31T02:57:20.412089Z"
updated_at = "2026-06-01T00:51:56.426601Z"
epic = "9c3c4c20"
target_branch = "epic/9c3c4c20-workflow-schema-cleanup-state-level-work"
depends_on = ["e05c0463"]
+++

## Spec

### Problem

The default `workflow.toml` (and the project's `.apm/workflow.toml`) contain three transitions that contradict the intended design:

**`in_design → ammend`**: A spec-writer encountering a blocker during design should route to `question`, not `ammend`. The `ammend` state is supervisor-initiated from `specd` or `implemented`; a spec-writer agent cannot self-request an amendment.

**`merge_failed → in_progress`**: When a merge fails, the correct recovery is to retry the merge (`merge_failed → implemented`) or escalate to the supervisor. Routing back to `in_progress` re-spawns the coder without new guidance and bypasses any supervisor review of the failure.

**`ammend → in_design` via `command:start`**: This creates two `command:start`-triggered paths into `in_design` (one from `groomed`, one from `ammend`), violating the trigger-uniqueness invariant planned for enforcement in the next ticket. The correct path is `ammend → groomed` (supervisor, manual), then `groomed → in_design` (command:start via `apm start`). The `ammend` state should become supervisor-actionable, since the only agent-dispatch path from it is being removed.

### Acceptance criteria

- [x] `apm state <id> ammend` on a ticket in `in_design` returns an error (transition not defined)
- [x] `apm state <id> in_progress` on a ticket in `merge_failed` returns an error (transition not defined)
- [x] `apm state <id> groomed` on a ticket in `ammend` succeeds and the ticket reaches `groomed`
- [x] `apm start` does not pick up tickets in `ammend` state (no `command:start` exits from `ammend`)
- [x] A ticket can traverse `specd → ammend → groomed → in_design → specd` without error
- [x] `apm list --actionable` does not include `ammend` tickets in the agent-actionable set
- [x] The spec-writer agent prompts no longer instruct the agent to run `apm state <id> in_design` from `ammend`
- [x] `cargo test --workspace` passes after all changes

### Out of scope

- Trigger-uniqueness validation rule (enforced in the next ticket in this epic, which depends on this one)
- Adding a `merge_failed → ready` escape hatch (only the `in_progress` removal is in scope)
- Removing the `ammend → specd` or `ammend → question` transitions (kept for supervisor edge-case use)
- The `specd → in_design` stale row in the `STATIC_STATE_MACHINE` table (unrelated cleanup)
- Help text or documentation sweep (separate ticket)
- Any code changes — this is a pure content change to TOML and Markdown files
- Changes to the `actionable` field: ticket f7340b57 drops the field entirely from `StateConfig` and removes all `actionable = [...]` lines from both workflow.toml files. No `actionable` change is needed in this ticket.

### Approach

#### workflow.toml changes (both files)

Apply the same three edits to `apm-core/src/default/workflow.toml` and `.apm/workflow.toml`:

1. **Remove** the `[[workflow.states.transitions]]` block under the `in_design` state with `to = "ammend"`.

2. **Remove** the `[[workflow.states.transitions]]` block under the `merge_failed` state with `to = "in_progress"`.

3. Under the `ammend` state, replace the `[[workflow.states.transitions]]` block that has `to = "in_design"` / `trigger = "command:start"` / `worker_profile = "claude/spec-writer"` with:
   ```toml
   [[workflow.states.transitions]]
   to      = "groomed"
   trigger = "manual"
   outcome = "needs_input"
   ```

Note: the `actionable` field on the `ammend` state header (`actionable = ["agent"]`) is removed by ticket f7340b57 which lands before this ticket. No `actionable` change is needed here.

#### instructions.rs static table

In `apm-core/src/instructions.rs`, the `STATIC_STATE_MACHINE` const is defined on line 11. On **line 22**, change:
```
| ammend | in_design | apm state <id> in_design |
```
to:
```
| ammend | groomed | apm state <id> groomed |
```

#### Agent prompt updates

**`apm-core/src/default/agents/claude/apm.spec-writer.md`** (the "Handling `ammend` tickets" section, lines 203–219):

The section currently reads:
```
When the ticket is in `ammend` state:
1. `apm show <id>` — read `### Amendment requests` in `## Spec` carefully; …
2. `apm state <id> in_design` — claim the ticket and provision its worktree; …
3. For each checkbox, make the requested change …
4. Update `### Approach` if the amendments change the implementation plan
5. Do not delete answered questions …
6. `apm spec` auto-commits …
7. `apm state <id> specd` — resubmit …
```

Apply these changes:
- Change the opening condition from "When the ticket is in `ammend` state:" to "When the ticket has unchecked items in `### Amendment requests`, you are handling an amendment. You are already in `in_design` when dispatched (the supervisor moved the ticket from `ammend → groomed`, then `apm start` dispatched via `groomed → in_design`):"
- Remove step 2 (`apm state <id> in_design — claim the ticket and provision its worktree; prints the worktree path`).
- Renumber the remaining steps (old step 3 becomes step 2, etc.).
- After the renumbered final step (old step 7, new step 6: `apm state <id> specd`), add a blank line and then: "If you cannot proceed during design, transition to `question`. Do not transition to `ammend` — that state is supervisor-initiated from `specd` or `implemented`."

**`.apm/agents/pi/apm.spec-writer.md`** (the "Ammend tickets" section, around line 156):

The section currently opens at **line 158** with:
```
If the ticket starts in state `ammend` instead of `in_design`:
```

Replace that opening line with:
```
If `### Amendment requests` has unchecked items, the ticket is an amendment. You are already in `in_design` when dispatched.
```

No other changes are needed in this file — it has no step instructing the agent to claim from `ammend`.

#### Test changes (`apm/tests/integration.rs`)

**Delete** `spawn_ammend_ticket_transitions_to_in_design` (lines 2079–2093). It verifies that `apm start` picks up an `ammend` ticket via `command:start`, which no longer exists.

**Add** `ammend_to_groomed_succeeds` after the deleted test:
```rust
#[test]
fn ammend_to_groomed_succeeds() {
    let dir = init_repo();
    let p = dir.path();
    let (id, branch) = write_ticket_to_branch(p, "ammend", "needs revision");
    let rel = ticket_rel_path(&branch);

    apm::cmd::state::run(p, &id, "groomed".into(), false, false).unwrap();
    let content = branch_content(p, &branch, &rel);
    assert!(
        content.contains("state = \"groomed\""),
        "expected groomed state:\n{content}"
    );
}
```

Note: `write_ticket_to_branch` uses `apm state --force` internally to advance state, so it can place a ticket in `ammend` without needing the full `groomed → in_design → specd → ammend` lifecycle. `init_repo()` (not `setup_with_merge_workflow()`) is sufficient here — no merge completion logic is needed.

**Delete** `merge_failed_to_in_progress_succeeds` (lines 6591–6616).

**Add** `merge_failed_to_in_progress_rejected` after the deleted test, mirroring the fixture setup of the deleted test but asserting failure. Use `init_repo()` so the default workflow applies (without `merge_failed → in_progress`):
```rust
#[test]
fn merge_failed_to_in_progress_rejected() {
    let dir = init_repo();
    let p = dir.path();
    let (id, branch) = write_ticket_to_branch(p, "merge_failed", "retry rejected");
    let rel = ticket_rel_path(&branch);

    let result = apm::cmd::state::run(p, &id, "in_progress".into(), false, false);
    assert!(result.is_err(), "expected merge_failed → in_progress to be rejected");
    let msg = result.unwrap_err().to_string();
    assert!(
        msg.contains("no transition"),
        "expected 'no transition' in error message: {msg}"
    );
    // State must not have changed.
    let content = branch_content(p, &branch, &rel);
    assert!(content.contains("state = \"merge_failed\""), "state should remain merge_failed:\n{content}");
}
```

The error text `"no transition"` comes from the `bail!("no transition from {:?} to {:?} …")` in `apm-core/src/state.rs` line 64.

### Open questions


### Amendment requests

- [x] Provide concrete test code patterns for the new and replaced tests. Specify how to construct an ammend ticket fixture in tests, mirroring how existing merge_failed tests build their fixtures. Specify which error type or message to assert for the rejected merge_failed to in_progress transition path.
- [x] Add an explicit step listing every file beyond workflow.toml that references the removed transitions: apm-core/src/instructions.rs static state machine table around line 22 (shows ammend to in_design), apm-core/src/default/agents/claude/apm.spec-writer.md around lines 205 to 209, .apm/agents/pi/apm.spec-writer.md around line 158. Each file needs its specific update.
- [x] Reconcile the actionable field references in this ticket with f7340b57. By the time 071886fc lands, f7340b57 has dropped the actionable field. Verify that ammend's actionable change mentioned in the original spec is no longer needed; remove any references to actionable in this ticket since the field is gone.

### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-05-31T02:57Z | — | new | philippepascal |
| 2026-05-31T07:04Z | new | groomed | philippepascal |
| 2026-05-31T07:20Z | groomed | in_design | philippepascal |
| 2026-05-31T07:26Z | in_design | specd | claude |
| 2026-05-31T19:35Z | specd | ammend | philippepascal |
| 2026-05-31T19:38Z | ammend | in_design | philippepascal |
| 2026-05-31T19:43Z | in_design | specd | claude |
| 2026-05-31T21:04Z | specd | ready | philippepascal |
| 2026-06-01T00:44Z | ready | in_progress | philippepascal |
| 2026-06-01T00:51Z | in_progress | implemented | claude |
