+++
id = "e05c0463"
title = "Drop transition.worker_profile (state-level is the only source)"
state = "ready"
priority = 0
effort = 3
risk = 2
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/e05c0463-drop-transition-worker-profile-state-lev"
created_at = "2026-05-31T02:57:03.550888Z"
updated_at = "2026-06-01T00:09:11.523791Z"
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

#### Prerequisite

This ticket builds directly on 28ac0f43. The worker must rebase their branch onto the merged epic branch (`epic/9c3c4c20-*`) after 28ac0f43 lands before starting implementation. All file references below assume `StateConfig.worker_profile` already exists, the dispatch resolution helper (`resolve_dispatch_profile`) already reads it, and the default/project `workflow.toml` files already carry state-level profiles for `in_design` and `in_progress`.

#### 1. apm-core/src/config.rs — TransitionConfig

Remove the `worker_profile: Option<String>` field from `TransitionConfig`. Add `#[serde(deny_unknown_fields)]` to the struct's derive so any workflow file that still carries `worker_profile` under a transition block fails to parse rather than silently ignoring the field.

Wrap **both** TOML parse calls in `Config::load()` with a `map_err` that inspects the error string and returns a migration hint when `worker_profile` appears in the message. Both the `config.toml` parse (line 686) and the `workflow.toml` parse (line 693) can contain transitions, so both need the check. Use the same pattern at each site — swap `.with_context(|| …)` for `.map_err(|e| …)`:

```rust
// config.toml parse site (currently line 686–687)
let mut config: Config = toml::from_str(&contents)
    .map_err(|e| {
        if e.to_string().contains("worker_profile") {
            anyhow::anyhow!(
                "{}: `transition.worker_profile` is no longer supported — \
                 move `worker_profile` to the state block instead",
                path.display()
            )
        } else {
            anyhow::anyhow!("cannot parse {}: {}", path.display(), e)
        }
    })?;

// workflow.toml parse site (currently line 693–694)
let wf: WorkflowFile = toml::from_str(&wf_contents)
    .map_err(|e| {
        if e.to_string().contains("worker_profile") {
            anyhow::anyhow!(
                "{}: `transition.worker_profile` is no longer supported — \
                 move `worker_profile` to the state block instead",
                workflow_path.display()
            )
        } else {
            anyhow::anyhow!("cannot parse {}: {}", workflow_path.display(), e)
        }
    })?;
```

No custom serde visitor is needed; a plain string-match on `e.to_string()` is sufficient.

Also delete the existing tests `transition_config_worker_profile_field` and the `worker_profile` assertion in `transition_config_minimal_parse` (the latter asserts `t.worker_profile.is_none()` on a struct that will no longer have the field).

#### 2. apm-core/src/start.rs — dispatch resolution

In `resolve_dispatch_profile` (added by 28ac0f43), remove the second-tier lookup that reads `triggering_transition.and_then(|tr| tr.worker_profile.as_deref())`. The cascade becomes:

1. `state.worker_profile` (destination state)
2. `workers.default`
3. Built-in `"claude/coder"`

Apply the same removal in `resolve_for_diagnostic`: drop the transition-level branch and update `profile_source` so it never names "transition". The compiler will flag any remaining reads of `TransitionConfig::worker_profile` — treat those as a checklist.

#### 3. apm-core/src/instructions.rs

In `format_live_state_machine`, remove the `owned_by_transition` variable and its `|| owned_by_transition` fallback. Keep only the state-level role check:

```rust
let state_role: Option<&str> = state.worker_profile.as_deref()
    .and_then(|wp| wp.split_once('/').map(|(_, r)| r));
if let Some(role_name) = role {
    if state_role != Some(role_name) {
        continue;
    }
}
```

**Delete `derive_transition_role`.** After removing `owned_by_transition` it has no callers; the compiler confirms. Also delete its two tests: `derive_transition_role_from_worker_profile` and `derive_transition_role_defaults_to_worker`.

Update `live_state_machine_filters_by_role`: remove any assertion that relied on transition-level `worker_profile` producing rows. The `in_design` state carries `worker_profile = "claude/spec-writer"` (added by 28ac0f43), so spec-writer rows now come exclusively from the state. Any test TOML that sets `worker_profile` on a transition block must be moved to the state block.

#### 4. apm-core/src/validate.rs — configured_agent_names

Drop the inner loop that reads `transition.worker_profile`. Keep only the state-level walk added by 28ac0f43. The compiler enforces this once the field is removed.

Note: the original ticket description referenced `apm-core/src/agents.rs`, but `configured_agent_names` lives in `validate.rs` — check both files and edit whichever contains the transition-level walk.

#### 5. apm-core/src/default/workflow.toml

Remove every `worker_profile = "..."` line from transition blocks. State-level equivalents for `in_design` and `in_progress` were added by 28ac0f43; no profile information is lost.

#### 6. .apm/workflow.toml

Same removal as step 5.

#### 7. Tests

Add to `apm-core/src/config.rs` test block:
- `transition_worker_profile_rejected`: inline TOML with `worker_profile = "claude/coder"` under a transition; assert `Config::load`-equivalent parse returns `Err` whose message contains `"worker_profile"` and `"state"`.
- `state_worker_profile_accepted`: inline TOML with `worker_profile` only at the state level; assert parse succeeds.

Add to `apm-core/src/start.rs` test block:
- `dispatch_ignores_transition_worker_profile`: config where no state has `worker_profile` and `workers.default` is set; assert `resolve_dispatch_profile` (or `resolve_for_diagnostic`) returns the `workers.default` value, confirming no transition fallback is consulted.

### Open questions


### Amendment requests

- [x] Add a note acknowledging this ticket assumes 28ac0f43 has landed and merged. The worker should rebase their branch onto current main before starting implementation. Pseudocode and references to state.worker_profile in this ticket assume the field already exists on StateConfig.
- [x] Pin the exact location of the parse-error wrapping site: which of the two parse sites in config.rs (config.toml around line 686 vs workflow.toml around line 693) receives the friendly migration-pointing error message. Specify the exact wrapping pattern (anyhow::Context or thiserror) and the message text.
- [x] Specify whether derive_transition_role in instructions.rs is deleted as part of this ticket or left for a later cleanup. If 28ac0f43 has already migrated the instructions filter to use state.worker_profile, derive_transition_role has no callers and should be deleted here. Otherwise, state the criteria for keeping it.

### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-05-31T02:57Z | — | new | philippepascal |
| 2026-05-31T07:04Z | new | groomed | philippepascal |
| 2026-05-31T07:16Z | groomed | in_design | philippepascal |
| 2026-05-31T07:20Z | in_design | specd | claude |
| 2026-05-31T19:35Z | specd | ammend | philippepascal |
| 2026-05-31T20:04Z | ammend | in_design | philippepascal |
| 2026-05-31T20:07Z | in_design | specd | claude |
| 2026-05-31T21:04Z | specd | ready | philippepascal |
| 2026-05-31T21:39Z | ready | in_progress | philippepascal |
| 2026-06-01T00:09Z | in_progress | ready | philippepascal |
