+++
id = "f31ea9ee"
title = "Verify coder Command Reference is properly filtered after epic completes"
state = "closed"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/f31ea9ee-verify-coder-command-reference-is-proper"
created_at = "2026-05-31T02:11:42.431746Z"
updated_at = "2026-05-31T03:04:12.567239Z"
epic = "a42eceea"
target_branch = "epic/a42eceea-workflow-schema-state-level-worker-profi"
depends_on = ["7e66181a", "56500644", "68829abb", "d2a947ea"]
+++

## Spec

### Problem

Verification ticket. After 7e66181a (instructions filter rewrite) lands, double-check that 'apm instructions --role coder' produces a narrowed Command Reference.

PROBLEM (current state on main): running 'apm instructions --role coder' produces a Command Reference section that lists every apm command — including commands a coder would never use (apm sessions, apm revoke, apm work, apm register, apm worktrees, etc.). The spec-writer role IS narrowed (8 commands). The asymmetry suggests the coder allow-list entry is either missing, set to 'all', or the filter is bypassing the coder case.

WHAT TO DO:
- Run 'apm instructions --role coder' after 7e66181a has landed.
- Verify the Command Reference section lists ONLY commands a coder needs: at minimum apm show, apm state, apm spec, apm new --side-note, apm list, apm next. The exact list is for 7e66181a's spec-writer to determine; this ticket just verifies the outcome.
- Verify 'apm instructions --role spec-writer' still produces its narrowed list.
- If the filter is still broken for coder, fix it here.

CONTEXT:
The coder's permitted-commands list in apm-core's role_command_allowlist (or equivalent) needs to align with what is documented in the coder role file (apm-core/src/default/agents/claude/apm.coder.md, currently lists 'apm show, apm state, apm new --side-note, apm spec' under 'Permitted apm commands'). The CLI instructions output should match that documented set, not surface every apm subcommand.

OUT OF SCOPE:
- The instructions filter state-machine logic (handled in 7e66181a).
- Updating the coder role file content.

REFERENCES:
- apm-core/src/instructions.rs::role_command_allowlist (or wherever the per-role list lives)
- apm-core/src/default/agents/claude/apm.coder.md (the source of truth for what commands a coder is allowed)
- 7e66181a (this epic) for the broader instructions filter rewrite

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
| 2026-05-31T02:11Z | — | new | philippepascal |
| 2026-05-31T03:04Z | new | closed | philippepascal |
