+++
id = "4004f5dc"
title = "Trim state.rs to pure state machine logic"
state = "groomed"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/4004f5dc-trim-state-rs-to-pure-state-machine-logi"
created_at = "2026-04-12T06:04:38.471678Z"
updated_at = "2026-04-12T06:12:26.499038Z"
epic = "57bce963"
target_branch = "epic/57bce963-refactor-apm-core-module-structure"
depends_on = ["4f67992b", "eb4789cf"]
+++

## Spec

### Problem

`state.rs` mixes pure state machine logic (`transition`, `available_transitions`, `append_history`) with unrelated concerns: worktree provisioning (`provision_worktree`), GitHub PR creation (`gh_pr_create_or_update`), git merge/pull operations (`merge_into_default`, `pull_default`), and spec document manipulation (`ensure_amendment_section`).

This ticket trims `state.rs` to only the state machine: `transition()`, `available_transitions()`, and `append_history()`. The extracted functions move to `worktree.rs`, `github.rs`, and `git_util.rs` respectively (all of which exist by the time this ticket is worked, per its dependencies).

See [REFACTOR-CORE.md](../../REFACTOR-CORE.md) section 5 for the full plan.

### Acceptance criteria

Checkboxes; each one independently testable.

### Out of scope

Explicit list of what this ticket does not cover.

### Approach

How the implementation will work.

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-04-12T06:04Z | — | new | philippepascal |
| 2026-04-12T06:12Z | new | groomed | apm |
