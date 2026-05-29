+++
id = "7dab64ea"
title = "sync::detect must derive closeable states from config, not hardcoded IDs"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/7dab64ea-sync-detect-must-derive-closeable-states"
created_at = "2026-05-29T00:56:29.083955Z"
updated_at = "2026-05-29T00:58:24.606615Z"
+++

## Spec

### Problem

PROBLEM: apm-core/src/sync.rs hardcodes workflow state IDs in six places, all of which are config-defined values from workflow.toml [[workflow.states]] and are renamable/replaceable per project:
- line 28: const PRE_IMPL = [new, groomed, specd, question] (added by ticket 39b9c568)
- line 60: PRE_IMPL guard in Case 1 (branch merged)
- line 87: PRE_IMPL guard in Case 3 (branch content merged)
- line 105: Case 2 checks state == "implemented"
- line 126: Case 4 checks state != "implemented"
- line 150: hint pass checks state == "implemented"

A project that renames these states (or uses a different state vocabulary) silently loses correct behavior with no error. The codebase already shows the right pattern at line 27: let terminal = config.terminal_state_ids(); — terminal-ness is read from a config flag (terminal = true in workflow.toml), not a literal. Every state-based decision in sync.rs should be config-derived the same way.

GOAL: replace ALL hardcoded state-ID logic in sync.rs with config-derived determinations. No string-literal state IDs should remain in sync.rs.

MECHANISM (for the spec-writer to refine): the branch-merge close passes (Case 1, Case 3) and the implemented-specific passes (Case 2, Case 4, hint pass) all hinge on the same underlying notion: a ticket is eligible for merge-based auto-close only once it has reached the state produced by a merging completion. That state is config-derivable: it is the 'to' target of any transition whose completion strategy is a merging one (Merge / Pr / PrOrEpicMerge). In the default workflow that transition is in_progress -> implemented (completion = pr_or_epic_merge), so the eligible state is 'implemented'. Add a Config helper analogous to terminal_state_ids() (e.g. one that returns the set of states reached via a merging completion strategy) and use it everywhere sync currently hardcodes a state name. Case 2 and Case 4's 'implemented' checks and the hint pass's 'implemented' check should consult the same config-derived set rather than the literal.

SUPERVISOR DECISION (already made — do not re-litigate): sync should auto-close ONLY tickets that have reached a merge-completion state. This is STRICTER than the behavior ticket 39b9c568 left in place (which still closed any non-terminal merged ticket in states like in_design/ammend/ready/in_progress/blocked). That broadening is intended and correct: a ticket that never reached a merge-completion state has no APM-merged work, so any merge signal is a git-topology artifact. Cases 1 and 3 should therefore only produce close candidates for tickets in the config-derived merge-completed set (plus whatever the spec-writer determines for the genuinely-merged terminal/edge cases).

OUT OF SCOPE: changing the git merge-detection functions (merged_into_main, content_merged_into_main, is_branch_merged_into) — those are correct and unchanged. This is a pure config-derivation refactor of sync::detect plus a new Config helper.

TESTS: existing sync integration tests must still pass. New/updated tests must not hardcode state IDs in a way that bakes in the default workflow's names where avoidable; assert behavior via the config-derived helper. Confirm the side-note false-positive (ticket 39b9c568's scenario) stays fixed and that an implemented merged ticket is still closed.

### Acceptance criteria

- [ ] `Config::merge_completed_state_ids()` returns the set of state IDs that are the `to` target of any transition whose `completion` is `Pr`, `Merge`, or `PrOrEpicMerge`.
- [ ] For the default workflow, `merge_completed_state_ids()` returns `{"implemented"}`.
- [ ] `sync::detect` produces no close candidates for a ticket whose branch is merged into main when the ticket's state is not in `merge_completed_state_ids()` (e.g., `ready`, `in_progress`, `specd`).
- [ ] `sync::detect` produces a close candidate for a ticket in a merge-completed state whose branch is merged into main (Cases 1 and 3 continue to work).
- [ ] `sync::detect` produces a close candidate for a ticket in a merge-completed state on main with no surviving branch, and for one merged into a non-default `target_branch` (Cases 2 and 4 continue to work).
- [ ] `sync::detect` emits a hint only for tickets in the merge-completed state whose branch was not detected by any pass (hint pass continues to work).
- [ ] `sync.rs` contains no string-literal state IDs after the change.
- [ ] A custom-workflow project with a renamed merge-completion state (e.g., `shipped`) correctly triggers close detection for tickets in that renamed state and skips tickets not in that state.

### Out of scope

- Changes to the git merge-detection functions (`merged_into_main`, `content_merged_into_main`, `is_branch_merged_into`) — those are correct and unchanged.
- Adding new `CompletionStrategy` variants or changing existing ones.
- Any change to `apm-server` — it calls `sync::detect` but contains no hardcoded state IDs.
- Changes to the ticket state machine definition or workflow config format.
- Behaviour changes beyond the strictness tightening described in the Problem section (Cases 2, 4, and the hint pass are a pure substitution of the literal for the config-derived set; no other logic changes).

### Approach

#### Config helper — `apm-core/src/config.rs`

Add `merge_completed_state_ids()` to `impl Config`, directly after `terminal_state_ids()`. It walks every transition in the workflow and collects the `to` IDs of transitions whose `completion` is `Pr`, `Merge`, or `PrOrEpicMerge`. `CompletionStrategy::Pull` is excluded: pull brings upstream into the ticket branch, not the reverse, and does not represent implementation completion. `CompletionStrategy::None` is likewise excluded.

```rust
pub fn merge_completed_state_ids(&self) -> std::collections::HashSet<String> {
    self.workflow.states.iter()
        .flat_map(|s| s.transitions.iter())
        .filter(|t| matches!(t.completion,
            CompletionStrategy::Pr
            | CompletionStrategy::Merge
            | CompletionStrategy::PrOrEpicMerge
        ))
        .map(|t| t.to.clone())
        .collect()
}
```

Add a unit test in `config.rs`'s `#[cfg(test)]` block asserting that a config with a single `pr_or_epic_merge` transition targeting `"implemented"` returns `{"implemented"}`, and that a config with no merging transitions returns an empty set.

#### sync.rs — `apm-core/src/sync.rs`

Remove `const PRE_IMPL` (line 28). Add `let merge_completed = config.merge_completed_state_ids();` immediately after `let terminal = config.terminal_state_ids();`.

Make five substitutions:

1. **Case 1** (branch merged): Replace the two `continue` guards
   ```rust
   if terminal.contains(t.frontmatter.state.as_str()) { continue; }
   if PRE_IMPL.contains(&t.frontmatter.state.as_str()) { continue; }
   ```
   with a single guard:
   ```rust
   let state = t.frontmatter.state.as_str();
   if terminal.contains(state) || !merge_completed.contains(state) { continue; }
   ```

2. **Case 3** (content merged): Replace
   ```rust
   if !terminal.contains(state) && !PRE_IMPL.contains(&state) {
   ```
   with:
   ```rust
   if !terminal.contains(state) && merge_completed.contains(state) {
   ```

3. **Case 2** (on main, branch gone): Replace
   ```rust
   if t.frontmatter.state == "implemented" {
   ```
   with:
   ```rust
   if merge_completed.contains(t.frontmatter.state.as_str()) {
   ```

4. **Case 4** (merged into target_branch): Replace
   ```rust
   if t.frontmatter.state != "implemented" { continue; }
   ```
   with:
   ```rust
   if !merge_completed.contains(t.frontmatter.state.as_str()) { continue; }
   ```

5. **Hint pass**: Replace
   ```rust
   if t.frontmatter.state == "implemented" {
   ```
   with:
   ```rust
   if merge_completed.contains(t.frontmatter.state.as_str()) {
   ```

After these changes verify with `grep -n '"implemented"\|"new"\|"groomed"\|"specd"\|PRE_IMPL' apm-core/src/sync.rs` — the output should be empty.

#### Tests — `apm/tests/integration.rs`

Existing tests need no modifications. The two pre-impl regression tests (`sync_detect_skips_pre_impl_ticket_with_fork_in_main`, `sync_detect_implemented_ticket_still_closed_after_pre_impl_filter`) pass unchanged: `"new"` is not in `merge_completed`, `"implemented"` is.

Add three new tests:

1. **`sync_detect_skips_non_merge_completed_ticket_on_merged_branch`**: Force a ticket to `ready` state, merge its branch into main, assert the ticket does not appear in close candidates. Assert via `config.merge_completed_state_ids().contains("ready")` that `ready` is genuinely not in the set.

2. **`sync_detect_uses_config_derived_merge_completed_state`**: Build a minimal repo with a custom workflow TOML where the merge-completion state is `shipped` (not `implemented`), `completion = "merge"`. Force a ticket to `shipped`, merge its branch, assert it appears in close candidates. Also force a second ticket to `ready`, merge its branch, assert it does not appear.

3. **`merge_completed_state_ids_returns_correct_set`** (unit test in `config.rs`): Covered above in the Config helper step.

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-05-29T00:56Z | — | new | philippepascal |
| 2026-05-29T00:58Z | new | groomed | philippepascal |
| 2026-05-29T00:58Z | groomed | in_design | philippepascal |