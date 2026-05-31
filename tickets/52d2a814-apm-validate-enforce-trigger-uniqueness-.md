+++
id = "52d2a814"
title = "apm validate: enforce trigger-uniqueness and worker_profile shape"
state = "in_design"
priority = 5
effort = 3
risk = 2
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/52d2a814-apm-validate-enforce-trigger-uniqueness-"
created_at = "2026-05-31T02:57:37.160432Z"
updated_at = "2026-05-31T19:43:50.480639Z"
epic = "9c3c4c20"
target_branch = "epic/9c3c4c20-workflow-schema-cleanup-state-level-work"
depends_on = ["071886fc"]
+++

## Spec

### Problem

`apm validate` currently enforces that transition targets exist, that terminal states have no outgoing edges, and that merge completions have `on_failure` set. It does not check three structural properties that, when violated, produce silently broken dispatch behaviour at runtime:

1. **Trigger uniqueness.** A `command:start` transition marks its destination state as a fresh dispatch point. If any other transition can also reach that same state, the dispatcher can no longer reliably determine that it should act — the flag becomes ambiguous. This rule applies only to states that receive at least one `command:start` incoming edge; states reachable exclusively via manual transitions may have multiple incoming edges without issue. No error is emitted today when a `command:start` target also has other incoming transitions.

2. **`worker_profile` shape.** Dispatch reads `state.worker_profile` and splits on `/` to extract the agent name and role. A value without a `/`, with empty halves, or with the reserved role `worker` causes a runtime panic or silently falls back to the wrong wrapper. The field is currently accepted without format validation.

3. **`command:start` → dispatch-capable state.** A `command:start` transition that targets a state with no `worker_profile` gives the dispatcher nothing to spawn. This is caught at runtime (no agent is launched) rather than at config-load time.

All three checks are pure additive validation in `validate_config_no_agents`. No existing API changes, no new config fields — malformed `workflow.toml` files are rejected with clear, actionable error messages instead of failing silently at dispatch time.

### Acceptance criteria

- [ ] `apm validate` reports an error when two transitions both target the same state and one of them uses `trigger = "command:start"`, and the error message names both source states and the destination state.
- [ ] `apm validate` reports an error when a manual transition and a `command:start` transition both target the same state, and the error message names both source states.
- [ ] `apm validate` reports an error when a state declares `worker_profile = "claude/worker"` (reserved role).
- [ ] `apm validate` reports an error when a state declares `worker_profile` with no `/` separator (e.g. `"claudecoder"`).
- [ ] `apm validate` reports an error when a state declares `worker_profile` with an empty agent component (e.g. `"/coder"`).
- [ ] `apm validate` reports an error when a state declares `worker_profile` with an empty role component (e.g. `"claude/"`).
- [ ] `apm validate` reports an error when a `command:start` transition targets a state that has no `worker_profile` set.
- [ ] `apm validate` reports no errors for the default `workflow.toml` (as it stands after ticket 071886fc).

### Out of scope

- Validating that exactly one initial state named `"new"` exists — separate ticket.
- Validating that every non-terminal state is reachable from the initial state — separate ticket.
- Unifying the worker command list under `[workers]` — separate ticket.
- Making `[workers].default` mandatory — separate ticket.
- `apm validate --fix` auto-repair for the new rules — separate ticket.
- Help text / `apm validate --help` changes — separate ticket.
- Changes to `apm/src/cmd/validate.rs` — the new checks live entirely in `apm-core/src/validate.rs`.
- Changes to `TransitionConfig` — `worker_profile` is state-level after ticket e05c0463; this ticket only validates `StateConfig.worker_profile`.
- Integration tests in `apm/tests/integration.rs` — unit tests in `apm-core/src/validate.rs` are sufficient.

### Approach

Only one file changes: `apm-core/src/validate.rs`. Add three new validation blocks inside `validate_config_no_agents`, after the existing per-transition checks (after line 444) and before the worktree gitignore check.

#### Rule 1 — Trigger uniqueness

Build a `HashMap<&str, Vec<(&str, &str)>>` mapping each destination state ID to its incoming `(source_state_id, trigger)` pairs:

```
for each state:
  for each transition:
    incoming[transition.to].push((state.id, transition.trigger))
```

For each `(dest, sources)` pair where any entry has `trigger != "manual"`: if `sources.len() > 1`, push an error:

```
"config: state.{dest} — {N} incoming transitions but trigger 'command:start' requires \
 exactly one; incoming from: {src1} (trigger: {t1}), {src2} (trigger: {t2})"
```

Note: `"closed"` is a valid target even when absent from `config.workflow.states`; include it in the incoming map regardless (the duplicate-incoming check is still correct for it).

#### Rule 2 — `worker_profile` shape

For each state where `worker_profile` is `Some(wp)`:

1. Count `/` characters in `wp`. If count ≠ 1, push:
   `"config: state.{id}.worker_profile — '{wp}' must contain exactly one '/' separator"`
2. If count == 1, call `split_once('/')` to get `(agent, role)`:
   - If `agent.is_empty()` or `role.is_empty()`, push:
     `"config: state.{id}.worker_profile — '{wp}' agent and role components must both be non-empty"`
   - If `role == "worker"`, push:
     `"config: state.{id}.worker_profile — role 'worker' is reserved as a process category; use a specific role name"`

#### Rule 3 — `command:start` targets a dispatch-capable state

Build a `HashSet<&str>` of state IDs where `worker_profile.is_some()`:

```
let dispatch_states: HashSet<&str> = config.workflow.states.iter()
    .filter(|s| s.worker_profile.is_some())
    .map(|s| s.id.as_str())
    .collect();
```

For each transition with `trigger == "command:start"` where the target is not in `dispatch_states`, push:

```
"config: state.{src}.transition({dest}) — trigger 'command:start' targets state '{dest}' \
 which has no worker_profile; the dispatcher has nothing to spawn"
```

Skip this check for transitions already flagged by Rule 1 (optional; duplicate errors on the same transition are acceptable since they flag different problems).

#### Tests

Add to the existing `#[cfg(test)] mod tests` in `validate.rs`. Each test builds a minimal TOML config, calls `validate_config_no_agents(&config, Path::new("/tmp"))`, and asserts on the returned error strings.

- `trigger_uniqueness_two_manual_to_same_dest_ok` — two manual transitions to same dest, no error
- `trigger_uniqueness_command_start_plus_manual_same_dest_rejected` — one `command:start` + one `manual` both pointing to the same dest; assert error contains dest ID and both source state IDs
- `trigger_uniqueness_two_command_start_same_dest_rejected` — two `command:start` to same dest; assert error contains dest ID and both source state IDs
- `worker_profile_valid_passes` — well-formed `"claude/coder"`; assert no new errors
- `worker_profile_reserved_role_rejected` — `"claude/worker"`; assert error mentions `"worker"`
- `worker_profile_no_slash_rejected` — `"claudecoder"`; assert error mentions `"exactly one"`
- `worker_profile_empty_agent_rejected` — `"/coder"`; assert error mentions "non-empty"
- `worker_profile_empty_role_rejected` — `"claude/"`; assert error mentions "non-empty"
- `command_start_missing_worker_profile_rejected` — `command:start → state_without_profile`; assert error mentions destination state ID
- `default_workflow_passes` — inline TOML replicating the default workflow structure after 071886fc (groomed → in_design via command:start, ready → in_progress via command:start, in_design and in_progress each carry `worker_profile`); assert `validate_config_no_agents` returns no errors for the new rules

The default-workflow test does not load `apm-core/src/default/workflow.toml` from disk; it embeds the relevant states inline to avoid I/O in unit tests and stay resilient to future file changes.

### Open questions


### Amendment requests

- [ ] Clarify the trigger-uniqueness rule wording in both the spec body and as a one-line comment near the validation code. The clarification should read: only triggered destination states must be unique. States reachable only via manual transitions may have multiple incoming edges. The current wording can be misread as enforcing uniqueness on all destinations.

### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-05-31T02:57Z | — | new | philippepascal |
| 2026-05-31T07:04Z | new | groomed | philippepascal |
| 2026-05-31T07:26Z | groomed | in_design | philippepascal |
| 2026-05-31T07:33Z | in_design | specd | claude |
| 2026-05-31T19:36Z | specd | ammend | philippepascal |
| 2026-05-31T19:43Z | ammend | in_design | philippepascal |