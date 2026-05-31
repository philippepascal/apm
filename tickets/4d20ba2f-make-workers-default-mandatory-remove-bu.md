+++
id = "4d20ba2f"
title = "Make [workers].default mandatory; remove built-in coder fallback"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/4d20ba2f-make-workers-default-mandatory-remove-bu"
created_at = "2026-05-31T02:58:15.922691Z"
updated_at = "2026-05-31T07:36:32.764726Z"
epic = "9c3c4c20"
target_branch = "epic/9c3c4c20-workflow-schema-cleanup-state-level-work"
+++

## Spec

### Problem

`apm-core/src/start.rs` has three dispatch functions (`run`, `run_next`, `spawn_next_worker`) and one diagnostic function (`resolve_for_diagnostic`), each ending a worker-profile cascade with `.unwrap_or("claude/coder")` or an explicit `else` branch returning `"claude/coder"`. In `validate.rs`, `configured_agent_names` falls back to `"claude"` and `audit_agent_resolution` falls back to `"claude/coder"` via the same pattern. These literals violate the project rule that agent and role names are configuration, not code, and they silently mask missing config — a project that omits `workers.default` dispatches as if it had set `"claude/coder"`, giving no signal that the field is absent.

The fix is to make `[workers].default` mandatory in `config.toml`: change its type from `Option<String>` to `String` (deserialization fails when the key is present but the field is absent), add a validation error when the field is empty (covers the case where `[workers]` is absent entirely and `WorkersConfig::default()` supplies an empty string), and remove every hardcoded `"claude/coder"` fallback from dispatch and validation code. `apm init` already writes the field; no scaffold change is needed.

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
| 2026-05-31T02:58Z | — | new | philippepascal |
| 2026-05-31T07:04Z | new | groomed | philippepascal |
| 2026-05-31T07:36Z | groomed | in_design | philippepascal |