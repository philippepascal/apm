+++
id = 41
title = "Configurable merge strategy for implemented→accepted"
state = "ready"
priority = 4
effort = 3
risk = 2
author = "claude-0328-1000-a1b2"
branch = "ticket/0041-configurable-merge-strategy-for-implemen"
created_at = "2026-03-28T08:14:53.494909Z"
updated_at = "2026-03-29T20:49:30.393183Z"
+++

## Spec

### Problem

The `implemented → accepted` transition is currently hardcoded to require an
open GitHub PR (`preconditions = ["pr_exists"]`) and relies on the GitHub
provider to detect when it is merged. This works for teams using GitHub PR
reviews, but is the wrong default for many workflows:

- A solo developer or small team may want a **direct merge** — push the branch,
  merge it locally, move on — with no PR required.
- A team with its own CI/CD or review tooling may want **none** — apm does
  nothing after `implemented`, and the user's external process handles the rest
  and transitions manually.

The merge strategy should be a per-transition config option in `apm.toml`, not
baked into the precondition list of a workflow state. This is already modelled
by the `completion` property on `[[workflow.states.transitions]]` (added in
ticket #53).

### Acceptance criteria

- [x] `completion` on a transition supports three values: `"pr"`, `"merge"`, `"none"`
  (already added to the config schema in ticket #53 — this ticket implements
  the runtime behaviour)
- [ ] When `apm state <id> <to>` is called and the matching transition has
  `completion = "pr"`: push the ticket branch and open (or update) a GitHub PR
  targeting the default branch
- [ ] When `apm state <id> <to>` has `completion = "merge"`: push the ticket
  branch, then merge it into the default branch locally and push the default
  branch; transition succeeds immediately; if the merge produces a conflict,
  abort and print a clear message (ticket stays in current state)
- [ ] When `completion = "none"` (or absent): `apm state` does only the state
  change — no push, no PR (existing behaviour, no regression)
- [ ] Default is `"none"` if `completion` is omitted from a transition
- [ ] `apm verify` reports the `completion` value for each transition that has
  one set

### Out of scope

- Squash or rebase merge options (always a standard merge commit for now)
- Per-ticket override of the completion strategy
- `"agent"` review strategy — deferred to a separate ticket
- GitLab / Bitbucket MR support (tracked separately)
- Auto-transitioning to `accepted` after `"merge"` completes — that is handled
  by `apm sync` detecting the merge

### Approach

The `completion` field is already parsed by `apm-core/src/config.rs` as
`CompletionStrategy` on `TransitionConfig` (ticket #53). This ticket wires
it to runtime behaviour in `apm state`.

**`apm/src/cmd/state.rs`** — after writing the new ticket state to the branch,
read the matched transition's `completion` field and act:

```rust
match transition.completion {
    CompletionStrategy::Pr => {
        git::push_branch(root, &branch)?;
        gh_pr_create_or_update(root, &branch, &config.project.default_branch, &ticket)?;
    }
    CompletionStrategy::Merge => {
        git::push_branch(root, &branch)?;
        merge_into_default(root, &branch, &config.project.default_branch)?;
    }
    CompletionStrategy::None => {}
}
```

`gh_pr_create_or_update`: checks for an existing open PR (`gh pr list --head <branch>`); creates with `gh pr create` if none exists, otherwise updates with `gh pr edit`. Uses the ticket title as PR title and a body linking to the ticket ID.

`merge_into_default`: checks out the default branch worktree (or uses `git -C <default-branch-wt>`), runs `git merge --no-ff <branch>`, then `git push`. On non-zero exit from merge: runs `git merge --abort`, prints "merge conflict — resolve manually", returns an error (state change was already committed to the ticket branch so it can be retried).

**`apm/src/cmd/verify.rs`** — already iterates transitions; add a line printing `completion = <value>` for any transition where it is set to `pr` or `merge`.

### Amendment requests

- [x] The design has changed: merge strategy is no longer a global `[workers]`
  config. It is now a `completion` property on individual
  `[[workflow.states.transitions]]` entries (e.g. `completion = "pr"` on
  `in_progress → implemented`). This allows different transitions to use
  different strategies. Rewrite the entire spec around `completion` on
  transitions instead of `[workers] merge_strategy`.
- [x] Replace `"pr"` / `"direct"` / `"agent"` / `"none"` values with the
  canonical set from TICKET-LIFECYCLE: `"pr"`, `"merge"`, `"none"`. The
  `"agent"` review strategy is deferred to a separate ticket.
- [x] The config section `[workers]` is no longer the right home. The
  `completion` property lives on the transition definition in `apm.toml`.
  Update all config structs and field names accordingly.
- [x] Update the approach section to reflect that `apm state` (not `apm start`)
  reads `completion` and performs the push/PR when transitioning.

## History

| When | From | To | By |
|------|------|----|----|
| 2026-03-28T08:14Z | — | new | claude-0328-1000-a1b2 |
| 2026-03-28T08:16Z | new | specd | claude-0328-1000-a1b2 |
| 2026-03-29T19:11Z | specd | ammend | claude-0329-1200-a1b2 |
| 2026-03-29T20:36Z | ammend | in_design | claude-0329-main |
| 2026-03-29T20:38Z | in_design | specd | claude-0329-main |
| 2026-03-29T20:49Z | specd | ready | claude-0329-main |
