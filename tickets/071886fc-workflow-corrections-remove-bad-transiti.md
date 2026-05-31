+++
id = "071886fc"
title = "Workflow corrections: remove bad transitions, restructure ammend path"
state = "ammend"
priority = 0
effort = 3
risk = 1
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/071886fc-workflow-corrections-remove-bad-transiti"
created_at = "2026-05-31T02:57:20.412089Z"
updated_at = "2026-05-31T19:35:59.576066Z"
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

- [ ] `apm state <id> ammend` on a ticket in `in_design` returns an error (transition not defined)
- [ ] `apm state <id> in_progress` on a ticket in `merge_failed` returns an error (transition not defined)
- [ ] `apm state <id> groomed` on a ticket in `ammend` succeeds and the ticket reaches `groomed`
- [ ] `apm start` does not pick up tickets in `ammend` state (no `command:start` exits from `ammend`)
- [ ] A ticket can traverse `specd → ammend → groomed → in_design → specd` without error
- [ ] `apm list --actionable` does not include `ammend` tickets in the agent-actionable set
- [ ] The spec-writer agent prompts no longer instruct the agent to run `apm state <id> in_design` from `ammend`
- [ ] `cargo test --workspace` passes after all changes

### Out of scope

- Trigger-uniqueness validation rule (enforced in the next ticket in this epic, which depends on this one)
- Adding a `merge_failed → ready` escape hatch (only the `in_progress` removal is in scope)
- Removing the `ammend → specd` or `ammend → question` transitions (kept for supervisor edge-case use)
- The `specd → in_design` stale row in the `STATIC_STATE_MACHINE` table (unrelated cleanup)
- Help text or documentation sweep (separate ticket)
- Any code changes — this is a pure content change to TOML and Markdown files

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

4. On the `ammend` state header, change `actionable = ["agent"]` to `actionable = ["supervisor"]`. Removing the only `command:start` exit means `apm start` will no longer dispatch from `ammend`; keeping it `agent`-actionable would produce misleading `apm list` output.

#### instructions.rs static table

In `apm-core/src/instructions.rs`, in `STATIC_STATE_MACHINE`, change:
```
| ammend | in_design | apm state <id> in_design |
```
to:
```
| ammend | groomed | apm state <id> groomed |
```

#### Agent prompt updates

**`apm-core/src/default/agents/claude/apm.spec-writer.md`** and **`.apm/agents/claude/apm.spec-writer.md`** (same content in both):

In the "Handling `ammend` tickets" section:
- Remove step 2 (`apm state <id> in_design — claim the ticket…`). The agent is already in `in_design` when dispatched; the supervisor moved the ticket from `ammend → groomed` first, then `apm start` dispatched the agent via `groomed → in_design`.
- Renumber the remaining steps.
- Add a note: "If you are in `in_design` and cannot proceed, transition to `question`. Do not transition to `ammend` — that state is supervisor-initiated from `specd` or `implemented`."

**`.apm/agents/pi/apm.spec-writer.md`**:

In the "Ammend tickets" section, the opening condition says "If the ticket starts in state `ammend` instead of `in_design`". Replace this with: "If `### Amendment requests` has unchecked items, the ticket is an amendment. You are already in `in_design`." Remove any instructions to claim the ticket from `ammend` (there are none in the pi prompt, so this is a wording fix only).

#### Test changes (`apm/tests/integration.rs`)

- **Delete** `spawn_ammend_ticket_transitions_to_in_design` (lines ~2079–2093). It verifies that `apm start` picks up an `ammend` ticket via `command:start`, which no longer exists.
- **Add** `ammend_to_groomed_succeeds`: create a ticket in `ammend` state via direct branch write, call `apm::cmd::state::run(p, &id, "groomed", false, false)`, assert the resulting ticket content contains `state = "groomed"`.
- **Delete** `merge_failed_to_in_progress_succeeds` (lines ~6591–6616).
- **Add** `merge_failed_to_in_progress_rejected`: create a ticket in `merge_failed` state, call `apm::cmd::state::run(p, &id, "in_progress", false, false)`, assert it returns `Err`.

### Open questions


### Amendment requests

- [ ] Provide concrete test code patterns for the new and replaced tests. Specify how to construct an ammend ticket fixture in tests, mirroring how existing merge_failed tests build their fixtures. Specify which error type or message to assert for the rejected merge_failed to in_progress transition path.
- [ ] Add an explicit step listing every file beyond workflow.toml that references the removed transitions: apm-core/src/instructions.rs static state machine table around line 22 (shows ammend to in_design), apm-core/src/default/agents/claude/apm.spec-writer.md around lines 205 to 209, .apm/agents/pi/apm.spec-writer.md around line 158. Each file needs its specific update.
- [ ] Reconcile the actionable field references in this ticket with f7340b57. By the time 071886fc lands, f7340b57 has dropped the actionable field. Verify that ammend's actionable change mentioned in the original spec is no longer needed; remove any references to actionable in this ticket since the field is gone.

### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-05-31T02:57Z | — | new | philippepascal |
| 2026-05-31T07:04Z | new | groomed | philippepascal |
| 2026-05-31T07:20Z | groomed | in_design | philippepascal |
| 2026-05-31T07:26Z | in_design | specd | claude |
| 2026-05-31T19:35Z | specd | ammend | philippepascal |