+++
id = "e05c0463"
title = "Drop transition.worker_profile (state-level is the only source)"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/e05c0463-drop-transition-worker-profile-state-lev"
created_at = "2026-05-31T02:57:03.550888Z"
updated_at = "2026-05-31T07:16:34.260985Z"
epic = "9c3c4c20"
target_branch = "epic/9c3c4c20-workflow-schema-cleanup-state-level-work"
depends_on = ["28ac0f43"]
+++

## Spec

### Problem

After ticket 28ac0f43 lands, both `StateConfig` and `TransitionConfig` carry `worker_profile`. The state-level field is the authoritative source; the transition-level field is retained only as a fallback during the migration window. With the default and project workflow.toml files updated to carry `worker_profile` at the state level, the transition-level field is now dead code. Leaving it in place keeps fallback paths alive in the dispatch resolution logic, the agent-name scanner, and the instructions formatter — paths that could mask misconfigured workflows and complicate future changes.

This ticket removes `worker_profile` from `TransitionConfig` entirely, adds `#[serde(deny_unknown_fields)]` to enforce the change at parse time, and strips every fallback path that read from the transition-level field. Any workflow.toml that still carries `worker_profile` under a transition block will fail to parse with a message that names the field and describes the fix.

### Acceptance criteria

- [ ] A workflow.toml with `worker_profile` under any `[[workflow.states.transitions]]` block fails to parse
- [ ] The parse error message names `worker_profile` and tells the user to move it to the state block
- [ ] A workflow.toml with `worker_profile` only at the state level parses correctly
- [ ] `apm start` resolves the worker profile via `state.worker_profile` → `workers.default` → built-in; no transition-level lookup occurs
- [ ] `configured_agent_names` collects agent names from state-level `worker_profile` only
- [ ] `format_live_state_machine` filters transitions by role using only state-level `worker_profile`; `derive_transition_role` is deleted
- [ ] `apm-core/src/default/workflow.toml` contains no `worker_profile` keys under any transition block
- [ ] `.apm/workflow.toml` contains no `worker_profile` keys under any transition block
- [ ] `cargo test --workspace` passes

### Out of scope

- Removing the built-in `"claude/coder"` fallback (covered by the later ticket: mandatory workers.default)
- Workflow transition corrections or reordering
- Validate rules for state.worker_profile format
- Any changes to the `workers.default` field behaviour

### Approach

#### 1. apm-core/src/config.rs — TransitionConfig

Remove the `worker_profile: Option<String>` field from `TransitionConfig`. Add `#[serde(deny_unknown_fields)]` to the struct's serde derive so any workflow.toml that still carries `worker_profile` under a transition block fails at parse time.

Wrap the toml parse call (wherever `Config` is loaded — likely a `Config::load` or `Config::from_str` method) with `.map_err` or `.with_context` that checks whether the error string mentions `worker_profile` and, if so, returns a friendlier message: `"workflow.toml: transition.worker_profile is no longer supported — move worker_profile to the state block instead"`. A simple string-match on the error's `to_string()` is sufficient; no custom serde visitor is needed.

#### 2. apm-core/src/start.rs — dispatch resolution

In `resolve_dispatch_profile` (or the equivalent inline logic at each dispatch site), remove the second-tier lookup (`transition.worker_profile`). The cascade becomes:

1. `state.worker_profile` (destination state)
2. `workers.default`
3. Built-in `"claude/coder"`

Apply the same change in `resolve_for_diagnostic`: drop the `transition.worker_profile` branch and update the `profile_source` label so it never names "transition". Since `TransitionConfig` no longer has `worker_profile`, the compiler will flag any remaining reads — use those compile errors to find any dispatch sites that were missed.

#### 3. apm-core/src/instructions.rs

In `format_live_state_machine`, remove the `owned_by_transition` variable and its fallback check. Keep only the state-level role check:

```rust
let state_role: Option<&str> = state.worker_profile.as_deref()
    .and_then(|wp| wp.split_once('/').map(|(_, r)| r));
if let Some(role_name) = role {
    if state_role != Some(role_name) {
        continue;
    }
}
```

Delete `derive_transition_role` — after removing the `owned_by_transition` branch it has no callers; the compiler confirms.

Update tests in this file:
- `live_state_machine_filters_by_role`: remove any assertion that relied on transition-level `worker_profile` producing rows (e.g. groomed→in_design appearing for spec-writer via the transition fallback). The `in_design` state now carries `worker_profile = "claude/spec-writer"` (added by 28ac0f43), so spec-writer rows come from the state.
- Any test that sets up `worker_profile` on a transition in its inline TOML must be updated to use the state block instead.

#### 4. apm-core/src/validate.rs — configured_agent_names

Drop the inner loop that walks `transition.worker_profile`. Keep only the state-level walk added in 28ac0f43. The compiler enforces this once the field is removed from `TransitionConfig`.

Note: the ticket description references `apm-core/src/agents.rs`, but the function `configured_agent_names` was found in `validate.rs` — check both files; edit whichever one contains the transition-level walk.

#### 5. apm-core/src/default/workflow.toml

Remove every `worker_profile = "..."` line from transition blocks. The state-level equivalents for `in_design` and `in_progress` were added by 28ac0f43, so no profile information is lost.

#### 6. .apm/workflow.toml

Same removal as step 5.

#### 7. Tests

Add to the `apm-core/src/config.rs` test block:
- `transition_worker_profile_rejected`: inline TOML where a transition has `worker_profile = "claude/coder"`; assert `Config::from_str()` (or equivalent) returns `Err` whose message contains `"worker_profile"` and a migration hint (`"state"`).
- `state_worker_profile_accepted`: inline TOML with `worker_profile` only at the state level; assert parsing succeeds.

Add to `apm-core/src/start.rs` test block:
- `dispatch_ignores_transition_worker_profile`: config where no state has `worker_profile` and `workers.default` is set; assert `resolve_dispatch_profile` (or `resolve_for_diagnostic`) returns the `workers.default` value, confirming no transition fallback is consulted.

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-05-31T02:57Z | — | new | philippepascal |
| 2026-05-31T07:04Z | new | groomed | philippepascal |
| 2026-05-31T07:16Z | groomed | in_design | philippepascal |