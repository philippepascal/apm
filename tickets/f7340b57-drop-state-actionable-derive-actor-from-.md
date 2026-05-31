+++
id = "f7340b57"
title = "Drop state.actionable; derive actor from outgoing triggers"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/f7340b57-drop-state-actionable-derive-actor-from-"
created_at = "2026-05-31T02:56:19.482471Z"
updated_at = "2026-05-31T07:05:07.625091Z"
epic = "9c3c4c20"
target_branch = "epic/9c3c4c20-workflow-schema-cleanup-state-level-work"
+++

## Spec

### Problem

`StateConfig` carries a `Vec<String>` field `actionable` whose only live values across both the `apm` and `syn` workflows are `"agent"` and `"supervisor"`. The information this field encodes is fully derivable from the outgoing transitions already present on the same state: a state is agent-actionable when it has at least one outgoing transition with `trigger = "command:start"`; otherwise it is supervisor-actionable (provided it is non-terminal). Keeping the explicit field invites future inconsistency, where a manual edit moves a state's transitions without updating `actionable`, silently diverging the two representations.

This ticket removes the field entirely and rewrites every callsite to derive actorhood from the trigger shape. Because `deny_unknown_fields` is added to `StateConfig`, any existing workflow file that still contains `actionable = [...]` will fail to parse with a clear TOML error rather than silently ignoring the stale key. Both `apm-core/src/default/workflow.toml` and `.apm/workflow.toml` are migrated as part of the same change. The result is a smaller config struct, a single source of truth for actor assignment, and a parse-time guard against stale config.

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
| 2026-05-31T02:56Z | — | new | philippepascal |
| 2026-05-31T07:04Z | new | groomed | philippepascal |
| 2026-05-31T07:05Z | groomed | in_design | philippepascal |