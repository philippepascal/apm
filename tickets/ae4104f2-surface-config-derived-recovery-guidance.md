+++
id = "ae4104f2"
title = "Surface config-derived recovery guidance for merge-failure states in apm CLI"
state = "specd"
priority = 0
effort = 5
risk = 2
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/ae4104f2-surface-config-derived-recovery-guidance"
created_at = "2026-05-30T02:11:03.737221Z"
updated_at = "2026-05-30T02:21:34.063369Z"
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
- [ ] Each `RecoveryOption` carries: to-state ID, display label (from `transition.label`, falling back to to-state ID when label is empty), and `RecoveryKind`
- [ ] Results are ordered by `workflow.states` declaration order; classification is independent of that order (shuffling the states list produces identical results)
- [ ] Against the default workflow, `classify_recovery_options("merge_failed", config)` returns `implemented` as `RetryMerge` and `in_progress` as `ReturnToWorker`
- [ ] Against a workflow where the merge-target state is renamed (e.g. `implemented` → `shipped`), the helper classifies `shipped` as `RetryMerge`
- [ ] When the queried state has no transitions to merge-target states, `classify_recovery_options` returns no `RetryMerge` entries
- [ ] `apm show <id>` prints a "Recovery options" block when the ticket's current state has any `RetryMerge` transitions OR the ticket body contains a section headed "Merge notes"
- [ ] The recovery block in `apm show` lists each option with its display label and the exact command `apm state <id> <to>`, and includes a reference to `docs/merge-failed-recovery.md`
- [ ] `apm show <id>` does not print a recovery block when no `RetryMerge` transitions exist and the body contains no "Merge notes" section
- [ ] `apm list --state <STATE>` appends a one-line recovery summary below ticket rows when STATE has `RetryMerge` transitions; omits the summary otherwise
- [ ] `apm next` (plain-text mode) prints recovery options below the ticket line when the selected ticket's state has `RetryMerge` transitions; JSON mode output is unchanged

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
```

Implementation of `classify_recovery_options`:

1. Build `merge_target_ids: HashSet<String>` — collect `transition.to` for every transition in the entire workflow whose `completion` is `Pr`, `Merge`, or `PrOrEpicMerge`.
2. Build `coder_start_ids: HashSet<String>` — collect `transition.to` for every transition with `trigger == "command:start"` and a `worker_profile` that does not end with `/spec-writer` (mirrors the existing logic in `implementation_state_ids`).
3. Locate the `StateConfig` for `state_id`; return an empty vec if not found.
4. For each transition in that state's `transitions`, in declaration order:
   - `transition.to ∈ merge_target_ids` → `RetryMerge`
   - else `transition.to ∈ coder_start_ids` → `ReturnToWorker`
   - else to-state has `terminal: true` → `Abandon`
   - else → `Other`
   - Label: `transition.label` if non-empty, otherwise `transition.to`
5. Return the `Vec<RecoveryOption>`.

The two-set approach (rather than checking the transition's own `completion`) is necessary because transitions FROM a merge-failure state do not themselves carry a merging completion — they are plain manual hops to states that were previously reached via a merge. Classifying by to-state ancestry is order-independent and rename-safe.

Export via `pub mod recovery` in `apm-core/src/lib.rs`; re-export the public types at crate root.

Unit tests inline in `apm-core/src/recovery.rs`:
- `test_default_workflow_merge_failed`: build config from the default workflow; assert `implemented` → RetryMerge, `in_progress` → ReturnToWorker.
- `test_shuffled_order_same_classification`: same workflow with `[[workflow.states]]` entries reversed; assert identical classification results.
- `test_renamed_merge_target`: workflow where `implemented` is renamed to `shipped`; assert `shipped` → RetryMerge.
- `test_no_merge_transitions`: state with no transitions to merge-target states; assert result contains no RetryMerge entries.

#### `apm/src/cmd/show.rs`

After the existing ticket output, call `classify_recovery_options` on the ticket's current state. Also scan the ticket body for any markdown heading that starts with "Merge notes" (case-insensitive prefix match on `## ` or `### ` lines).

If either condition is true, print (blank line before, blank line after):

```
Recovery options:
  <label>  →  apm state <id> <to>
  ...

  See: docs/merge-failed-recovery.md
```

Within the block, list RetryMerge options first, then ReturnToWorker, then Abandon, then Other; within each group, preserve the order returned by `classify_recovery_options`.

#### `apm/src/cmd/list.rs`

When `--state <STATE>` is given, after printing all ticket rows call `classify_recovery_options(STATE, config)`. If any RetryMerge entries exist, print a blank line followed by:

```
Recovery: <label> → apm state <id> <to>  [<label2> → apm state <id> <to2> ...]
```

Use the literal text `<id>` as a placeholder (the list covers multiple tickets). Show only RetryMerge and ReturnToWorker options; omit Abandon and Other to keep the line concise. Print nothing if no RetryMerge entries exist.

#### `apm/src/cmd/next.rs`

In plain-text mode, after printing the selected ticket line, call `classify_recovery_options` on the ticket's current state. If any RetryMerge entries exist, print the recovery options in the same indented format used by `apm show` (omit the `docs/` pointer, which appears in full only on `apm show`).

JSON mode: no change.

#### Integration tests (`apm/tests/integration.rs`)

Three tests in a temp repo with the default workflow:
- Set a ticket to `merge_failed`, run `apm show <id>`: assert stdout contains "Recovery options:" and the string `apm state <id> implemented`.
- Same setup, run `apm list --state merge_failed`: assert stdout ends with a line containing "Recovery:".
- Same setup with the ticket at highest priority, run `apm next`: assert stdout contains "Recovery options:".

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-05-30T02:11Z | — | new | philippepascal |
| 2026-05-30T02:14Z | new | groomed | philippepascal |
| 2026-05-30T02:14Z | groomed | in_design | philippepascal |
| 2026-05-30T02:21Z | in_design | specd | claude |
