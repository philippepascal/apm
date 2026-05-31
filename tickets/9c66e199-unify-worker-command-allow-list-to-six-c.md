+++
id = "9c66e199"
title = "Unify worker command allow-list to six commands; remove per-role lists"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/9c66e199-unify-worker-command-allow-list-to-six-c"
created_at = "2026-05-31T02:57:57.400665Z"
updated_at = "2026-05-31T07:33:30.439351Z"
epic = "9c3c4c20"
target_branch = "epic/9c3c4c20-workflow-schema-cleanup-state-level-work"
+++

## Spec

### Problem

STEP 6 of the incremental workflow schema cleanup. Independent of the schema changes; can land in parallel.

PROBLEM: instructions.rs has TWO hardcoded role-specific command lists:

  match role {
    'spec-writer' => Some(8 commands),
    'worker'      => Some(8 commands different from spec-writer),
    _             => None,  // no filter, shows EVERY apm command
  }

Three issues:
- 'coder' is missing → falls to None → coder sees all 30+ apm commands in the Command Reference (the bug spotted in our earlier review)
- The two lists embed role names ('spec-writer', 'worker') in code; per project rule, role names belong in configs, not code
- Workers really only need the same small subset; role-specific lists are overengineering

DESIGN:

Replace with a single hardcoded constant in apm-core/src/instructions.rs:

  const WORKER_COMMAND_ALLOWLIST: &[&str] = &['show', 'state', 'spec', 'set', 'new', 'instructions'];

Every dispatched worker sees this same list in the Command Reference section, regardless of role.

Rationale: a worker's job is to edit tickets (show, state, spec, set, new for side-notes) and bootstrap its session (instructions). Anything else (sync, list, next, start, work, validate, etc.) belongs to supervisors or the orchestrator.

SCOPE:

1. apm-core/src/instructions.rs:
   - Add WORKER_COMMAND_ALLOWLIST constant with the six commands above.
   - Replace role_command_allowlist function with logic that returns the constant when a role is supplied, None otherwise. (Or inline the constant lookup in the Command Reference rendering path; whichever is cleaner.)
   - Delete the per-role match arms (no more 'spec-writer' / 'worker' string-matching for command filtering).

2. Update unit tests that asserted the old per-role lists. Each role now produces the same 6-command Command Reference.

3. Update apm-core/src/default/agents/claude/apm.coder.md and apm-core/src/default/agents/claude/apm.spec-writer.md (and their .apm/ project copies) Permitted apm commands section to match the unified list. Currently coder.md lists show / state / new --side-note / spec; spec-writer.md lists show / spec / set / state / new. Bring both to: show, state, spec, set, new, instructions.

OUT OF SCOPE:
- Schema changes (covered by earlier tickets).
- build_system_prompt empty commands bug (separate ticket).
- Help text sweep (separate ticket).
- The hardcoded 'claude/coder' fallback in start.rs (separate ticket about mandatory workers.default).

TESTS:
- apm instructions --role coder Command Reference section lists exactly: show, state, spec, set, new, instructions.
- apm instructions --role spec-writer Command Reference section lists the same six commands.
- apm instructions --role anything-else lists the same six commands.
- No reference to 'spec-writer' or 'worker' literal strings in instructions.rs after this change (grep for them; they should not appear in code paths, only in tests if at all).

REFERENCES:
- apm-core/src/instructions.rs (role_command_allowlist)
- apm-core/src/default/agents/claude/apm.coder.md
- apm-core/src/default/agents/claude/apm.spec-writer.md
- .apm/agents/claude/apm.coder.md
- .apm/agents/claude/apm.spec-writer.md

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
| 2026-05-31T02:57Z | — | new | philippepascal |
| 2026-05-31T07:04Z | new | groomed | philippepascal |
| 2026-05-31T07:33Z | groomed | in_design | philippepascal |
