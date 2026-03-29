+++
id = 41
title = "Configurable merge strategy for implemented→accepted"
state = "ammend"
priority = 4
effort = 3
risk = 2
author = "claude-0328-1000-a1b2"
branch = "ticket/0041-configurable-merge-strategy-for-implemen"
created_at = "2026-03-28T08:14:53.494909Z"
updated_at = "2026-03-29T19:28:44.107321Z"
+++

## Spec

### Problem

The `implemented → accepted` transition is currently hardcoded to require an
open GitHub PR (`preconditions = ["pr_exists"]`) and relies on the GitHub
provider to detect when it is merged. This works for teams using GitHub PR
reviews, but is the wrong default for many workflows:

- A solo developer or small team may want a **direct merge** — push the branch,
  merge it locally, move on — with no PR required.
- A team that trusts its agents may want an **agent-supervised merge** — apm
  triggers a review agent that checks the diff, runs tests, and merges if they
  pass — without a human touching GitHub.
- A team with its own CI/CD or review tooling may want **none** — apm does
  nothing after `implemented`, and the user's external process handles the rest
  and transitions manually.

The merge strategy should be a first-class config option in `apm.toml`, not
baked into the precondition list of a workflow state.

### Acceptance criteria

- [ ] `apm.toml` supports `[workers] merge_strategy` with four values:
  - `"pr"` — current behaviour: open a GitHub PR; `accepted` fires when the PR
    is merged (requires a GitHub provider)
  - `"direct"` — `apm start` merges the branch to the default branch locally
    and pushes after the worker exits; transitions to `accepted` immediately
  - `"agent"` — after the worker exits, `apm start` spawns a review agent with
    the diff as context; the review agent approves (merge + `accepted`) or
    rejects (transitions back to `in_progress` with a note)
  - `"none"` — `apm start` does nothing after the worker exits beyond pushing
    the branch; the transition to `accepted` is entirely manual
- [ ] Default is `"pr"` (no behaviour change for existing users)
- [ ] `"direct"` aborts and leaves the ticket in `implemented` if the merge
  produces a conflict; prints a clear message asking the user to resolve manually
- [ ] `"agent"` review prompt includes: the ticket spec, the full diff, and
  instructions to approve or reject with a reason
- [ ] `"none"` suppresses the post-worker push (branch push is also skipped);
  the user is responsible for everything after `implemented`
- [ ] `apm verify` prints the configured merge strategy

### Out of scope

- Squash or rebase merge options (always a standard merge commit for now)
- Per-ticket override of the merge strategy
- `"agent"` strategy wiring to specific review models — uses the same `claude`
  CLI as the worker
- GitLab / Bitbucket MR support (tracked separately)

### Approach

**Config** (`apm-core/src/config.rs`):

```rust
#[derive(Debug, Clone, Deserialize, Default, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum MergeStrategy {
    #[default]
    Pr,
    Direct,
    Agent,
    None,
}

// Add to WorkersConfig:
#[serde(default)]
pub merge_strategy: MergeStrategy,
```

**`apm/src/cmd/start.rs`** — post-worker dispatch:

```
match config.workers.merge_strategy {
    MergeStrategy::Pr     => push_branch_and_create_pr(...),
    MergeStrategy::Direct => push_and_merge_direct(...),
    MergeStrategy::Agent  => push_branch_and_spawn_review_agent(...),
    MergeStrategy::None   => { /* do nothing */ }
}
```

`push_and_merge_direct`: runs `git fetch`, `git merge --no-ff <branch>`,
`git push`. On conflict: aborts the merge, prints a message, leaves
ticket in `implemented`.

`push_branch_and_spawn_review_agent`: pushes the branch, then launches
`claude --dangerously-skip-permissions --print` with a system prompt from
`apm.reviewer.md` (created by `apm init`). The review agent runs `apm state
<id> accepted` or `apm state <id> in_progress` based on its verdict.

**`apm/src/cmd/init.rs`**: generate `apm.reviewer.md` when `merge_strategy =
"agent"` is set, or unconditionally alongside `apm.worker.md`.

### Amendment requests

- [ ] The design has changed: merge strategy is no longer a global `[workers]`
  config. It is now a `completion` property on individual
  `[[workflow.states.transitions]]` entries (e.g. `completion = "pr"` on
  `in_progress → implemented`). This allows different transitions to use
  different strategies. Rewrite the entire spec around `completion` on
  transitions instead of `[workers] merge_strategy`.
- [ ] Replace `"pr"` / `"direct"` / `"agent"` / `"none"` values with the
  canonical set from TICKET-LIFECYCLE: `"pr"`, `"merge"`, `"none"`. The
  `"agent"` review strategy is deferred to a separate ticket.
- [ ] The config section `[workers]` is no longer the right home. The
  `completion` property lives on the transition definition in `apm.toml`.
  Update all config structs and field names accordingly.
- [ ] Update the approach section to reflect that `apm state` (not `apm start`)
  reads `completion` and performs the push/PR when transitioning.

## History

| When | From | To | By |
|------|------|----|----|
| 2026-03-28T08:14Z | — | new | claude-0328-1000-a1b2 |
| 2026-03-28T08:16Z | new | specd | claude-0328-1000-a1b2 |
| 2026-03-29T19:11Z | specd | ammend | claude-0329-1200-a1b2 |