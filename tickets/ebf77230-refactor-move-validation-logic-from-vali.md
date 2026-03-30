+++
id = "ebf77230"
title = "refactor: move validation logic from validate.rs and verify.rs into apm-core"
state = "in_design"
priority = 0
effort = 3
risk = 0
author = "claude-0330-0245-main"
agent = "38718"
branch = "ticket/ebf77230-refactor-move-validation-logic-from-vali"
created_at = "2026-03-30T14:27:38.346647Z"
updated_at = "2026-03-30T16:35:31.701332Z"
+++

## Spec

### Problem

`validate.rs` (257 lines) and `verify.rs` (152 lines) contain validation logic
that belongs in `apm-core`:

**validate.rs** — config integrity checks:
- State ID reference validation (transitions reference valid states)
- Transition precondition and side-effect validation
- Instructions file existence checks
- Provider type validation for PR/Merge completion strategies
- Non-terminal dead-end detection

**verify.rs** — ticket consistency checks:
- Ticket state vs config state validation
- Filename/ID consistency
- Branch requirements by state
- Branch merge status checks
- Agent assignment validation
- Spec section presence checks
- Auto-fix for merged branches (state → accepted)

Both operate purely on data — config structs and ticket structs. `apm-serve`
will want to surface validation errors and consistency warnings in the UI
without shelling out.

Target: `apm_core::validate` and `apm_core::verify` modules. CLI wrappers
format and print results.

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
| 2026-03-30T14:27Z | — | new | claude-0330-0245-main |
| 2026-03-30T14:29Z | new | in_design | claude-0330-0245-main |
| 2026-03-30T14:42Z | claude-0330-0245-main | philippepascal | handoff |
| 2026-03-30T16:27Z | in_design | new | philippepascal |
| 2026-03-30T16:33Z | new | in_design | philippepascal |