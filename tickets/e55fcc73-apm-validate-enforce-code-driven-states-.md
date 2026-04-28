+++
id = "e55fcc73"
title = "apm validate: enforce code-driven states are declared in workflow.toml"
state = "groomed"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/e55fcc73-apm-validate-enforce-code-driven-states-"
created_at = "2026-04-28T22:42:06.291026Z"
updated_at = "2026-04-28T22:44:08.564690Z"
depends_on = ["50649e84"]
+++

## Spec

### Problem

**The wart:** `apm-core/src/state.rs:161-184` directly writes `state = "merge_failed"` on merge failure during the `in_progress → implemented` transition. This bypasses the state machine entirely — workflow.toml is not consulted. As a result, a ticket can land in a state that the project's `workflow.toml` does not declare, with no transitions defined, leaving it unreachable through `apm state` and visible only via `apm list`.

**Concrete consequence:** a user whose project was init'd before `a7bce26b` (the commit that introduced `merge_failed` to the default template) ends up stuck. Ticket 63f5e6d2 hit this: state `merge_failed`, no transitions out, manual workflow.toml edit required to recover.

**The fix — make workflow.toml the source of truth.**

Two parts:

**1. `apm validate` enforces that every state the code can write is declared in workflow.toml.**

Maintain a registry (in `apm-core/src/state.rs` or a sibling module) listing all "system states" the code can produce — currently just `merge_failed`, but anything future code paths add must register here. `apm validate` walks this registry and the loaded `workflow.toml`; any registered state that is not declared is reported as a config error.

This must integrate with the merged validate from ticket 50649e84 (verify → validate consolidation). Hence the dep.

**2. `apm validate --fix` ports missing states from the default template into the project's workflow.toml.**

For each missing state, find the corresponding block in `apm-core/src/default/workflow.toml` (embedded via `include_str!` or similar — the template is already shipped with the binary) and append it to the project's `.apm/workflow.toml`. Idempotent: re-running `--fix` on an already-correct config is a no-op. Output explicitly names which states were added.

**Hash-trip integration:** the existing hash-trip on config-file changes (b10d957a) covers the case where a user edits their workflow.toml. It does NOT catch the case where the binary changes (new states added in code, config unchanged). `apm validate` is the natural surfacing point regardless — it runs on the next mutating command via the existing hash-trip plumbing, and `apm validate` itself is exempt so users can always run it.

A second-order option, **out of scope for this ticket**: include the binary's build-time hash (or a static "workflow-state-set version") as part of the hash-trip stamp, so a binary upgrade triggers re-validation. Mention as a follow-up but do not implement here.

**Implementation pointers:**

- `apm-core/src/state.rs`: define `pub const SYSTEM_STATES: &[&str] = &["merge_failed"];` (or similar). Any code path that writes a non-user-driven state must add to this list.
- `apm-core/src/validate.rs` (post-50649e84): add a check that walks `SYSTEM_STATES` against `config.workflow.states`. Missing entries → config error.
- `apm/src/cmd/validate.rs` (post-50649e84): in `--fix` path, call a new `apm-core::workflow::port_missing_state(workflow_path, state_id)` helper. The helper extracts the named state's block from `apm-core/src/default/workflow.toml` (via `include_str!`) and appends to `workflow_path`. Use a TOML-aware parser/emitter so existing comments and structure are preserved.

**Acceptance pointers:**

- A fresh `apm init` produces a workflow.toml that passes the new check (because the default template includes `merge_failed`).
- A pre-`a7bce26b` project (no `merge_failed` block in workflow.toml) → `apm validate` fails with an error naming the missing state and the fix command.
- `apm validate --fix` on that project adds the `merge_failed` block from the default template; re-running validate passes.
- The `SYSTEM_STATES` registry contains exactly the states the code is currently capable of writing directly (audit `state.rs` and any other code paths that assign to `t.frontmatter.state`).
- A test asserts that adding a new entry to `SYSTEM_STATES` without a corresponding block in the default template causes `apm validate --fix` to fail with a clear error (i.e., `SYSTEM_STATES` and the default template must stay in sync — both are source-of-truth).

**Out of scope:**

- Recovering ticket 63f5e6d2 specifically (operational, addressed manually).
- The worker-leak / transcript work in ticket 498febe0 (separate concern; this ticket is about workflow.toml correctness, not worker isolation).
- Binary-version stamp in the hash-trip (a follow-up if and when this proves needed).
- A general-purpose "sync project config from default template" command beyond states (e.g., merging instruction files, `ticket.toml` defaults). `apm validate --fix` here only handles workflow states.

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
| 2026-04-28T22:42Z | — | new | philippepascal |
| 2026-04-28T22:44Z | new | groomed | philippepascal |
