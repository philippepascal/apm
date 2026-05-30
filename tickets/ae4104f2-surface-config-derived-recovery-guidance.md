+++
id = "ae4104f2"
title = "Surface config-derived recovery guidance for merge-failure states in apm CLI"
state = "in_progress"
priority = 0
effort = 5
risk = 2
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/ae4104f2-surface-config-derived-recovery-guidance"
created_at = "2026-05-30T02:11:03.737221Z"
updated_at = "2026-05-30T04:05:49.090403Z"
+++

## Spec

### Problem

When a ticket lands in a merge-failure state (e.g. `merge_failed` in the default workflow, though the state name is project-configurable), the supervisor has no in-context guidance on how to proceed. `apm show` prints frontmatter and history without surfacing recovery options. `apm list` filtered to the failure state prints rows with no hint. `apm next` can surface a merge-failure ticket as actionable without explaining what action to take. The supervisor must either know the conventions from memory or consult external documentation.

With config-aware surfacing, the CLI derives recovery options directly from the workflow configuration: which transition retries the merge, which returns the ticket to a worker, and which abandons it. All labels and target state IDs come from config, enforcing the order-independence discipline established by tickets ada017c0 and 27439a80 — no state name is hardcoded anywhere in the output path.

### Acceptance criteria

- [ ] `classify_recovery_options(state_id, config)` classifies a transition as `RetryMerge` when its to-state is the target of at least one merging-completion transition (Pr, Merge, or PrOrEpicMerge) anywhere in the workflow
- [ ] `classify_recovery_options` classifies a transition as `ReturnToWorker` when its to-state is the target of at least one non-spec-writer `command:start` transition anywhere in the workflow
- [ ] `classify_recovery_options` classifies a transition as `Abandon` when its to-state has `terminal: true`
- [ ] `classify_recovery_options` classifies a transition as `Other` when none of the above apply
- [x] Each `RecoveryOption` carries: to-state ID, display label (from `transition.label`, falling back to to-state ID when label is empty), and `RecoveryKind`
- [x] Results are ordered by `workflow.states` declaration order; classification is independent of that order (shuffling the states list produces identical results)
- [x] Against the default workflow, `classify_recovery_options("merge_failed", config)` returns `implemented` as `RetryMerge` and `in_progress` as `ReturnToWorker`
- [x] Against a workflow where the merge-target state is renamed (e.g. `implemented` → `shipped`), the helper classifies `shipped` as `RetryMerge`
- [x] When the queried state has no transitions to merge-target states, `classify_recovery_options` returns no `RetryMerge` entries
- [x] `is_merge_failure_state(state_id, workflow)` returns true iff `state_id` equals `transition.on_failure` for at least one transition in the entire workflow whose `completion` is `Pr`, `Merge`, or `PrOrEpicMerge`; transitions with a missing or empty `on_failure` are skipped
- [x] `is_merge_failure_state` returns false for all non-failure states in the default workflow — `new`, `groomed`, `specd`, `ready`, `in_progress`, `implemented`, `closed` — and true only for `merge_failed`
- [x] `apm show <id>` prints a "Recovery options" block iff `is_merge_failure_state(ticket.state, workflow)` returns true; the block lists each option with its display label and the exact command `apm state <id> <to>`, and includes a reference to `docs/merge-failed-recovery.md`
- [ ] `apm show <id>` does not print a recovery block when `is_merge_failure_state(ticket.state, workflow)` returns false, including when the ticket is in `in_progress`
- [ ] `apm list --state <STATE>` appends a one-line recovery summary below ticket rows when `is_merge_failure_state(STATE, workflow)` returns true; omits the summary otherwise, including for `--state in_progress`
- [ ] `apm next` (plain-text mode) prints recovery options below the ticket line when `is_merge_failure_state(ticket.state, workflow)` returns true; JSON mode output is unchanged; no recovery options are printed when the selected ticket is in `in_progress` or any other non-failure state

### Out of scope

- Terminal hint printed immediately after `apm state` completes (deliberately dropped — the high-value surfaces are the ones the supervisor reaches during triage)
- `apm work` / dispatcher changes (the dispatcher already treats merge-failure states as supervisor-actionable)
- Auto-recovery, auto-retry, or action buttons of any kind
- `apm-server` / `apm-ui` surfaces (covered by a separate ticket)
- Adding or modifying `[[workflow.states]]` entries or transitions in `workflow.toml`
- Hardcoding any state name or state ID as a string literal in CLI output paths

### Approach

#### `apm-core/src/recovery.rs` (new file)

Define:

```rust
pub enum RecoveryKind { RetryMerge, ReturnToWorker, Abandon, Other }
pub struct RecoveryOption { pub to: String, pub label: String, pub kind: RecoveryKind }
pub fn classify_recovery_options(state_id: &str, config: &WorkflowConfig) -> Vec<RecoveryOption>
pub fn is_merge_failure_state(state_id: &str, workflow: &WorkflowConfig) -> bool
```

**`is_merge_failure_state`**: iterate every `StateConfig` in `workflow.states`, then every `TransitionConfig` in each state. Return true iff `state_id == transition.on_failure` (as string equality) for any transition whose `completion` is `Pr`, `Merge`, or `PrOrEpicMerge`. Skip transitions where `on_failure` is `None` or the empty string. Return false if no match is found.

**`classify_recovery_options`**:

1. Build `merge_target_ids: HashSet<String>` — collect `transition.to` for every transition in the entire workflow whose `completion` is `Pr`, `Merge`, or `PrOrEpicMerge`.
2. Build `coder_start_ids: HashSet<String>` — collect `transition.to` for every transition with `trigger == "command:start"` and a `worker_profile` that does not end with `/spec-writer`.
3. Locate the `StateConfig` for `state_id`; return an empty vec if not found.
4. For each transition in that state's `transitions`, in declaration order:
   - `transition.to ∈ merge_target_ids` → `RetryMerge`
   - else `transition.to ∈ coder_start_ids` → `ReturnToWorker`
   - else to-state has `terminal: true` → `Abandon`
   - else → `Other`
   - Label: `transition.label` if non-empty, otherwise `transition.to`
5. Return the `Vec<RecoveryOption>`.

The two-set approach (rather than checking the transition's own `completion`) is necessary because transitions from a merge-failure state do not themselves carry a merging completion — they are plain manual hops to states that were previously reached via a merge. Classifying by to-state membership is order-independent and rename-safe.

Export via `pub mod recovery` in `apm-core/src/lib.rs`; re-export the public types at crate root.

Unit tests inline in `apm-core/src/recovery.rs`:
- `test_default_workflow_merge_failed`: build config from the default workflow; assert `implemented` → RetryMerge, `in_progress` → ReturnToWorker.
- `test_shuffled_order_same_classification`: same workflow with `[[workflow.states]]` entries reversed; assert identical classification results.
- `test_renamed_merge_target`: workflow where `implemented` is renamed to `shipped`; assert `shipped` → RetryMerge.
- `test_no_merge_transitions`: state with no transitions to merge-target states; assert result contains no RetryMerge entries.
- `test_is_merge_failure_state_default_workflow`: assert `is_merge_failure_state("merge_failed", &config)` is true; assert false for `"new"`, `"groomed"`, `"specd"`, `"ready"`, `"in_progress"`, `"implemented"`, and `"closed"`.
- `test_is_merge_failure_state_renamed`: workflow where the `on_failure` target is renamed from `merge_failed` to `pr_failed`; assert `is_merge_failure_state("pr_failed", &config)` is true and `is_merge_failure_state("merge_failed", &config)` is false.

#### `apm/src/cmd/show.rs`

In `print_ticket`, after the existing output, load the workflow config and call `is_merge_failure_state(&fm.state, &config.workflow)`. If true, call `classify_recovery_options(&fm.state, &config.workflow)` and print (blank line before, blank line after):

```
Recovery options:
  <label>  →  apm state <id> <to>
  ...

  See: docs/merge-failed-recovery.md
```

Within the block, list RetryMerge options first, then ReturnToWorker, then Abandon, then Other; within each group, preserve the order returned by `classify_recovery_options`. Do not print the block when `is_merge_failure_state` returns false. Do not scan the ticket body for "Merge notes" headings.

#### `apm/src/cmd/list.rs`

When `--state <STATE>` is given, after printing all ticket rows call `is_merge_failure_state(STATE, &ctx.config.workflow)`. If true, call `classify_recovery_options(STATE, &ctx.config.workflow)` and print a blank line followed by:

```
Recovery: <label> → apm state <id> <to>  [<label2> → apm state <id> <to2> ...]
```

Use the literal text `<id>` as a placeholder (the list covers multiple tickets). Show only RetryMerge and ReturnToWorker options; omit Abandon and Other. Print nothing when `is_merge_failure_state` returns false.

#### `apm/src/cmd/next.rs`

In plain-text mode, after printing the selected ticket line, call `is_merge_failure_state(&fm.state, &config.workflow)`. If true, call `classify_recovery_options` and print the recovery options in the same indented format used by `apm show` (omit the `docs/` pointer). JSON mode: no change.

#### Integration tests (`apm/tests/integration.rs`)

Three positive tests in a temp repo with the default workflow:
- Set ticket to `merge_failed`, run `apm show <id>`: assert stdout contains "Recovery options:" and the string `apm state <id> implemented`.
- Same setup, run `apm list --state merge_failed`: assert stdout contains "Recovery:".
- Same setup with ticket at highest priority, run `apm next`: assert stdout contains "Recovery options:".

Three negative tests (new):
- Ticket in `in_progress`, run `apm show <id>`: assert stdout does NOT contain "Recovery options:".
- Run `apm list --state in_progress`: assert stdout does NOT contain "Recovery:".
- Highest-priority ticket in `in_progress`, run `apm next`: assert stdout does NOT contain "Recovery options:".

### Open questions


### Amendment requests

- [x] REAL BUG: the spec uses 'state has a RetryMerge transition' as the trigger for surfacing recovery hints, but classify_recovery_options labels a transition as RetryMerge when its to-state is in merge_target_ids — and that includes the normal in_progress -> implemented transition. So under the spec as written, every in_progress ticket has a RetryMerge transition and would trigger the recovery block in apm show, the recovery summary in apm list --state in_progress, and the recovery note in apm next. That is over-fire on the most common state in the workflow.

The classifier itself is correct; only the consumer trigger is wrong. The right signal for 'this state is a merge-failure state' is set membership in 'states that appear as the on_failure value of some merging-completion transition'. That is order-independent, rename-safe, and precise (only fires on actual on_failure targets — merge_failed in the default workflow, whatever a custom workflow calls it).

REQUIRED CHANGES:
1. ADD a second helper in apm-core/src/recovery.rs alongside classify_recovery_options: pub fn is_merge_failure_state(state_id: &str, workflow: &WorkflowConfig) -> bool. It iterates every transition in workflow.states, returning true iff state_id equals transition.on_failure (as a string match) for any transition whose completion is Pr, Merge, or PrOrEpicMerge. Defensive on missing/empty on_failure: skip. Export from apm_core::recovery.

2. SWITCH the trigger for surfacing in all three CLI commands from 'has RetryMerge transitions' to 'is_merge_failure_state(current_state, workflow)':
   - apm show: print the Recovery options block iff is_merge_failure_state(ticket.state, workflow). The list of options inside the block still comes from classify_recovery_options(ticket.state, workflow). The body 'Merge notes' check is DROPPED entirely (it is stale signal after a ticket recovers; see below).
   - apm list --state STATE: print the recovery summary iff is_merge_failure_state(STATE, workflow).
   - apm next: print recovery options below the ticket line iff is_merge_failure_state(ticket.state, workflow). JSON mode unchanged.

3. DROP the body-section trigger from apm show. set_merge_notes writes the 'Merge notes' section when on_failure fires and nothing currently removes it on recovery, so the section persists indefinitely. Once is_merge_failure_state drives surfacing, the body section becomes incidental historical content, not a control signal. Showing the merge notes content inside apm show is fine if you want it, but it must not gate whether the recovery block prints.

4. ADD negative ACs:
   - is_merge_failure_state returns false for 'new', 'groomed', 'specd', 'ready', 'in_progress', 'implemented', 'closed', and any other normal state in the default workflow — only 'merge_failed' returns true.
   - apm show on an in_progress ticket prints no Recovery options block.
   - apm list --state in_progress prints no recovery summary.
   - apm next selecting an in_progress ticket prints no recovery options.

5. ADD negative tests mirroring the negative ACs. The current tests only assert the positive direction (merge_failed shows the block); the over-fire passes silently without negative coverage.

6. UPDATE the apm-server consumer (covered by ticket 778b63c6) — a parallel amendment is being filed there to use is_merge_failure_state for merge_failure_state_ids.

### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-05-30T02:11Z | — | new | philippepascal |
| 2026-05-30T02:14Z | new | groomed | philippepascal |
| 2026-05-30T02:14Z | groomed | in_design | philippepascal |
| 2026-05-30T02:21Z | in_design | specd | claude |
| 2026-05-30T02:44Z | specd | ammend | philippepascal |
| 2026-05-30T03:28Z | ammend | in_design | philippepascal |
| 2026-05-30T03:33Z | in_design | specd | claude |
| 2026-05-30T03:59Z | specd | ready | philippepascal |
| 2026-05-30T04:05Z | ready | in_progress | philippepascal |